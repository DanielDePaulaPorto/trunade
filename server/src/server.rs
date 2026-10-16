use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};

use crate::config::{Config, expand};
use crate::document::{self, Cell};
use crate::executor::{Control, Executor, Outcome, RunEvent, RunRequest, Stream, TerminalSize};
use crate::judge::{TestCase, Verdict};
use crate::preferences::{self, Preferences};

const WATCH_DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(RustEmbed)]
#[folder = "../web/.output/public"]
#[allow_missing = true]
struct WebAssets;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub file_name: String,
    pub cells: Vec<Cell>,
    pub error: Option<String>,
}

pub struct AppState {
    pub config: Config,
    pub document_path: PathBuf,
    pub executor: Arc<dyn Executor>,
    pub snapshot: watch::Receiver<Snapshot>,
    pub allowed_origins: Vec<String>,
    /// Shared by every open page; a change in one tab reaches the others.
    pub preferences: watch::Sender<Preferences>,
    pub preferences_path: Option<PathBuf>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(websocket))
        .route("/api/document", get(current_document))
        .fallback(static_file)
        .with_state(state)
}

pub fn load_snapshot(path: &Path, config: &Config) -> Snapshot {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    match std::fs::read_to_string(path) {
        Ok(markdown) => Snapshot {
            file_name,
            cells: document::parse(&markdown, |language| {
                config.language(language).map(|(name, _)| name.to_string())
            }),
            error: None,
        },
        Err(error) => Snapshot {
            file_name,
            cells: vec![],
            error: Some(format!("reading {}: {error}", path.display())),
        },
    }
}

/// Re-parses the document whenever it changes on disk. Keep the returned
/// debouncer alive for as long as watching should continue.
pub fn watch_document(
    path: PathBuf,
    config: Config,
    snapshots: watch::Sender<Snapshot>,
) -> Result<Debouncer<notify_debouncer_full::notify::RecommendedWatcher, RecommendedCache>> {
    let directory = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let file_name = path.file_name().map(|name| name.to_os_string());
    let mut debouncer = new_debouncer(WATCH_DEBOUNCE, None, move |result: DebounceEventResult| {
        let Ok(events) = result else { return };
        // Editors often save via rename, so match by file name rather than by inode.
        let touched = events.iter().flat_map(|event| &event.paths).any(|changed| {
            changed.file_name().map(|name| name.to_os_string()) == file_name
        });
        if touched {
            snapshots.send_replace(load_snapshot(&path, &config));
        }
    })?;
    debouncer.watch(&directory, RecursiveMode::NonRecursive)?;
    Ok(debouncer)
}

async fn current_document(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::Json(state.snapshot.borrow().clone())
}

// ---------------------------------------------------------------- WebSocket

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ClientMessage {
    #[serde(rename_all = "camelCase")]
    Run {
        run_id: String,
        cell_id: String,
        source: String,
        cases: Vec<TestCase>,
        cols: u16,
        rows: u16,
    },
    #[serde(rename_all = "camelCase")]
    Input { run_id: String, data: String },
    #[serde(rename_all = "camelCase")]
    Resize { run_id: String, cols: u16, rows: u16 },
    #[serde(rename_all = "camelCase")]
    Stop { run_id: String },
    #[serde(rename_all = "camelCase")]
    OpenEditor { line: usize },
    SavePreferences { preferences: Preferences },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ServerMessage {
    Document {
        snapshot: Snapshot,
    },
    Preferences {
        preferences: Preferences,
    },
    #[serde(rename_all = "camelCase")]
    Compiling { run_id: String },
    #[serde(rename_all = "camelCase")]
    Running { run_id: String, case: usize, interactive: bool },
    #[serde(rename_all = "camelCase")]
    Output { run_id: String, stream: Stream, text: String },
    #[serde(rename_all = "camelCase")]
    CaseFinished {
        run_id: String,
        case: usize,
        outcome: Outcome,
        run_ms: u128,
        verdict: Option<Verdict>,
    },
    #[serde(rename_all = "camelCase")]
    Finished {
        run_id: String,
        outcome: Outcome,
        compile_ms: Option<u128>,
        run_ms: Option<u128>,
    },
    Error {
        message: String,
    },
}

type Outbox = mpsc::UnboundedSender<ServerMessage>;
type Runs = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Control>>>>;

async fn websocket(
    upgrade: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Response {
    // Any web page can open a WebSocket to localhost; only our own page may run code.
    if !origin_allowed(&headers, &state.allowed_origins) {
        return StatusCode::FORBIDDEN.into_response();
    }
    upgrade.on_upgrade(move |socket| session(socket, state))
}

fn origin_allowed(headers: &HeaderMap, allowed: &[String]) -> bool {
    match headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) {
        None => true, // not a browser
        Some(origin) => allowed.iter().any(|allowed| allowed == origin),
    }
}

async fn session(mut socket: WebSocket, state: Arc<AppState>) {
    let (outbox, mut outgoing) = mpsc::unbounded_channel::<ServerMessage>();
    let document_updates = tokio::spawn(forward_documents(state.snapshot.clone(), outbox.clone()));
    let preference_updates = tokio::spawn(forward_preferences(state.preferences.subscribe(), outbox.clone()));
    let runs: Runs = Arc::default();

    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(message) => handle_client_message(message, &state, &outbox, &runs),
                    Err(error) => send(&outbox, ServerMessage::Error { message: error.to_string() }),
                },
                Some(Ok(_)) => {}
                _ => break,
            },
            Some(message) = outgoing.recv() => {
                let Ok(json) = serde_json::to_string(&message) else { continue };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    for control in runs.lock().unwrap().values() {
        let _ = control.send(Control::Stop);
    }
    document_updates.abort();
    preference_updates.abort();
}

async fn forward_documents(mut snapshots: watch::Receiver<Snapshot>, outbox: Outbox) {
    loop {
        let snapshot = snapshots.borrow_and_update().clone();
        send(&outbox, ServerMessage::Document { snapshot });
        if snapshots.changed().await.is_err() {
            return;
        }
    }
}

async fn forward_preferences(mut updates: watch::Receiver<Preferences>, outbox: Outbox) {
    loop {
        let preferences = updates.borrow_and_update().clone();
        send(&outbox, ServerMessage::Preferences { preferences });
        if updates.changed().await.is_err() {
            return;
        }
    }
}

fn save_preferences(state: &AppState, outbox: &Outbox, new: Preferences) {
    if let Some(path) = &state.preferences_path {
        if let Err(error) = preferences::save(path, &new) {
            send(outbox, ServerMessage::Error { message: format!("{error:#}") });
        }
    }
    state.preferences.send_if_modified(|current| {
        let changed = *current != new;
        *current = new;
        changed
    });
}

fn handle_client_message(message: ClientMessage, state: &AppState, outbox: &Outbox, runs: &Runs) {
    match message {
        ClientMessage::Run { run_id, cell_id, source, cases, cols, rows } => {
            let size = TerminalSize { cols, rows };
            match build_request(state, &cell_id, source, cases, size) {
                Ok(request) => start_run(state, run_id, request, outbox, runs),
                Err(message) => send(
                    outbox,
                    ServerMessage::Finished {
                        run_id,
                        outcome: Outcome::Error { message },
                        compile_ms: None,
                        run_ms: None,
                    },
                ),
            }
        }
        ClientMessage::Input { run_id, data } => {
            control_run(runs, &run_id, Control::Input(data.into_bytes()))
        }
        ClientMessage::Resize { run_id, cols, rows } => {
            control_run(runs, &run_id, Control::Resize(TerminalSize { cols, rows }))
        }
        ClientMessage::Stop { run_id } => control_run(runs, &run_id, Control::Stop),
        ClientMessage::SavePreferences { preferences } => save_preferences(state, outbox, preferences),
        ClientMessage::OpenEditor { line } => {
            if let Err(message) = open_editor(&state.config, &state.document_path, line) {
                send(outbox, ServerMessage::Error { message });
            }
        }
    }
}

/// Language, flags, template and timeout come from the document; source and
/// cases come from the browser, which may hold unsaved edits.
fn build_request(
    state: &AppState,
    cell_id: &str,
    source: String,
    cases: Vec<TestCase>,
    terminal_size: TerminalSize,
) -> Result<RunRequest, String> {
    let snapshot = state.snapshot.borrow();
    let cell = document::find_cell(&snapshot.cells, cell_id)
        .ok_or("this block changed in the file; reload it")?;
    let Cell::Code { fence_language, flags, timeout_seconds, before, after, .. } = cell else {
        return Err("not a code block".to_string());
    };
    let (_, language) = state
        .config
        .language(fence_language)
        .ok_or_else(|| format!("no language configured for '{fence_language}'"))?;
    Ok(RunRequest {
        language: language.clone(),
        source: document::assemble(before, &source, after),
        flags: flags.clone(),
        cases,
        timeout: timeout_seconds.map(Duration::from_secs_f64).unwrap_or(state.config.timeout),
        terminal_size,
    })
}

fn start_run(state: &AppState, run_id: String, request: RunRequest, outbox: &Outbox, runs: &Runs) {
    let mut handle = state.executor.start(request);
    runs.lock().unwrap().insert(run_id.clone(), handle.control);
    let outbox = outbox.clone();
    let runs = runs.clone();
    tokio::spawn(async move {
        while let Some(event) = handle.events.recv().await {
            let finished = matches!(event, RunEvent::Finished { .. });
            send(&outbox, server_message(&run_id, event));
            if finished {
                break;
            }
        }
        runs.lock().unwrap().remove(&run_id);
    });
}

fn server_message(run_id: &str, event: RunEvent) -> ServerMessage {
    let run_id = run_id.to_string();
    match event {
        RunEvent::Compiling => ServerMessage::Compiling { run_id },
        RunEvent::Running { case, interactive } => ServerMessage::Running { run_id, case, interactive },
        RunEvent::Output { stream, text } => ServerMessage::Output { run_id, stream, text },
        RunEvent::CaseFinished { case, outcome, run, verdict } => ServerMessage::CaseFinished {
            run_id,
            case,
            outcome,
            run_ms: run.as_millis(),
            verdict,
        },
        RunEvent::Finished { outcome, compile, run } => ServerMessage::Finished {
            run_id,
            outcome,
            compile_ms: compile.map(|duration| duration.as_millis()),
            run_ms: run.map(|duration| duration.as_millis()),
        },
    }
}

fn control_run(runs: &Runs, run_id: &str, control: Control) {
    if let Some(sender) = runs.lock().unwrap().get(run_id) {
        let _ = sender.send(control);
    }
}

fn open_editor(config: &Config, document: &Path, line: usize) -> Result<(), String> {
    let file = document.to_string_lossy();
    let line = line.to_string();
    let command = expand(&config.editor, &[("file", &file), ("line", &line)], &[]);
    let (program, arguments) = command.split_first().ok_or("editor command is empty")?;
    std::process::Command::new(program)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("opening editor ({program}): {error}"))
}

fn send(outbox: &Outbox, message: ServerMessage) {
    let _ = outbox.send(message);
}

// ------------------------------------------------------------- static files

/// Embedded web app first, then files next to the Markdown (images, iframes...).
async fn static_file(State(state): State<Arc<AppState>>, uri: Uri) -> Response {
    let path = percent_decode(uri.path().trim_start_matches('/'));
    let asset_path = if path.is_empty() { "index.html" } else { path.as_str() };
    if let Some(asset) = WebAssets::get(asset_path) {
        return file_response(asset_path, asset.data.into_owned());
    }
    if path.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Trunade! web app not built: run `pnpm generate` in TrunadeCode/web and rebuild.",
        )
            .into_response();
    }
    let directory = state.document_path.parent().unwrap_or(Path::new("/"));
    match read_inside(directory, &path).await {
        Some(bytes) => file_response(&path, bytes),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Reads `relative` only if it resolves inside `directory`.
async fn read_inside(directory: &Path, relative: &str) -> Option<Vec<u8>> {
    let root = tokio::fs::canonicalize(directory).await.ok()?;
    let candidate = tokio::fs::canonicalize(root.join(relative)).await.ok()?;
    if !candidate.starts_with(&root) {
        return None;
    }
    tokio::fs::read(candidate).await.ok()
}

fn file_response(path: &str, bytes: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    ([(header::CONTENT_TYPE, mime.as_ref().to_string())], bytes).into_response()
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let hex = bytes.get(index + 1..index + 3).and_then(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()
        });
        match (bytes[index], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                index += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decoding_handles_spaces_and_utf8() {
        assert_eq!(percent_decode("img/a%20b.png"), "img/a b.png");
        assert_eq!(percent_decode("ol%C3%A1"), "olá");
        assert_eq!(percent_decode("100%"), "100%");
    }

    #[test]
    fn origin_must_match_when_present() {
        let allowed = vec!["http://localhost:4747".to_string()];
        let mut headers = HeaderMap::new();
        assert!(origin_allowed(&headers, &allowed));
        headers.insert(header::ORIGIN, "http://evil.example".parse().unwrap());
        assert!(!origin_allowed(&headers, &allowed));
        headers.insert(header::ORIGIN, "http://localhost:4747".parse().unwrap());
        assert!(origin_allowed(&headers, &allowed));
    }

    #[tokio::test]
    async fn files_outside_the_document_directory_are_not_served() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("ok.txt"), "ok").unwrap();
        assert_eq!(read_inside(directory.path(), "ok.txt").await.unwrap(), b"ok");
        assert!(read_inside(directory.path(), "../../../etc/hosts").await.is_none());
    }
}
