//! CPG Builder - deterministic graph fusion (Step 3.2)
//!
//! **MECHANICAL, NOT CLEVER**
//!
//! Fusion order (fixed):
//! 1. Files (lexical order)
//! 2. Functions (lexical order per file)
//! 3. AST nodes (tree order)
//! 4. CFG nodes (program order)
//! 5. DFG values (definition order)

use crate::cpg::model::*;
use crate::cpg::epoch::CPGEpoch;
use crate::semantic::SemanticEpoch;
use crate::types::ByteRange;
use anyhow::Result;

/// CPG Builder - fuses AST + CFG + DFG
pub struct CPGBuilder {
    /// Next node ID
    next_node_id: u64,
    
    /// Next edge ID
    next_edge_id: u64,
}

impl CPGBuilder {
    /// Create a new CPG builder
    pub fn new() -> Self {
        Self {
            next_node_id: 0,
            next_edge_id: 0,
        }
    }

    /// Build CPG from semantic epoch
    ///
    /// **Order is fixed and deterministic**
    pub fn build(&mut self, _semantic: &SemanticEpoch, cpg_epoch: &mut CPGEpoch) -> Result<()> {
        // Step 1: Files (lexical order)
        // Step 2: Functions (lexical order per file)
        // Step 3: AST nodes (tree order)
        // Step 4: CFG nodes (program order)
        // Step 5: DFG values (definition order)
        
        // TODO: Implement fusion logic in Step 3.2
        
        Ok(())
    }

    /// Get next node ID
    fn next_node_id(&mut self) -> CPGNodeId {
        let id = CPGNodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    /// Get next edge ID
    fn next_edge_id(&mut self) -> CPGEdgeId {
        let id = CPGEdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpg_builder_creation() {
        let builder = CPGBuilder::new();
        assert_eq!(builder.next_node_id, 0);
    }
}
