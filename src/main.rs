use anyhow::Result;
use clap::Parser;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

mod backend;

use backend::{PackageManagerHandler, apk::Apk, apt::Apt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value_t = 8090)]
    port: u32,
    #[arg(default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Auto-detect OS and create appropriate backend
    let router = if std::path::Path::new("/etc/alpine-release").exists() {
        tracing::info!("Detected Alpine Linux, using APK backend");
        let handler = PackageManagerHandler::new(Apk::new());
        let service = StreamableHttpService::new(
            move || Ok(handler.clone()),
            LocalSessionManager::default().into(),
            Default::default(),
        );
        axum::Router::new().nest_service("/mcp", service)
    } else if std::path::Path::new("/etc/debian_version").exists() {
        tracing::info!("Detected Debian/Debian-derivative, using APT backend");
        let handler = PackageManagerHandler::new(Apt::new());
        let service = StreamableHttpService::new(
            move || Ok(handler.clone()),
            LocalSessionManager::default().into(),
            Default::default(),
        );
        axum::Router::new().nest_service("/mcp", service)
    } else {
        anyhow::bail!("Unsupported OS: neither Alpine nor Debian detected");
    };

    let tcp_listener =
        tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await;

    Ok(())
}
