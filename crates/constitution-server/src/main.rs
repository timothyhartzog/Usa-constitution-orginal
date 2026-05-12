//! `constitution-server` binary entry point.
//!
//! Loads the binary archive once at startup and serves a read-only REST
//! surface plus the static frontend. Single-process, no database, no
//! background tasks — designed for stateless containerised deployment.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use constitution_archive::Archive;
use constitution_server::{router, AppState};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "constitution-server", version, about)]
struct Cli {
    /// Path to the binary archive built by `constitution-archive build`.
    #[arg(
        long,
        default_value = "data/index/constitution_archive.bin",
        env = "ARCHIVE_PATH"
    )]
    archive: PathBuf,
    /// Bind address.
    #[arg(long, default_value = "127.0.0.1:8080", env = "BIND_ADDR")]
    addr: SocketAddr,
    /// Optional static directory to serve at `/`.
    #[arg(long, env = "STATIC_DIR")]
    static_dir: Option<PathBuf>,
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .with_target(false)
        .compact()
        .init();
}

fn load_archive(path: &PathBuf) -> Result<Archive> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {path:?}"))?;
    Archive::load(&bytes).with_context(|| format!("decoding {path:?}"))
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    let archive = load_archive(&cli.archive)?;
    let stats = archive.stats();
    info!(
        chunks = stats.chunks,
        documents = stats.documents,
        terms = stats.terms,
        events = stats.events,
        citations = stats.citations,
        "loaded archive"
    );

    let mut state = AppState::new(Arc::new(archive));
    if let Some(dir) = cli.static_dir.clone() {
        state = state.with_static_dir(dir);
    }
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(&cli.addr)
        .await
        .with_context(|| format!("binding {}", cli.addr))?;
    info!(addr = %cli.addr, "constitution-server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
