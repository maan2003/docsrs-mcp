use anyhow::{anyhow, Result};

mod v46;
mod v48;
mod v49;
mod v50;
mod v51;
mod v52;
mod v53;
mod version;

use self::version::get_format_version;

/// Parse the main crate information based on the rustdoc format version
pub fn parse_crate_info(json_str: &str) -> Result<String> {
    // First, extract just the format version without full deserialization
    let format_version = get_format_version(json_str)?;

    // Dispatch to the appropriate parser based on version
    match format_version {
        53 => {
            let rustdoc: rustdoc_types_v53::Crate = serde_json::from_str(json_str)?;
            v53::parse_crate_info(&rustdoc)
        }
        52 => {
            let rustdoc: rustdoc_types_v52::Crate = serde_json::from_str(json_str)?;
            v52::parse_crate_info(&rustdoc)
        }
        51 => {
            let rustdoc: rustdoc_types_v51::Crate = serde_json::from_str(json_str)?;
            v51::parse_crate_info(&rustdoc)
        }
        50 => {
            let rustdoc: rustdoc_types_v50::Crate = serde_json::from_str(json_str)?;
            v50::parse_crate_info(&rustdoc)
        }
        49 => {
            let rustdoc: rustdoc_types_v49::Crate = serde_json::from_str(json_str)?;
            v49::parse_crate_info(&rustdoc)
        }
        48 => {
            let rustdoc: rustdoc_types_v48::Crate = serde_json::from_str(json_str)?;
            v48::parse_crate_info(&rustdoc)
        }
        46 => {
            let rustdoc: rustdoc_types_v46::Crate = serde_json::from_str(json_str)?;
            v46::parse_crate_info(&rustdoc)
        }
        _ => Err(anyhow!(
            "Unsupported rustdoc format version: {}. Supported versions: 46, 48-53",
            format_version
        )),
    }
}

/// Find and parse a specific item by path based on the rustdoc format version
pub fn find_item(json_str: &str, item_path: &str) -> Result<String> {
    // First, extract just the format version without full deserialization
    let format_version = get_format_version(json_str)?;

    // Dispatch to the appropriate parser based on version
    match format_version {
        53 => {
            let rustdoc: rustdoc_types_v53::Crate = serde_json::from_str(json_str)?;
            v53::find_item(&rustdoc, item_path)
        }
        52 => {
            let rustdoc: rustdoc_types_v52::Crate = serde_json::from_str(json_str)?;
            v52::find_item(&rustdoc, item_path)
        }
        51 => {
            let rustdoc: rustdoc_types_v51::Crate = serde_json::from_str(json_str)?;
            v51::find_item(&rustdoc, item_path)
        }
        50 => {
            let rustdoc: rustdoc_types_v50::Crate = serde_json::from_str(json_str)?;
            v50::find_item(&rustdoc, item_path)
        }
        49 => {
            let rustdoc: rustdoc_types_v49::Crate = serde_json::from_str(json_str)?;
            v49::find_item(&rustdoc, item_path)
        }
        48 => {
            let rustdoc: rustdoc_types_v48::Crate = serde_json::from_str(json_str)?;
            v48::find_item(&rustdoc, item_path)
        }
        46 => {
            let rustdoc: rustdoc_types_v46::Crate = serde_json::from_str(json_str)?;
            v46::find_item(&rustdoc, item_path)
        }
        _ => Err(anyhow!(
            "Unsupported rustdoc format version: {}. Supported versions: 46, 48-53",
            format_version
        )),
    }
}
