use anyhow::{Context, Result};
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchCratesParams {
    /// Search query for crate names (supports partial matches)
    pub query: String,

    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

/// Crates.io API response structure
#[derive(Debug, Deserialize)]
struct CratesIoSearchResponse {
    crates: Vec<CrateInfo>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CrateInfo {
    name: String,
    description: Option<String>,
    downloads: u64,
    recent_downloads: u64,
    max_version: String,
    documentation: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    total: usize,
}

pub async fn handle(params: SearchCratesParams) -> Result<String> {
    tracing::info!(
        "Searching crates.io for: '{}' (limit: {})",
        params.query,
        params.limit
    );

    // Create HTTP client with timeout
    let client = Client::builder()
        .user_agent("docsrs-mcp/0.1.0")
        .timeout(Duration::from_secs(5))
        .build()
        .context("Failed to create HTTP client")?;

    // Build the search URL
    let search_url = format!(
        "https://crates.io/api/v1/crates?q={}&per_page={}",
        urlencoding::encode(&params.query),
        params.limit
    );

    // Make the request
    let response = client
        .get(&search_url)
        .send()
        .await
        .context("Failed to send request to crates.io")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to search crates: HTTP {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }

    // Parse the response
    let data: CratesIoSearchResponse = response
        .json()
        .await
        .context("Failed to parse crates.io response")?;

    // Format the results
    if data.crates.is_empty() {
        return Ok(format!("No crates found matching \"{}\"", params.query));
    }

    let mut result = format!(
        "Found {} crates matching \"{}\" (showing top {}):\n\n",
        data.meta.total,
        params.query,
        data.crates.len()
    );

    for (index, crate_info) in data.crates.iter().enumerate() {
        result.push_str(&format!(
            "{}. **{}** v{}\n",
            index + 1,
            crate_info.name,
            crate_info.max_version
        ));

        if let Some(desc) = &crate_info.description {
            result.push_str(&format!("   {}\n", desc));
        }

        result.push_str(&format!(
            "   Downloads: {} ({} recent)\n",
            format_number(crate_info.downloads),
            format_number(crate_info.recent_downloads)
        ));

        if let Some(docs) = &crate_info.documentation {
            result.push_str(&format!("   Docs: {}\n", docs));
        }

        result.push('\n');
    }

    Ok(result)
}

/// Helper function to suggest similar crate names
pub async fn suggest_similar_crates(crate_name: &str, limit: usize) -> Result<Vec<String>> {
    tracing::info!("Finding similar crates to: {}", crate_name);

    let params = SearchCratesParams {
        query: crate_name.to_string(),
        limit,
    };

    // Create HTTP client with timeout
    let client = Client::builder()
        .user_agent("docsrs-mcp/0.1.0")
        .timeout(Duration::from_secs(5))
        .build()
        .context("Failed to create HTTP client")?;

    // Build the search URL
    let search_url = format!(
        "https://crates.io/api/v1/crates?q={}&per_page={}",
        urlencoding::encode(&params.query),
        params.limit
    );

    // Make the request
    let response = client
        .get(&search_url)
        .send()
        .await
        .context("Failed to send request to crates.io")?;

    if !response.status().is_success() {
        return Ok(Vec::new()); // Return empty vector on error
    }

    // Parse the response
    let data: CratesIoSearchResponse = response
        .json()
        .await
        .context("Failed to parse crates.io response")?;

    // Extract crate names
    Ok(data.crates.into_iter().map(|c| c.name).collect())
}

/// Format a number with thousand separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for ch in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(ch);
        count += 1;
    }

    result.chars().rev().collect()
}
