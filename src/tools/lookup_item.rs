use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::docs_fetcher::DocsFetcher;
use crate::rustdoc_parser;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LookupItemParams {
    /// Name of the Rust crate
    #[serde(rename = "crateName")]
    pub crate_name: String,

    /// Path to specific item (e.g., "struct.MyStruct" or "fn.my_function")
    #[serde(rename = "itemPath")]
    pub item_path: String,

    /// Specific version or semver range
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Target platform
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

pub async fn handle(fetcher: &DocsFetcher, params: LookupItemParams) -> Result<String> {
    tracing::info!(
        "Looking up item documentation for: {} in crate {} (version: {:?})",
        params.item_path,
        params.crate_name,
        params.version
    );

    // Fetch the rustdoc JSON from docs.rs
    let rustdoc = fetcher
        .fetch_crate_json(
            &params.crate_name,
            params.version.as_deref(),
            params.target.as_deref(),
            None, // format_version not needed for item lookup
        )
        .await?;

    // Find and format the specific item
    let content = rustdoc_parser::find_item(&rustdoc, &params.item_path)?;

    Ok(content)
}
