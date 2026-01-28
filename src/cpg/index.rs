//! CPG Indices - read-only, derived, rebuildable (Step 3.3)
//!
//! All indices are derived from the CPG.
//! They can be rebuilt at any time.
//! They live inside CPGEpoch - when epoch dies, indices die.

use crate::cpg::model::*;
use crate::semantic::model::{FunctionId, SymbolId, ValueId};
use std::collections::HashMap;

/// CPG Indices - all derived and rebuildable
pub struct CPGIndices {
    /// Symbol → definitions
    pub symbol_to_defs: HashMap<SymbolId, Vec<CPGNodeId>>,
    
    /// Variable → uses
    pub var_to_uses: HashMap<ValueId, Vec<CPGNodeId>>,
    
    /// Function → call sites
    pub func_to_calls: HashMap<FunctionId, Vec<CPGNodeId>>,
    
    /// Node → outgoing edges (by kind)
    pub node_edges: HashMap<CPGNodeId, HashMap<CPGEdgeKind, Vec<CPGEdgeId>>>,
}

impl CPGIndices {
    /// Create empty indices
    pub fn new() -> Self {
        Self {
            symbol_to_defs: HashMap::new(),
            var_to_uses: HashMap::new(),
            func_to_calls: HashMap::new(),
            node_edges: HashMap::new(),
        }
    }

    /// Build indices from CPG
    pub fn build(cpg: &CPG) -> Self {
        let mut indices = Self::new();

        // Build node_edges index
        for edge in &cpg.edges {
            indices
                .node_edges
                .entry(edge.from)
                .or_insert_with(HashMap::new)
                .entry(edge.kind)
                .or_insert_with(Vec::new)
                .push(edge.id);
        }

        // TODO: Build other indices in Step 3.3

        indices
    }

    /// Get outgoing edges from a node
    pub fn get_edges_from(&self, node: CPGNodeId, kind: CPGEdgeKind) -> Option<&Vec<CPGEdgeId>> {
        self.node_edges
            .get(&node)
            .and_then(|edges_by_kind| edges_by_kind.get(&kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpg_indices_creation() {
        let indices = CPGIndices::new();
        assert_eq!(indices.symbol_to_defs.len(), 0);
    }

    #[test]
    fn test_cpg_indices_build() {
        let cpg = CPG::new();
        let indices = CPGIndices::build(&cpg);
        
        assert_eq!(indices.node_edges.len(), 0);
    }
}
