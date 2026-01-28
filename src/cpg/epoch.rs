//! CPG Epoch - owns all CPG data (Step 3.1)
//!
//! CPGEpoch is the 4th epoch in the hierarchy:
//! - IngestionEpoch
//! - ParseEpoch
//! - SemanticEpoch
//! - CPGEpoch ← this one
//!
//! When dropped, all CPG memory is freed.

use crate::cpg::model::CPG;
use crate::cpg::index::CPGIndices;

/// CPG Epoch - owns unified Code Property Graph
///
/// **Memory Safety**: All CPG data lives within this epoch.
/// When the epoch is dropped, all memory is freed automatically.
pub struct CPGEpoch {
    /// Reference to semantic epoch (read-only)
    _semantic_epoch_marker: u64,  // Would be lifetime in real impl
    
    /// The unified CPG
    cpg: CPG,
    
    /// Derived indices (rebuildable)
    indices: CPGIndices,
    
    /// Epoch ID for debugging
    epoch_id: u64,
}

impl CPGEpoch {
    /// Create a new CPG epoch
    pub fn new(_semantic_epoch_marker: u64, epoch_id: u64) -> Self {
        Self {
            _semantic_epoch_marker,
            cpg: CPG::new(),
            indices: CPGIndices::new(),
            epoch_id,
        }
    }

    /// Get reference to CPG (read-only)
    pub fn cpg(&self) -> &CPG {
        &self.cpg
    }

    /// Get mutable reference to CPG (builder only)
    pub(crate) fn cpg_mut(&mut self) -> &mut CPG {
        &mut self.cpg
    }

    /// Get reference to indices (read-only)
    pub fn indices(&self) -> &CPGIndices {
        &self.indices
    }

    /// Rebuild indices from CPG
    pub fn rebuild_indices(&mut self) {
        self.indices = CPGIndices::build(&self.cpg);
    }

    /// Get epoch ID
    pub fn epoch_id(&self) -> u64 {
        self.epoch_id
    }

    /// Get statistics
    pub fn stats(&self) -> CPGEpochStats {
        let cpg_stats = self.cpg.stats();
        CPGEpochStats {
            epoch_id: self.epoch_id,
            total_nodes: cpg_stats.total_nodes,
            total_edges: cpg_stats.total_edges,
        }
    }
}

impl Drop for CPGEpoch {
    fn drop(&mut self) {
        // All CPG data freed automatically
    }
}

/// Statistics about a CPG epoch
#[derive(Debug, Clone)]
pub struct CPGEpochStats {
    pub epoch_id: u64,
    pub total_nodes: usize,
    pub total_edges: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpg_epoch_creation() {
        let epoch = CPGEpoch::new(2,3);
        assert_eq!(epoch.epoch_id(), 3);
    }

    #[test]
    fn test_cpg_epoch_stats() {
        let epoch = CPGEpoch::new(2, 3);
        let stats = epoch.stats();
        
        assert_eq!(stats.epoch_id, 3);
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_edges, 0);
    }
}
