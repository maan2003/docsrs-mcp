use serde::Deserialize;

/// Minimal type to extract just the format version from rustdoc JSON
/// without deserializing the entire document
#[derive(Debug, Deserialize)]
pub struct RustdocVersionInfo {
    pub format_version: u32,
}

/// Extract the format version from raw JSON without full deserialization
pub fn get_format_version(json_str: &str) -> anyhow::Result<u32> {
    let version_info: RustdocVersionInfo = serde_json::from_str(json_str)?;
    Ok(version_info.format_version)
}
