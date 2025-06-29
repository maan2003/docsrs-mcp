use anyhow::{anyhow, Result};
use rustdoc_types_v48::{Crate as RustdocCrate, Id, Item, ItemEnum, Visibility};

/// Get the first line of documentation, truncated if too long
fn get_first_line(docs: &str) -> String {
    let first_line = docs.lines().next().unwrap_or("").trim();
    if first_line.len() > 100 {
        format!("{}...", &first_line[..97])
    } else {
        first_line.to_string()
    }
}

/// Get the kind of an item as a string
fn get_item_kind(item: &Item) -> &'static str {
    match &item.inner {
        ItemEnum::Module(_) => "Module",
        ItemEnum::Struct(_) => "Struct",
        ItemEnum::Enum(_) => "Enum",
        ItemEnum::Function(_) => "Function",
        ItemEnum::Trait(_) => "Trait",
        ItemEnum::TypeAlias(_) => "Type Alias",
        ItemEnum::Impl(_) => "Implementation",
        ItemEnum::Constant { .. } => "Constant",
        ItemEnum::Static(_) => "Static",
        ItemEnum::Macro(_) => "Macro",
        ItemEnum::ExternCrate { .. } => "External Crate",
        ItemEnum::Use(_) => "Import",
        ItemEnum::Union(_) => "Union",
        ItemEnum::ProcMacro(_) => "Procedural Macro",
        ItemEnum::Primitive(_) => "Primitive",
        ItemEnum::AssocConst { .. } => "Associated Constant",
        ItemEnum::AssocType { .. } => "Associated Type",
        ItemEnum::StructField(_) => "Struct Field",
        ItemEnum::Variant(_) => "Enum Variant",
        ItemEnum::TraitAlias(_) => "Trait Alias",
        ItemEnum::ExternType => "External Type",
    }
}

/// Extract modules from a parent item
fn extract_modules(rustdoc: &RustdocCrate, parent_id: &Id) -> Vec<String> {
    let mut modules = Vec::new();

    if let Some(parent_item) = rustdoc.index.get(parent_id) {
        if let ItemEnum::Module(module) = &parent_item.inner {
            for item_id in &module.items {
                if let Some(item) = rustdoc.index.get(item_id) {
                    if let ItemEnum::Module(_) = &item.inner {
                        if matches!(item.visibility, Visibility::Public) {
                            let desc = item
                                .docs
                                .as_ref()
                                .map(|d| format!(": {}", get_first_line(d)))
                                .unwrap_or_default();
                            if let Some(name) = &item.name {
                                modules.push(format!("- **{}**{}", name, desc));
                            }
                        }
                    }
                }
            }
        }
    }

    modules
}

/// Extract types (structs, enums, traits) from a parent item
fn extract_types(
    rustdoc: &RustdocCrate,
    parent_id: &Id,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();

    if let Some(parent_item) = rustdoc.index.get(parent_id) {
        if let ItemEnum::Module(module) = &parent_item.inner {
            for item_id in &module.items {
                if let Some(item) = rustdoc.index.get(item_id) {
                    if matches!(item.visibility, Visibility::Public) {
                        if let Some(name) = &item.name {
                            let desc = item
                                .docs
                                .as_ref()
                                .map(|d| format!(": {}", get_first_line(d)))
                                .unwrap_or_default();
                            let entry = format!("- **{}**{}", name, desc);

                            match &item.inner {
                                ItemEnum::Struct(_) => structs.push(entry),
                                ItemEnum::Enum(_) => enums.push(entry),
                                ItemEnum::Trait(_) => traits.push(entry),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    (structs, enums, traits)
}

/// Extract functions from a parent item
fn extract_functions(rustdoc: &RustdocCrate, parent_id: &Id) -> Vec<String> {
    let mut functions = Vec::new();

    if let Some(parent_item) = rustdoc.index.get(parent_id) {
        if let ItemEnum::Module(module) = &parent_item.inner {
            for item_id in &module.items {
                if let Some(item) = rustdoc.index.get(item_id) {
                    if let ItemEnum::Function(_) = &item.inner {
                        if matches!(item.visibility, Visibility::Public) {
                            if let Some(name) = &item.name {
                                let desc = item
                                    .docs
                                    .as_ref()
                                    .map(|d| format!(": {}", get_first_line(d)))
                                    .unwrap_or_default();
                                functions.push(format!("- **{}**{}", name, desc));
                            }
                        }
                    }
                }
            }
        }
    }

    functions
}

/// Format struct details
fn format_struct(struct_data: &rustdoc_types_v48::Struct) -> Vec<String> {
    let mut sections = Vec::new();

    sections.push(format!(
        "\n**Struct Type:** {}",
        match struct_data.kind {
            rustdoc_types_v48::StructKind::Plain { .. } => "plain",
            rustdoc_types_v48::StructKind::Tuple(_) => "tuple",
            rustdoc_types_v48::StructKind::Unit => "unit",
        }
    ));

    if !struct_data.impls.is_empty() {
        sections.push(format!(
            "\n**Implementations:** {} impl block(s)",
            struct_data.impls.len()
        ));
    }

    sections
}

/// Format enum details
fn format_enum(enum_data: &rustdoc_types_v48::Enum) -> Vec<String> {
    let mut sections = Vec::new();

    if !enum_data.variants.is_empty() {
        sections.push(format!(
            "\n**Variants:** {} variant(s)",
            enum_data.variants.len()
        ));
    }

    if !enum_data.impls.is_empty() {
        sections.push(format!(
            "\n**Implementations:** {} impl block(s)",
            enum_data.impls.len()
        ));
    }

    sections
}

/// Format function details
fn format_function(func: &rustdoc_types_v48::Function) -> Vec<String> {
    let mut sections = Vec::new();

    let mut attrs = Vec::new();
    if func.header.is_const {
        attrs.push("const");
    }
    if func.header.is_async {
        attrs.push("async");
    }
    if func.header.is_unsafe {
        attrs.push("unsafe");
    }

    if !attrs.is_empty() {
        sections.push(format!("\n**Attributes:** {}", attrs.join(", ")));
    }

    sections
}

/// Format trait details
fn format_trait(trait_data: &rustdoc_types_v48::Trait) -> Vec<String> {
    let mut sections = Vec::new();

    let mut attrs = Vec::new();
    if trait_data.is_auto {
        attrs.push("auto");
    }
    if trait_data.is_unsafe {
        attrs.push("unsafe");
    }

    if !attrs.is_empty() {
        sections.push(format!("\n**Attributes:** {}", attrs.join(", ")));
    }

    if !trait_data.items.is_empty() {
        sections.push(format!(
            "\n**Items:** {} associated item(s)",
            trait_data.items.len()
        ));
    }

    sections
}

/// Format a single item
fn format_item(item: &Item, kind: Option<&str>) -> String {
    let mut sections = Vec::new();

    // Name and type
    if let Some(name) = &item.name {
        sections.push(format!("# {}", name));
    }

    // Kind/Type
    let item_kind = kind.unwrap_or_else(|| get_item_kind(item));
    sections.push(format!("\n**Type:** {}", item_kind));

    // Visibility
    if !matches!(item.visibility, Visibility::Public) {
        sections.push(format!("**Visibility:** {:?}", item.visibility));
    }

    // Documentation
    if let Some(docs) = &item.docs {
        sections.push(format!("\n## Documentation\n{}", docs));
    }

    // Deprecation notice
    if item.deprecation.is_some() {
        sections.push("\n⚠️ **Deprecated**".to_string());
    }

    // Additional details based on inner type
    match &item.inner {
        ItemEnum::Struct(s) => sections.extend(format_struct(s)),
        ItemEnum::Enum(e) => sections.extend(format_enum(e)),
        ItemEnum::Function(f) => sections.extend(format_function(f)),
        ItemEnum::Trait(t) => sections.extend(format_trait(t)),
        _ => {}
    }

    sections.join("\n")
}

/// Parse the main crate information
pub fn parse_crate_info(rustdoc: &RustdocCrate) -> Result<String> {
    let root_item = rustdoc
        .index
        .get(&rustdoc.root)
        .ok_or_else(|| anyhow!("Root item '{}' not found in index", rustdoc.root.0))?;

    let mut sections = Vec::new();

    // Crate name and version
    if let Some(name) = &root_item.name {
        let mut header = format!("# Crate: {}", name);
        if let Some(version) = &rustdoc.crate_version {
            header.push_str(&format!(" v{}", version));
        }
        sections.push(header);
    }

    // Documentation
    if let Some(docs) = &root_item.docs {
        sections.push(format!("\n## Documentation\n{}", docs));
    }

    // Main modules
    let modules = extract_modules(rustdoc, &rustdoc.root);
    if !modules.is_empty() {
        sections.push(format!("\n## Modules\n{}", modules.join("\n")));
    }

    // Main types
    let (structs, enums, traits) = extract_types(rustdoc, &rustdoc.root);
    if !structs.is_empty() {
        sections.push(format!("\n## Structs\n{}", structs.join("\n")));
    }
    if !enums.is_empty() {
        sections.push(format!("\n## Enums\n{}", enums.join("\n")));
    }
    if !traits.is_empty() {
        sections.push(format!("\n## Traits\n{}", traits.join("\n")));
    }

    // Main functions
    let functions = extract_functions(rustdoc, &rustdoc.root);
    if !functions.is_empty() {
        sections.push(format!("\n## Functions\n{}", functions.join("\n")));
    }

    Ok(sections.join("\n"))
}

/// Find and parse a specific item by path
pub fn find_item(rustdoc: &RustdocCrate, item_path: &str) -> Result<String> {
    // First try to find by path in the paths index
    for (id, path_info) in &rustdoc.paths {
        let full_path = path_info.path.join("::");
        if full_path.ends_with(item_path) || path_info.path.last().is_some_and(|p| p == item_path) {
            if let Some(item) = rustdoc.index.get(id) {
                return Ok(format_item(item, Some(&format!("{:?}", path_info.kind))));
            }
        }
    }

    // Fallback: search through all items by name
    let search_name = item_path.split('.').next_back().unwrap_or(item_path);
    for item in rustdoc.index.values() {
        if item.name.as_deref() == Some(search_name) {
            return Ok(format_item(item, None));
        }
    }

    Err(anyhow!("Item '{}' not found in crate", item_path))
}
