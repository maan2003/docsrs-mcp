use anyhow::{anyhow, Context, Result};
use async_compression::tokio::bufread::ZstdDecoder;
use reqwest::Client;
use tokio::io::AsyncReadExt;

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

        // Log response headers for debugging
        tracing::debug!("Response headers for {}: {:?}", url, response.headers());

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
        tracing::debug!("Response is successful, attempting to read body");

        // Check if response is zstd compressed
        let is_zstd = response
            .headers()
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.eq_ignore_ascii_case("zstd"))
            .unwrap_or(false);

        // Get the response body as bytes
        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        tracing::debug!("Body length: {} bytes", bytes.len());

        if bytes.is_empty() {
            return Err(anyhow!("Empty response body from docs.rs"));
        }

        // Decompress if needed
        let body = if is_zstd {
            tracing::debug!("Decompressing zstd content");
            let mut decoder = ZstdDecoder::new(tokio::io::BufReader::new(&bytes[..]));
            let mut decompressed = String::new();
            decoder
                .read_to_string(&mut decompressed)
                .await
                .context("Failed to decompress zstd content")?;
            decompressed
        } else {
            String::from_utf8(bytes.to_vec()).context("Response body is not valid UTF-8")?
        };

        tracing::debug!("Decoded body length: {} chars", body.len());

        if body.trim().is_empty() {
            return Err(anyhow!("Empty response body from docs.rs"));
        }

        // Check if we got HTML instead of JSON (docs.rs returns HTML when JSON is not available)
        if body.trim().starts_with("<!DOCTYPE") || body.trim().starts_with("<html") {
            return Err(anyhow!(
                "Crate '{}' does not have rustdoc JSON available. Docs.rs returned HTML instead. \
         Note: docs.rs only builds rustdoc JSON for crates published after 2023-05-23.",
                crate_name
            ));
        }

        // Log the first part of the response for debugging
        if tracing::enabled!(tracing::Level::DEBUG) {
            let preview = if body.len() > 500 {
                &body[..500]
            } else {
                &body
            };
            tracing::debug!("Response body preview: {}", preview);
        }

        // Just do basic validation that it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&body).with_context(|| {
            format!(
                "Failed to parse response as JSON. Response starts with: {}",
                &body.chars().take(100).collect::<String>()
            )
        })?;

        tracing::info!("Successfully fetched rustdoc JSON for {}", crate_name);

        Ok(body)
    }
}
