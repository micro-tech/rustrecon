use std::path::Path;
use tree_sitter::Tree;

/// Traverses the Tree-sitter AST and extracts code chunks.
/// This is a basic implementation and can be greatly refined.
///
/// For now, it extracts top-level functions and modules as chunks.
pub fn chunk_code_for_llm(tree: &Tree, content: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let root_node = tree.root_node();
    let source_bytes = content.as_bytes();

    for child in root_node.children(&mut root_node.walk()) {
        match child.kind() {
            "function_item" | "mod_item" | "impl_item" | "struct_item" | "enum_item" => {
                let start_byte = child.start_byte();
                let end_byte = child.end_byte();
                if let Ok(chunk) = std::str::from_utf8(&source_bytes[start_byte..end_byte]) {
                    chunks.push(chunk.to_string());
                }
            }
            // You might want to handle other top-level items or expressions
            _ => {
                // Optionally, include smaller statements or expressions
                // For a more sophisticated approach, this would involve recursive chunking
                // or specific query-based extraction.
            }
        }
    }

    if chunks.is_empty() && !content.is_empty() {
        // If no specific items are found, treat the whole file as one chunk
        chunks.push(content.to_string());
    }

    chunks
}

/// Helper function to get the crate name from a given path.
/// This is a simplified version and might need `cargo_metadata` for robustness.
pub fn get_crate_name_from_path(crate_path: &Path) -> String {
    crate_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown_crate".to_string())
}
