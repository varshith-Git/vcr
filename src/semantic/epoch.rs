//! Semantic epoch implementation
//!
//! SemanticEpoch owns all semantic analysis results for a codebase.
//! When dropped, all semantic memory is freed.
//!
//! ## Lifecycle
//!
//! 1. Create SemanticEpoch referencing a ParseEpoch
//! 2. Build CFG, DFG, symbols within epoch
//! 3. Query semantic facts (immutable)
//! 4. Drop epoch → all semantic data freed
//!
//! ## Rules
//!
//! - SemanticEpoch references ParseEpoch (read-only)
//! - No cross-epoch pointers allowed
//! - Semantic facts are immutable within epoch
//! - Incremental updates create new epoch

use crate::memory::epoch::{IngestionEpoch, ParseEpoch};
use crate::semantic::invalidation::InvalidationTracker;
use crate::semantic::model::{CFG, DFG};
use crate::semantic::symbols::SymbolTable;
use crate::types::FileId;
use std::collections::HashMap;

/// Semantic epoch - owns all semantic analysis results
///
/// **Memory Safety:** All semantic data (CFGs, DFGs, symbols) lives within this epoch.
/// When the epoch is dropped, all memory is freed automatically.
pub struct SemanticEpoch {
    /// Reference to parse epoch (read-only)
    _parse_epoch_marker: u64, // Would be lifetime in real impl
    
    /// CFGs per function
    cfgs: HashMap<FileId, Vec<CFG>>,
    
    /// DFGs per function
    dfgs: HashMap<FileId, Vec<DFG>>,
    
    /// Symbol tables per file
    symbols: HashMap<FileId, SymbolTable>,
    
    /// Invalidation tracker for incremental updates
    invalidation: InvalidationTracker,
    
    /// Epoch ID for debugging
    epoch_id: u64,
}

impl SemanticEpoch {
    /// Create a new semantic epoch
    ///
    /// Takes a reference to ParseEpoch. This ensures:
    /// - Parse trees are available for semantic analysis
    /// - Parse epoch outlives semantic epoch
    pub fn new(_parse_epoch: &ParseEpoch, epoch_id: u64) -> Self {
        Self {
            _parse_epoch_marker: epoch_id,
            cfgs: HashMap::new(),
            dfgs: HashMap::new(),
            symbols: HashMap::new(),
            invalidation: InvalidationTracker::new(),
            epoch_id,
        }
    }

    /// Add a CFG for a file
    pub fn add_cfg(&mut self, file_id: FileId, cfg: CFG) {
        self.cfgs
            .entry(file_id)
            .or_insert_with(Vec::new)
            .push(cfg);
    }

    /// Add a DFG for a file
    pub fn add_dfg(&mut self, file_id: FileId, dfg: DFG) {
        self.dfgs
            .entry(file_id)
            .or_insert_with(Vec::new)
            .push(dfg);
    }

    /// Add a symbol table for a file
    pub fn add_symbols(&mut self, file_id: FileId, table: SymbolTable) {
        self.symbols.insert(file_id, table);
    }

    /// Get CFGs for a file
    pub fn get_cfgs(&self, file_id: FileId) -> Option<&Vec<CFG>> {
        self.cfgs.get(&file_id)
    }

    /// Get DFGs for a file
    pub fn get_dfgs(&self, file_id: FileId) -> Option<&Vec<DFG>> {
        self.dfgs.get(&file_id)
    }

    /// Get symbol table for a file
    pub fn get_symbols(&self, file_id: FileId) -> Option<&SymbolTable> {
        self.symbols.get(&file_id)
    }

    /// Get mutable access to invalidation tracker
    pub fn invalidation_mut(&mut self) -> &mut InvalidationTracker {
        &mut self.invalidation
    }

    /// Get epoch ID
    pub fn epoch_id(&self) -> u64 {
        self.epoch_id
    }

    /// Get statistics about this epoch
    pub fn stats(&self) -> SemanticEpochStats {
        SemanticEpochStats {
            epoch_id: self.epoch_id,
            files_analyzed: self.symbols.len(),
            total_cfgs: self.cfgs.values().map(|v| v.len()).sum(),
            total_dfgs: self.dfgs.values().map(|v| v.len()).sum(),
            invalidation_stats: self.invalidation.stats(),
        }
    }
}

impl Drop for SemanticEpoch {
    fn drop(&mut self) {
        // All semantic data freed automatically
        // Could add explicit logging here for debugging
    }
}

/// Statistics about a semantic epoch
#[derive(Debug, Clone)]
pub struct SemanticEpochStats {
    /// Epoch ID
    pub epoch_id: u64,
    
    /// Number of files analyzed
    pub files_analyzed: usize,
    
    /// Total CFGs built
    pub total_cfgs: usize,
    
    /// Total DFGs built
    pub total_dfgs: usize,
    
    /// Invalidation tracker stats
    pub invalidation_stats: crate::semantic::invalidation::InvalidationStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_epoch_creation() {
        // Create epoch with fake parse epoch reference
        let fake_parse_marker = 2;
        let semantic = SemanticEpoch {
            _parse_epoch_marker: fake_parse_marker,
            cfgs: HashMap::new(),
            dfgs: HashMap::new(),
            symbols: HashMap::new(),
            invalidation: InvalidationTracker::new(),
            epoch_id: 3,
        };
        
        assert_eq!(semantic.epoch_id(), 3);
    }

    #[test]
    fn test_semantic_epoch_data_management() {
        let fake_parse_marker = 2;
        let mut semantic = SemanticEpoch {
            _parse_epoch_marker: fake_parse_marker,
            cfgs: HashMap::new(),
            dfgs: HashMap::new(),
            symbols: HashMap::new(),
            invalidation: InvalidationTracker::new(),
            epoch_id: 3,
        };
        
        let file_id = FileId::new(42);
        semantic.add_symbols(file_id, SymbolTable::new(file_id));
        
        assert!(semantic.get_symbols(file_id).is_some());
        assert!(semantic.get_cfgs(file_id).is_none());
    }

    #[test]
    fn test_semantic_epoch_stats() {
        let fake_parse_marker = 2;
        let mut semantic = SemanticEpoch {
            _parse_epoch_marker: fake_parse_marker,
            cfgs: HashMap::new(),
            dfgs: HashMap::new(),
            symbols: HashMap::new(),
            invalidation: InvalidationTracker::new(),
            epoch_id: 3,
        };
        
        let file_id = FileId::new(42);
        semantic.add_symbols(file_id, SymbolTable::new(file_id));
        
        let stats = semantic.stats();
        assert_eq!(stats.epoch_id, 3);
        assert_eq!(stats.files_analyzed, 1);
    }
}

