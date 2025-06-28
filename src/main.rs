use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

mod docs_fetcher;
mod rustdoc_parser;
mod server;
mod tools;

use crate::server::DocsRsServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting docs.rs MCP server");

    // Create an instance of our docs.rs server
    let service = DocsRsServer::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("Failed to start server: {:?}", e);
    })?;

    // Wait for the server to complete
    service.waiting().await?;

    Ok(())
}
