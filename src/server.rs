use std::future::Future;
use std::sync::Arc;

use crate::docs_fetcher::DocsFetcher;
use crate::tools::{
    lookup_crate, lookup_item, search_crates, search_crates::suggest_similar_crates,
};
use anyhow::Result;
use reqwest::Client;
use rmcp::{
    Error as McpError, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::tool::Parameters, model::*, tool, tool_handler, tool_router,
};
use std::time::Duration;

#[derive(Clone)]
pub struct DocsRsServer {
    client: Client,
    fetcher: Arc<DocsFetcher>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DocsRsServer {
    pub fn new() -> Self {
        // Create shared HTTP client with optimal settings for docs.rs
        let client = Client::builder()
            .user_agent("docsrs-mcp/0.1.0")
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        let fetcher = Arc::new(DocsFetcher::new(client.clone()));
        Self {
            client,
            fetcher,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Lookup documentation for a Rust crate from docs.rs",
        annotations(
            title = "Lookup Rust Crate Documentation",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn lookup_crate_docs(
        &self,
        Parameters(params): Parameters<lookup_crate::LookupCrateParams>,
    ) -> Result<CallToolResult, McpError> {
        match lookup_crate::handle(&self.fetcher, params.clone()).await {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => {
                let mut error_message = format!("Error: {}", e);

                // If crate not found, suggest similar crates
                if e.to_string().contains("not found") {
                    if let Ok(suggestions) =
                        suggest_similar_crates(&self.client, &params.crate_name, 5).await
                    {
                        // Only show suggestions if we found actual alternatives
                        if !suggestions.is_empty() && !suggestions.contains(&params.crate_name) {
                            error_message.push_str("\n\nDid you mean one of these crates?\n");
                            for suggestion in suggestions {
                                error_message.push_str(&format!("- {}\n", suggestion));
                            }
                        }
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(error_message)]))
            }
        }
    }

    #[tool(
        description = "Lookup documentation for a specific item (struct, function, etc.) in a Rust crate",
        annotations(
            title = "Lookup Rust Item Documentation",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn lookup_item_docs(
        &self,
        Parameters(params): Parameters<lookup_item::LookupItemParams>,
    ) -> Result<CallToolResult, McpError> {
        match lookup_item::handle(&self.fetcher, params).await {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                e
            ))])),
        }
    }

    #[tool(
        description = "Search for Rust crates on crates.io with fuzzy/partial name matching",
        annotations(
            title = "Search Rust Crates",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn search_crates(
        &self,
        Parameters(params): Parameters<search_crates::SearchCratesParams>,
    ) -> Result<CallToolResult, McpError> {
        match search_crates::handle(&self.client, params).await {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                e
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for DocsRsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "docsrs-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "MCP server for accessing Rust crate documentation via docs.rs JSON API. \
                 Use 'lookup_crate_docs' to get an overview of a crate, 'lookup_item_docs' to \
                 find specific items like structs or functions, and 'search_crates' to search \
                 for crates by name on crates.io."
                    .to_string(),
            ),
        }
    }
}
