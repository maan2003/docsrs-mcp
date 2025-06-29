use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::docs_fetcher::DocsFetcher;
use crate::rustdoc_parser;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LookupCrateParams {
    /// Name of the Rust crate to lookup documentation for
    #[serde(rename = "crateName")]
    pub crate_name: String,

    /// Specific version (e.g., "1.0.0") or semver range (e.g., "~4")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Target platform (e.g., "i686-pc-windows-msvc")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

pub async fn handle(fetcher: &DocsFetcher, params: LookupCrateParams) -> Result<String> {
    tracing::info!(
        "Looking up crate documentation for: {} (version: {:?})",
        params.crate_name,
        params.version
    );

    // Fetch the rustdoc JSON from docs.rs
    let json_str = fetcher
        .fetch_crate_json(
            &params.crate_name,
            params.version.as_deref(),
            params.target.as_deref(),
            None,
        )
        .await?;

    // Parse and format the crate information
    let content = rustdoc_parser::parse_crate_info(&json_str)?;

    Ok(content)
}
