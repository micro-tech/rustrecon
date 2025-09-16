use anyhow::Result;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

pub struct Scanner {
    crate_path: PathBuf,
    parser: Parser,
}

impl Scanner {
    pub fn new(crate_path: PathBuf) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language())?;
        Ok(Scanner { crate_path, parser })
    }

    pub fn scan_crate(&mut self) -> Result<Vec<FileAnalysisResult>> {
        let mut results = Vec::new();
        for entry in WalkDir::new(&self.crate_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().map_or(false, |ext| ext == "rs")
                && !self.should_exclude_file(entry.path())
            {
                if let Some(analysis_result) = self.analyze_file(entry.path())? {
                    results.push(analysis_result);
                }
            }
        }
        Ok(results)
    }

    /// Check if a file should be excluded from scanning
    fn should_exclude_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        // Exclude build artifacts and generated files
        let exclude_patterns = [
            "target/",
            "/target/",
            "\\target\\",
            "/build/",
            "\\build\\",
            ".git/",
            "/.git/",
            "\\.git\\",
            "node_modules/",
            "/node_modules/",
            "\\node_modules\\",
            "bindgen.rs",
            "/bindgen.rs",
            "\\bindgen.rs",
            "/tests.rs",
            "\\tests.rs",
        ];

        // Check if path contains any excluded patterns
        for pattern in &exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        // Exclude very large files (likely generated)
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > 500_000 {  // 500KB limit
                return true;
            }
        }

        false
    }

    fn analyze_file(&mut self, path: &Path) -> Result<Option<FileAnalysisResult>> {
        let content = std::fs::read_to_string(path)?;
        let tree = self
            .parser
            .parse(&content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file: {}", path.display()))?;

        // TODO: Implement initial pattern scanning here (e.g., for 'unsafe' keywords)
        // This will be a preliminary scan before LLM analysis.
        // This can involve traversing the tree-sitter AST.
        // Example: iterate over named nodes, check their kind, etc.

        Ok(Some(FileAnalysisResult {
            path: path.to_path_buf(),
            content,
            tree,
            // suspicious_patterns: Vec::new(), // Placeholder
        }))
    }
}

#[derive(Debug)]
pub struct FileAnalysisResult {
    pub path: PathBuf,
    pub content: String,
    pub tree: Tree, // Changed from syn::File to tree_sitter::Tree
                    // pub suspicious_patterns: Vec<SuspiciousPattern>, // Placeholder for patterns found by initial scan
}

// Example of how you might traverse the tree (can be moved to a separate module/function)
// fn traverse_tree(node: Node, source: &[u8]) {
//     let kind = node.kind();
//     let text = node.utf8_text(source).unwrap_or_default();
//     println!("Node kind: {}, Text: {}", kind, text);

//     for child in node.children(&mut node.walk()) {
//         traverse_tree(child, source);
//     }
// }

// #[derive(Debug)]
// pub struct SuspiciousPattern {
//     pub line: usize,
//     pub column: usize,
//     pub pattern_type: String,
//     pub description: String,
// }
