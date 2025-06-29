use anyhow::{Context, Result, anyhow};
use reqwest::Client;

pub struct DocsFetcher {
    client: Client,
}

impl DocsFetcher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Build the docs.rs JSON URL for a crate
    fn build_json_url(
        &self,
        crate_name: &str,
        version: Option<&str>,
        target: Option<&str>,
        format_version: Option<u32>,
    ) -> String {
        let mut url = format!("https://docs.rs/crate/{}", crate_name);

        // Add version (latest by default)
        url.push('/');
        url.push_str(version.unwrap_or("latest"));

        // Add target if specified
        if let Some(target) = target {
            url.push('/');
            url.push_str(target);
        }

        // Add JSON endpoint
        url.push_str("/json");

        // Add format version if specified
        if let Some(format_version) = format_version {
            url.push('/');
            url.push_str(&format_version.to_string());
        }

        url
    }

    /// Fetch rustdoc JSON for a crate
    pub async fn fetch_crate_json(
        &self,
        crate_name: &str,
        version: Option<&str>,
        target: Option<&str>,
        format_version: Option<u32>,
    ) -> Result<String> {
        let url = self.build_json_url(crate_name, version, target, format_version);

        tracing::info!("Fetching rustdoc JSON from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to docs.rs")?;

        if response.status() == 404 {
            let version_str = version
                .map(|v| format!(" version {}", v))
                .unwrap_or_default();
            return Err(anyhow!(
                "Crate '{}'{} not found. Note: docs.rs started building rustdoc JSON on 2023-05-23, so older releases may not have JSON available yet.",
                crate_name,
                version_str
            ));
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch rustdoc JSON: HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ));
        }

        // Get the response body as text first for better error messages
        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        if body.trim().is_empty() {
            return Err(anyhow!("Empty response body from docs.rs"));
        }

        // Just do basic validation that it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&body)
            .context("Failed to parse rustdoc JSON. The response may not be valid rustdoc JSON.")?;

        tracing::info!("Successfully fetched rustdoc JSON for {}", crate_name);

        Ok(body)
    }
}
