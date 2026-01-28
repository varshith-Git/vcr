//! Incremental parser (Step 1.4)
//!
//! Tree-sitter integration with incremental reparsing.

use crate::io::SourceFile;
use crate::types::{ByteRange, Language, ParsedFile};
use anyhow::{Context, Result};
use std::time::Instant;
use tree_sitter::{InputEdit, Parser, Tree};

/// Incremental parser using Tree-sitter.
pub struct IncrementalParser {
    language: Language,
    parser: Parser,
}

impl IncrementalParser {
    /// Create a new incremental parser for the given language.
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        
        // Set the language
        let ts_language = match language {
            Language::Rust => tree_sitter_rust::language(),
        };
        
        parser.set_language(ts_language)
            .context("Failed to set Tree-sitter language")?;

        Ok(Self { language, parser })
    }

    /// Parse a source file, optionally using an old tree for incremental parsing.
    pub fn parse(
        &mut self,
        file: &dyn SourceFile,
        old_tree: Option<&Tree>,
    ) -> Result<ParsedFile> {
        let start = Instant::now();
        
        let source = file.bytes();
        let tree = self.parser.parse(source, old_tree)
            .context("Failed to parse source file")?;

        let parse_time_us = start.elapsed().as_micros() as u64;

        // For now, we parse the entire file as one range
        let byte_ranges = vec![ByteRange::new(0, source.len())];

        Ok(ParsedFile {
            file_id: file.file_id(),
            tree,
            byte_ranges,
            parse_time_us,
        })
    }

    /// Apply an edit to a tree.
    pub fn apply_edit(&mut self, tree: &mut Tree, edit: InputEdit) {
        tree.edit(&edit);
    }

    /// Get the language this parser is configured for.
    pub fn language(&self) -> Language {
        self.language
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::MmappedFile;
    use crate::types::FileId;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_rust() {
        let temp_file = NamedTempFile::new().unwrap();
        let source = b"fn main() { println!(\"Hello\"); }";
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = MmappedFile::open(temp_file.path(), file_id).unwrap();

        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        assert_eq!(parsed.file_id, file_id);
        assert!(!parsed.tree.root_node().has_error());
        assert!(parsed.parse_time_us > 0);
    }

    #[test]
    fn test_incremental_parse() {
        let temp_file = NamedTempFile::new().unwrap();
        let source1 = b"fn main() {}";
        fs::write(temp_file.path(), source1).unwrap();

        let file_id = FileId::new(1);
        let mmap1 = MmappedFile::open(temp_file.path(), file_id).unwrap();

        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed1 = parser.parse(&mmap1, None).unwrap();

        // Modify file
        let source2 = b"fn main() { let x = 42; }";
        fs::write(temp_file.path(), source2).unwrap();
        let mmap2 = MmappedFile::open(temp_file.path(), file_id).unwrap();

        // Reparse with old tree
        let parsed2 = parser.parse(&mmap2, Some(&parsed1.tree)).unwrap();

        assert!(!parsed2.tree.root_node().has_error());
    }
}
