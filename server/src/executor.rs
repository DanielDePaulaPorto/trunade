//! Compiles and runs one cell. Events stream out while the program runs; the
//! caller can send input, resize the terminal or stop it.
//!
//! With test cases (fixed stdin, maybe an expected output) the program is
//! compiled once and runs on pipes once per case, under the configured timeout.
//! Without cases it runs on a pseudo-terminal so `cin` is interactive; there is
//! no timeout there (the program may be waiting for the user), only Stop.

use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep_until;

use crate::config::{Language, expand};
use crate::judge::{TestCase, Verdict, judge};

const BINARY_NAME: &str = "program";
const READ_BUFFER_BYTES: usize = 8192;
const DRAIN_AFTER_EXIT: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stream {
    Compiler,
    Stdout,
    Stderr,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Compile,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Outcome {
    Exited { code: i32 },
    Signaled { signal: String },
    CompileFailed,
    TimedOut { phase: Phase },
    OutputLimitExceeded { limit: usize },
    Stopped,
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunEvent {
    Compiling,
    /// A case starts (index 0 for an interactive run).
    Running { case: usize, interactive: bool },
    Output { stream: Stream, text: String },
    /// `verdict` only when the case has an expected output and the program exited with 0.
    CaseFinished {
        case: usize,
        outcome: Outcome,
        run: Duration,
        verdict: Option<Verdict>,
    },
    /// After the last case: its outcome, or why no case ran (compile error...).
    Finished {
        outcome: Outcome,
        compile: Option<Duration>,
        run: Option<Duration>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Input(Vec<u8>),
    Resize(TerminalSize),
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub language: Language,
    pub source: String,
    pub flags: Vec<String>,
    /// Empty: one interactive run on a pseudo-terminal.
    pub cases: Vec<TestCase>,
    pub timeout: Duration,
    pub terminal_size: TerminalSize,
}

pub struct RunHandle {
    pub events: mpsc::UnboundedReceiver<RunEvent>,
    pub control: mpsc::UnboundedSender<Control>,
}

pub trait Executor: Send + Sync {
    fn start(&self, request: RunRequest) -> RunHandle;
}

pub struct LocalExecutor {
    compile_timeout: Duration,
    output_limit: usize,
}

impl LocalExecutor {
    pub fn new(compile_timeout: Duration, output_limit: usize) -> Self {
        Self { compile_timeout, output_limit }
    }
}

impl Executor for LocalExecutor {
    fn start(&self, request: RunRequest) -> RunHandle {
        let (event_sender, events) = mpsc::unbounded_channel();
        let (control, control_receiver) = mpsc::unbounded_channel();
        let job = Job {
            request,
            compile_timeout: self.compile_timeout,
            output_limit: self.output_limit,
            events: event_sender,
            control: control_receiver,
            stdout: String::new(),
        };
        tokio::spawn(job.run());
        RunHandle { events, control }
    }
}

struct Job {
    request: RunRequest,
    compile_timeout: Duration,
    output_limit: usize,
    events: mpsc::UnboundedSender<RunEvent>,
    control: mpsc::UnboundedReceiver<Control>,
    /// Standard output of the current case, for the judge.
    stdout: String,
}

/// How a child process ended, before it is mapped to an `Outcome`.
enum ProcessEnd {
    Exited(Outcome),
    TimedOut,
    Stopped,
    OutputLimitExceeded,
}

impl Job {
    async fn run(mut self) {
        let mut compile = None;
        let mut run = None;
        let outcome = self.compile_and_run(&mut compile, &mut run).await;
        self.emit(RunEvent::Finished { outcome, compile, run });
    }

    async fn compile_and_run(
        &mut self,
        compile_time: &mut Option<Duration>,
        run_time: &mut Option<Duration>,
    ) -> Outcome {
        let workdir = match tempfile::Builder::new().prefix("trunade-").tempdir() {
            Ok(dir) => dir,
            Err(error) => return error_outcome(format!("creating temp dir: {error}")),
        };
        let language = self.request.language.clone();
        if let Err(error) = std::fs::write(workdir.path().join(&language.source), &self.request.source)
        {
            return error_outcome(format!("writing source: {error}"));
        }
        let binary = workdir.path().join(BINARY_NAME);
        let binary_text = binary.to_string_lossy().into_owned();
        let placeholders = [("source", language.source.as_str()), ("binary", binary_text.as_str())];

        if let Some(template) = &language.compile {
            self.emit(RunEvent::Compiling);
            let started = Instant::now();
            let command = expand(template, &placeholders, &self.request.flags);
            let end = self.run_compiler(&command, workdir.path()).await;
            *compile_time = Some(started.elapsed());
            match end {
                ProcessEnd::Exited(Outcome::Exited { code: 0 }) => {}
                ProcessEnd::Exited(Outcome::Error { message }) => return error_outcome(message),
                ProcessEnd::Exited(_) => return Outcome::CompileFailed,
                ProcessEnd::TimedOut => return Outcome::TimedOut { phase: Phase::Compile },
                ProcessEnd::Stopped => return Outcome::Stopped,
                ProcessEnd::OutputLimitExceeded => return self.output_limit_outcome(),
            }
        }

        let command = expand(&language.run, &placeholders, &self.request.flags);
        let cases = self.request.cases.clone();
        if cases.is_empty() {
            return self.run_case(0, None, &command, workdir.path(), run_time).await;
        }
        let mut outcome = Outcome::Stopped;
        for (index, case) in cases.iter().enumerate() {
            outcome = self.run_case(index, Some(case), &command, workdir.path(), run_time).await;
            if outcome == Outcome::Stopped {
                break;
            }
        }
        outcome
    }

    /// One run of the compiled program: piped with the case's input, or interactive.
    async fn run_case(
        &mut self,
        index: usize,
        case: Option<&TestCase>,
        command: &[String],
        workdir: &Path,
        total_time: &mut Option<Duration>,
    ) -> Outcome {
        self.emit(RunEvent::Running { case: index, interactive: case.is_none() });
        self.stdout.clear();
        let started = Instant::now();
        let end = match case {
            Some(case) => self.run_piped(command, workdir, &case.stdin).await,
            None => self.run_on_terminal(command, workdir).await,
        };
        let elapsed = started.elapsed();
        *total_time = Some(total_time.unwrap_or_default() + elapsed);
        let outcome = match end {
            ProcessEnd::Exited(outcome) => outcome,
            ProcessEnd::TimedOut => Outcome::TimedOut { phase: Phase::Run },
            ProcessEnd::Stopped => Outcome::Stopped,
            ProcessEnd::OutputLimitExceeded => self.output_limit_outcome(),
        };
        let verdict = match (case.and_then(|case| case.expected.as_deref()), &outcome) {
            (Some(expected), Outcome::Exited { code: 0 }) => Some(judge(expected, &self.stdout)),
            _ => None,
        };
        self.emit(RunEvent::CaseFinished { case: index, outcome: outcome.clone(), run: elapsed, verdict });
        outcome
    }

    async fn run_compiler(&mut self, command: &[String], workdir: &Path) -> ProcessEnd {
        let deadline = tokio::time::Instant::now() + self.compile_timeout;
        match spawn(command, workdir, Stdio::null()) {
            Ok(child) => {
                self.supervise_piped(child, [Stream::Compiler, Stream::Compiler], deadline)
                    .await
            }
            Err(message) => ProcessEnd::Exited(error_outcome(message)),
        }
    }

    async fn run_piped(&mut self, command: &[String], workdir: &Path, stdin: &str) -> ProcessEnd {
        let deadline = tokio::time::Instant::now() + self.request.timeout;
        let mut child = match spawn(command, workdir, Stdio::piped()) {
            Ok(child) => child,
            Err(message) => return ProcessEnd::Exited(error_outcome(message)),
        };
        if let Some(mut pipe) = child.stdin.take() {
            let input = stdin.as_bytes().to_vec();
            // Ignore errors: a program that exits without reading closes the pipe.
            tokio::spawn(async move {
                let _ = pipe.write_all(&input).await;
            });
        }
        self.supervise_piped(child, [Stream::Stdout, Stream::Stderr], deadline)
            .await
    }

    /// Forwards stdout/stderr until both close, enforcing deadline, limit and Stop.
    async fn supervise_piped(
        &mut self,
        mut child: Child,
        streams: [Stream; 2],
        deadline: tokio::time::Instant,
    ) -> ProcessEnd {
        let (chunk_sender, mut chunks) = mpsc::unbounded_channel();
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(read_chunks(stdout, streams[0], chunk_sender.clone()));
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(read_chunks(stderr, streams[1], chunk_sender));
        }
        let mut decoders = [Utf8Decoder::default(), Utf8Decoder::default()];
        let mut written = 0;
        loop {
            tokio::select! {
                chunk = chunks.recv() => {
                    let Some((stream, bytes)) = chunk else { break };
                    written += bytes.len();
                    if written > self.output_limit {
                        let _ = child.kill().await;
                        return ProcessEnd::OutputLimitExceeded;
                    }
                    let decoder = &mut decoders[usize::from(stream != streams[0])];
                    self.emit_output(stream, decoder.push(&bytes));
                }
                _ = sleep_until(deadline) => {
                    let _ = child.kill().await;
                    return ProcessEnd::TimedOut;
                }
                control = self.control.recv() => {
                    if matches!(control, Some(Control::Stop) | None) {
                        let _ = child.kill().await;
                        return ProcessEnd::Stopped;
                    }
                }
            }
        }
        match child.wait().await {
            Ok(status) => ProcessEnd::Exited(outcome_from_status(status)),
            Err(error) => ProcessEnd::Exited(error_outcome(format!("waiting for process: {error}"))),
        }
    }

    async fn run_on_terminal(&mut self, command: &[String], workdir: &Path) -> ProcessEnd {
        match self.supervise_terminal(command, workdir).await {
            Ok(end) => end,
            Err(error) => ProcessEnd::Exited(error_outcome(format!("{error:#}"))),
        }
    }

    async fn supervise_terminal(
        &mut self,
        command: &[String],
        workdir: &Path,
    ) -> anyhow::Result<ProcessEnd> {
        let pair = native_pty_system().openpty(pty_size(self.request.terminal_size))?;
        let mut builder = CommandBuilder::new(&command[0]);
        builder.args(&command[1..]);
        builder.cwd(workdir);
        builder.env("TERM", "xterm-256color");
        let mut child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| anyhow::anyhow!("starting {}: {error}", command[0]))?;
        drop(pair.slave); // otherwise the reader never sees EOF
        let master = pair.master;
        let mut killer = Some(child.clone_killer());

        let mut chunks = spawn_reader(master.try_clone_reader()?);
        let input = spawn_writer(master.take_writer()?);
        let (exit_sender, mut exit) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = exit_sender.send(child.wait());
        });

        let mut decoder = Utf8Decoder::default();
        let mut written = 0;
        let mut status = None;
        let mut stopped = false;
        let mut drain_deadline = None;
        loop {
            tokio::select! {
                chunk = chunks.recv() => {
                    let Some(bytes) = chunk else { break };
                    written += bytes.len();
                    if written > self.output_limit {
                        kill_in_background(killer.take());
                        return Ok(ProcessEnd::OutputLimitExceeded);
                    }
                    self.emit_output(Stream::Terminal, decoder.push(&bytes));
                }
                result = &mut exit, if status.is_none() => {
                    status = Some(result);
                    drain_deadline = Some(tokio::time::Instant::now() + DRAIN_AFTER_EXIT);
                }
                _ = sleep_until(drain_deadline.unwrap_or_else(far_future)), if drain_deadline.is_some() => break,
                control = self.control.recv() => match control {
                    Some(Control::Input(data)) => { let _ = input.send(data); }
                    Some(Control::Resize(size)) => { let _ = master.resize(pty_size(size)); }
                    Some(Control::Stop) | None => {
                        stopped = true;
                        kill_in_background(killer.take());
                    }
                },
            }
        }
        let status = match status {
            Some(status) => status,
            None => exit.await,
        };
        if stopped {
            return Ok(ProcessEnd::Stopped);
        }
        Ok(match status {
            Ok(Ok(status)) => ProcessEnd::Exited(match status.signal() {
                Some(signal) => Outcome::Signaled { signal: signal.to_string() },
                None => Outcome::Exited { code: status.exit_code() as i32 },
            }),
            Ok(Err(error)) => ProcessEnd::Exited(error_outcome(format!("waiting for process: {error}"))),
            Err(_) => ProcessEnd::Exited(error_outcome("process waiter vanished".to_string())),
        })
    }

    fn output_limit_outcome(&self) -> Outcome {
        Outcome::OutputLimitExceeded { limit: self.output_limit }
    }

    fn emit_output(&mut self, stream: Stream, text: String) {
        if stream == Stream::Stdout {
            self.stdout.push_str(&text);
        }
        if !text.is_empty() {
            self.emit(RunEvent::Output { stream, text });
        }
    }

    fn emit(&self, event: RunEvent) {
        let _ = self.events.send(event);
    }
}

fn spawn(command: &[String], workdir: &Path, stdin: Stdio) -> Result<Child, String> {
    let Some((program, arguments)) = command.split_first() else {
        return Err("empty command".to_string());
    };
    Command::new(program)
        .args(arguments)
        .current_dir(workdir)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => format!("command not found: {program}"),
            _ => format!("starting {program}: {error}"),
        })
}

async fn read_chunks(
    mut reader: impl AsyncRead + Unpin,
    stream: Stream,
    chunks: mpsc::UnboundedSender<(Stream, Vec<u8>)>,
) {
    let mut buffer = vec![0; READ_BUFFER_BYTES];
    while let Ok(read) = reader.read(&mut buffer).await {
        if read == 0 || chunks.send((stream, buffer[..read].to_vec())).is_err() {
            break;
        }
    }
}

fn spawn_reader(mut reader: Box<dyn Read + Send>) -> mpsc::UnboundedReceiver<Vec<u8>> {
    let (sender, receiver) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let mut buffer = vec![0; READ_BUFFER_BYTES];
        while let Ok(read) = reader.read(&mut buffer) {
            if read == 0 || sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    receiver
}

fn spawn_writer(mut writer: Box<dyn Write + Send>) -> std::sync::mpsc::Sender<Vec<u8>> {
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        for data in receiver {
            if writer.write_all(&data).and_then(|()| writer.flush()).is_err() {
                break;
            }
        }
    });
    sender
}

/// `ChildKiller::kill` sleeps between SIGHUP and SIGKILL, so keep it off the runtime.
fn kill_in_background(killer: Option<Box<dyn portable_pty::ChildKiller + Send + Sync>>) {
    if let Some(mut killer) = killer {
        std::thread::spawn(move || {
            let _ = killer.kill();
        });
    }
}

fn pty_size(size: TerminalSize) -> PtySize {
    PtySize { rows: size.rows, cols: size.cols, pixel_width: 0, pixel_height: 0 }
}

fn far_future() -> tokio::time::Instant {
    tokio::time::Instant::now() + Duration::from_secs(86_400)
}

fn outcome_from_status(status: std::process::ExitStatus) -> Outcome {
    use std::os::unix::process::ExitStatusExt;
    match (status.code(), status.signal()) {
        (Some(code), _) => Outcome::Exited { code },
        (None, Some(signal)) => Outcome::Signaled { signal: signal_description(signal) },
        (None, None) => error_outcome("process ended without code or signal".to_string()),
    }
}

/// Same wording portable-pty uses for terminal runs (`strsignal`).
fn signal_description(signal: i32) -> String {
    // SAFETY: strsignal returns a pointer to a static or thread-local string.
    let description = unsafe { libc::strsignal(signal) };
    if description.is_null() {
        return format!("Signal {signal}");
    }
    unsafe { std::ffi::CStr::from_ptr(description) }
        .to_string_lossy()
        .into_owned()
}

fn error_outcome(message: String) -> Outcome {
    Outcome::Error { message }
}

/// Turns byte chunks into text without splitting multi-byte characters.
#[derive(Default)]
struct Utf8Decoder {
    pending: Vec<u8>,
}

impl Utf8Decoder {
    fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let complete = match std::str::from_utf8(&self.pending) {
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            _ => self.pending.len(),
        };
        let rest = self.pending.split_off(complete);
        let text = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending = rest;
        text
    }
}

#[cfg(test)]
mod tests;
