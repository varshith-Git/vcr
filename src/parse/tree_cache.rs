//! Parse tree cache (Step 1.4)
//!
//! Manages parse tree reuse across epochs.

use crate::types::FileId;
use std::collections::HashMap;
use tree_sitter::Tree;

/// Cache for parse trees.
///
/// Tracks which trees are still valid and provides them for incremental reparsing.
pub struct TreeCache {
    trees: HashMap<FileId, Tree>,
}

impl TreeCache {
    /// Create a new empty tree cache.
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
        }
    }

    /// Store a parse tree.
    pub fn insert(&mut self, file_id: FileId, tree: Tree) {
        self.trees.insert(file_id, tree);
    }

    /// Get a parse tree if available.
    pub fn get(&self, file_id: FileId) -> Option<&Tree> {
        self.trees.get(&file_id)
    }

    /// Remove a parse tree (e.g., when file is deleted or modified).
    pub fn invalidate(&mut self, file_id: FileId) -> Option<Tree> {
        self.trees.remove(&file_id)
    }

    /// Clear all cached trees.
    pub fn clear(&mut self) {
        self.trees.clear();
    }

    /// Get the number of cached trees.
    pub fn len(&self) -> usize {
        self.trees.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }
}

impl Default for TreeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_cache() {
        let cache = TreeCache::new();
        assert!(cache.is_empty());

        // We would need a real tree to test this properly
        // For now, just test the structure
        assert_eq!(cache.len(), 0);
    }
}
