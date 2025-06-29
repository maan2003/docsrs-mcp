use anyhow::Result;
use clap::{Parser, Subcommand};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

mod docs_fetcher;
mod rustdoc_parser;
mod server;
mod tools;

use crate::server::DocsRsServer;

#[derive(Parser)]
#[command(name = "docsrs-mcp")]
#[command(about = "MCP server for accessing Rust crate documentation via docs.rs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Look up documentation for a Rust crate
    LookupCrate {
        /// Name of the Rust crate
        crate_name: String,

        /// Specific version (e.g., "1.0.0") or semver range (e.g., "~4")
        #[arg(short, long)]
        version: Option<String>,

        /// Target platform (e.g., "i686-pc-windows-msvc")
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Look up documentation for a specific item in a crate
    LookupItem {
        /// Name of the Rust crate
        crate_name: String,

        /// Path to specific item (e.g., "struct.MyStruct" or "fn.my_function")
        item_path: String,

        /// Specific version or semver range
        #[arg(short, long)]
        version: Option<String>,

        /// Target platform
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Search for Rust crates on crates.io
    Search {
        /// Search query for crate names
        query: String,

        /// Maximum number of results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Run as MCP server (default behavior)
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::LookupCrate {
            crate_name,
            version,
            target,
        }) => {
            // Test lookup_crate_docs tool
            tracing::info!("Testing lookup_crate_docs tool");

            let server = DocsRsServer::new();
            let params = tools::lookup_crate::LookupCrateParams {
                crate_name,
                version,
                target,
            };

            match tools::lookup_crate::handle(&server.fetcher, params).await {
                Ok(content) => {
                    println!("{}", content);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::LookupItem {
            crate_name,
            item_path,
            version,
            target,
        }) => {
            // Test lookup_item_docs tool
            tracing::info!("Testing lookup_item_docs tool");

            let server = DocsRsServer::new();
            let params = tools::lookup_item::LookupItemParams {
                crate_name,
                item_path,
                version,
                target,
            };

            match tools::lookup_item::handle(&server.fetcher, params).await {
                Ok(content) => {
                    println!("{}", content);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Search { query, limit }) => {
            // Test search_crates tool
            tracing::info!("Testing search_crates tool");

            let server = DocsRsServer::new();
            let params = tools::search_crates::SearchCratesParams { query, limit };

            match tools::search_crates::handle(&server.client, params).await {
                Ok(content) => {
                    println!("{}", content);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Serve) | None => {
            // Run as MCP server (default behavior)
            tracing::info!("Starting docs.rs MCP server");

            // Create an instance of our docs.rs server
            let service = DocsRsServer::new().serve(stdio()).await.inspect_err(|e| {
                tracing::error!("Failed to start server: {:?}", e);
            })?;

            // Wait for the server to complete
            service.waiting().await?;
        }
    }

    Ok(())
}
