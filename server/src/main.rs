mod config;
mod document;
mod executor;
mod export;
mod judge;
mod preferences;
mod server;
mod usage_log;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::config::Config;
use crate::executor::LocalExecutor;
use crate::server::{AppState, load_snapshot, router, watch_document};
use crate::usage_log::UsageLog;

const DEFAULT_PORT: u16 = 4747;

/// Serve a Markdown lesson whose code blocks run in the browser.
#[derive(Parser)]
#[command(name = "trunade", version)]
struct Cli {
    /// Markdown file to serve.
    file: PathBuf,
    /// Port to listen on (falls back to a free one if taken).
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Do not open the browser.
    #[arg(long)]
    no_open: bool,
    /// Config file to use instead of ~/.config/trunade/config.toml and ./trunade.toml.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Extra origin allowed to open the WebSocket (the Nuxt dev server).
    #[arg(long, hide = true)]
    dev_origin: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let document = cli
        .file
        .canonicalize()
        .with_context(|| format!("cannot open {}", cli.file.display()))?;
    let config = Config::load(&document, cli.config.as_deref())?;

    let listener = bind(cli.port).await?;
    let port = listener.local_addr()?.port();
    let url = format!("http://localhost:{port}/");

    let (snapshots, snapshot) = watch::channel(load_snapshot(&document, &config));
    let preferences_path = preferences::default_path();
    let saved_preferences = preferences_path.as_deref().map(preferences::load).unwrap_or_default();
    let log = Arc::new(open_usage_log(&document));
    log.record("start", serde_json::json!({
        "document": document.file_name().map(|name| name.to_string_lossy()),
        "version": env!("CARGO_PKG_VERSION"),
        "codeBlocks": server::count_code_cells(&snapshot.borrow().cells),
    }));
    let _watcher = watch_document(document.clone(), config.clone(), snapshots, log.clone())?;
    let executor = Arc::new(LocalExecutor::new(config.compile_timeout, config.output_limit_bytes));
    let state = Arc::new(AppState {
        allowed_origins: allowed_origins(port, cli.dev_origin),
        config,
        document_path: document.clone(),
        executor,
        snapshot,
        preferences: watch::Sender::new(saved_preferences),
        preferences_path,
        log,
    });

    println!("Trunade! serving {} at {url}", document.display());
    if !cli.no_open {
        open_browser(&url);
    }
    axum::serve(listener, router(state)).await?;
    Ok(())
}

/// Always on; without a writable data directory the lesson goes on unrecorded.
fn open_usage_log(document: &std::path::Path) -> UsageLog {
    let Some(dir) = usage_log::default_dir() else { return UsageLog::disabled() };
    match UsageLog::create(&dir, document) {
        Ok((log, path)) => {
            println!("Usage log: {}", path.display());
            log
        }
        Err(error) => {
            eprintln!("warning: no usage log in {}: {error}", dir.display());
            UsageLog::disabled()
        }
    }
}

async fn bind(port: u16) -> Result<TcpListener> {
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => Ok(listener),
        Err(_) => TcpListener::bind(("127.0.0.1", 0))
            .await
            .context("binding a local port"),
    }
}

fn allowed_origins(port: u16, dev_origin: Option<String>) -> Vec<String> {
    let mut origins = vec![
        format!("http://localhost:{port}"),
        format!("http://127.0.0.1:{port}"),
    ];
    origins.extend(dev_origin);
    origins
}

fn open_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    if let Err(error) = std::process::Command::new(opener).arg(url).spawn() {
        eprintln!("could not open the browser ({opener}): {error}");
    }
}
