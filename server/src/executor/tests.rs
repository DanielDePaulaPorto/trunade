use std::time::Duration;

use super::*;
use crate::config::Config;

const SIZE: TerminalSize = TerminalSize { cols: 80, rows: 24 };

fn language(name: &str) -> Language {
    Config::default().languages[name].clone()
}

fn has_command(language: &Language) -> bool {
    let program = language.compile.as_ref().unwrap_or(&language.run)[0].clone();
    std::process::Command::new("which")
        .arg(program)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn request(name: &str, source: &str, stdin: Option<&str>) -> RunRequest {
    RunRequest {
        language: language(name),
        source: source.to_string(),
        flags: vec![],
        stdin: stdin.map(str::to_string),
        timeout: Duration::from_secs(10),
        terminal_size: SIZE,
    }
}

fn executor() -> LocalExecutor {
    LocalExecutor::new(Duration::from_secs(60), 64 * 1024)
}

struct Finished {
    outcome: Outcome,
    output: Vec<(Stream, String)>,
}

impl Finished {
    fn text(&self, stream: Stream) -> String {
        self.output
            .iter()
            .filter(|(s, _)| *s == stream)
            .map(|(_, text)| text.as_str())
            .collect()
    }
}

async fn collect(mut handle: RunHandle) -> Finished {
    let mut output = Vec::new();
    let wait = async {
        while let Some(event) = handle.events.recv().await {
            match event {
                RunEvent::Output { stream, text } => output.push((stream, text)),
                RunEvent::Finished { outcome, .. } => return outcome,
                _ => {}
            }
        }
        panic!("events ended without Finished");
    };
    let outcome = tokio::time::timeout(Duration::from_secs(60), wait)
        .await
        .expect("run did not finish");
    Finished { outcome, output }
}

async fn run(name: &str, source: &str, stdin: Option<&str>) -> Option<Finished> {
    let request = request(name, source, stdin);
    if !has_command(&request.language) {
        eprintln!("skipping: {name} toolchain not installed");
        return None;
    }
    Some(collect(executor().start(request)).await)
}

#[tokio::test]
async fn piped_program_writes_stdout_and_stderr() {
    let Some(result) = run(
        "cpp",
        "#include <iostream>\nint main() { std::cout << \"out\"; std::cerr << \"err\"; }\n",
        Some(""),
    )
    .await
    else {
        return;
    };
    assert_eq!(result.outcome, Outcome::Exited { code: 0 });
    assert_eq!(result.text(Stream::Stdout), "out");
    assert_eq!(result.text(Stream::Stderr), "err");
}

#[tokio::test]
async fn compile_error_streams_compiler_output() {
    let Some(result) = run("cpp", "int main() { return x; }\n", Some("")).await else { return };
    assert_eq!(result.outcome, Outcome::CompileFailed);
    assert!(result.text(Stream::Compiler).contains("not declared"));
}

#[tokio::test]
async fn nonzero_exit_code_is_reported() {
    let Some(result) = run("cpp", "int main() { return 3; }\n", Some("")).await else { return };
    assert_eq!(result.outcome, Outcome::Exited { code: 3 });
}

#[tokio::test]
async fn segfault_is_reported_as_signal() {
    let source = "int main() { volatile int* p = nullptr; return *p; }\n";
    let Some(result) = run("cpp", source, Some("")).await else { return };
    let Outcome::Signaled { signal } = result.outcome else {
        panic!("expected signal, got {:?}", result.outcome);
    };
    assert!(signal.contains("Segmentation"), "{signal}");
}

#[tokio::test]
async fn piped_infinite_loop_times_out() {
    let mut request = request("cpp", "int main() { for (;;) {} }\n", Some(""));
    if !has_command(&request.language) {
        return;
    }
    request.timeout = Duration::from_secs(1);
    let result = collect(executor().start(request)).await;
    assert_eq!(result.outcome, Outcome::TimedOut { phase: Phase::Run });
}

#[tokio::test]
async fn fixed_stdin_is_forwarded() {
    let source = "#include <iostream>\nint main() { int a, b; std::cin >> a >> b; std::cout << a + b; }\n";
    let Some(result) = run("cpp", source, Some("2 40\n")).await else { return };
    assert_eq!(result.text(Stream::Stdout), "42");
}

#[tokio::test]
async fn output_flood_hits_the_limit() {
    let source = "#include <cstdio>\nint main() { for (;;) std::puts(\"spam\"); }\n";
    let Some(result) = run("cpp", source, Some("")).await else { return };
    assert_eq!(result.outcome, Outcome::OutputLimitExceeded { limit: 64 * 1024 });
}

#[tokio::test]
async fn flags_reach_the_compiler() {
    let mut request = request("cpp", "int main() { return VALUE; }\n", Some(""));
    if !has_command(&request.language) {
        return;
    }
    request.flags = vec!["-DVALUE=7".to_string()];
    let result = collect(executor().start(request)).await;
    assert_eq!(result.outcome, Outcome::Exited { code: 7 });
}

#[tokio::test]
async fn missing_compiler_is_an_error() {
    let mut request = request("cpp", "int main() {}\n", Some(""));
    request.language.compile = Some(vec!["trunade-no-such-compiler".to_string()]);
    let result = collect(executor().start(request)).await;
    assert_eq!(
        result.outcome,
        Outcome::Error { message: "command not found: trunade-no-such-compiler".to_string() }
    );
}

#[tokio::test]
async fn interpreted_language_runs_without_compile_step() {
    let Some(result) = run("python", "print('hi')\n", Some("")).await else { return };
    assert_eq!(result.outcome, Outcome::Exited { code: 0 });
    assert_eq!(result.text(Stream::Stdout), "hi\n");
}

#[tokio::test]
async fn terminal_run_receives_interactive_input() {
    let source = "#include <iostream>\n#include <string>\nint main() { std::string s; std::getline(std::cin, s); std::cout << \"hello \" << s << \"\\n\"; }\n";
    let request = request("cpp", source, None);
    if !has_command(&request.language) {
        return;
    }
    let mut handle = executor().start(request);
    loop {
        match handle.events.recv().await {
            Some(RunEvent::Running { interactive }) => {
                assert!(interactive);
                break;
            }
            Some(RunEvent::Finished { outcome, .. }) => panic!("finished early: {outcome:?}"),
            _ => {}
        }
    }
    handle.control.send(Control::Input(b"Ana\n".to_vec())).unwrap();
    let result = collect(handle).await;
    assert_eq!(result.outcome, Outcome::Exited { code: 0 });
    assert!(result.text(Stream::Terminal).contains("hello Ana"));
}

#[tokio::test]
async fn terminal_run_can_be_stopped() {
    let request = request("cpp", "int main() { for (;;) {} }\n", None);
    if !has_command(&request.language) {
        return;
    }
    let mut handle = executor().start(request);
    while !matches!(handle.events.recv().await, Some(RunEvent::Running { .. })) {}
    handle.control.send(Control::Stop).unwrap();
    assert_eq!(collect(handle).await.outcome, Outcome::Stopped);
}

#[test]
fn utf8_decoder_keeps_split_characters_together() {
    let mut decoder = Utf8Decoder::default();
    let bytes = "olá".as_bytes();
    assert_eq!(decoder.push(&bytes[..3]), "ol");
    assert_eq!(decoder.push(&bytes[3..]), "á");
}
