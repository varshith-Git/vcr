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
    ///
    /// **All indices are derived and deterministic**
    pub fn build(cpg: &CPG) -> Self {
        let mut indices = Self::new();

        // Build node_edges index (outgoing edges by kind)
        for edge in &cpg.edges {
            indices
                .node_edges
                .entry(edge.from)
                .or_insert_with(HashMap::new)
                .entry(edge.kind)
                .or_insert_with(Vec::new)
                .push(edge.id);
        }

        // Build symbol_to_defs (Symbol nodes defining symbols)
        for node in &cpg.nodes {
            if node.kind == CPGNodeKind::Symbol {
                if let OriginRef::Symbol { symbol_id } = node.origin {
                    indices
                        .symbol_to_defs
                        .entry(symbol_id)
                        .or_insert_with(Vec::new)
                        .push(node.id);
                }
            }
        }

        // Build var_to_uses (DFG values and their uses)
        for node in &cpg.nodes {
            if node.kind == CPGNodeKind::DfgValue {
                if let OriginRef::Dfg { value_id } = node.origin {
                    // Find all edges pointing to this value (uses)
                    for edge in &cpg.edges {
                        if edge.to == node.id && edge.kind == CPGEdgeKind::DataFlow {
                            indices
                                .var_to_uses
                                .entry(value_id)
                                .or_insert_with(Vec::new)
                                .push(edge.from);
                        }
                    }
                }
            }
        }

        // Build func_to_calls (Function nodes and their call sites)
        for edge in &cpg.edges {
            if edge.kind == CPGEdgeKind::Calls {
                // Get the target function node
                if let Some(target_node) = cpg.get_node(edge.to) {
                    if let OriginRef::Function { function_id } = target_node.origin {
                        indices
                            .func_to_calls
                            .entry(function_id)
                            .or_insert_with(Vec::new)
                            .push(edge.from);
                    }
                }
            }
        }

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
    use crate::types::ByteRange;

    #[test]
    fn test_cpg_indices_creation() {
        let indices = CPGIndices::new();
        assert_eq!(indices.symbol_to_defs.len(), 0);
    }

    #[test]
    fn test_cpg_indices_build_empty() {
        let cpg = CPG::new();
        let indices = CPGIndices::build(&cpg);
        
        assert_eq!(indices.node_edges.len(), 0);
        assert_eq!(indices.symbol_to_defs.len(), 0);
        assert_eq!(indices.var_to_uses.len(), 0);
        assert_eq!(indices.func_to_calls.len(), 0);
    }

    #[test]
    fn test_cpg_indices_node_edges() {
        let mut cpg = CPG::new();
        
        // Add nodes
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: FunctionId(1) },
            ByteRange::new(0, 10),
        ));
        
        cpg.add_node(CPGNode::new(
            CPGNodeId(2),
            CPGNodeKind::CfgNode,
            OriginRef::Cfg { node_id: crate::semantic::model::NodeId(1) },
            ByteRange::new(0, 10),
        ));
        
        // Add edge
        cpg.add_edge(CPGEdge::new(
            CPGEdgeId(1),
            CPGEdgeKind::ControlFlow,
            CPGNodeId(1),
            CPGNodeId(2),
        ));
        
        let indices = CPGIndices::build(&cpg);
        
        // Check node_edges index
        let edges = indices.get_edges_from(CPGNodeId(1), CPGEdgeKind::ControlFlow);
        assert!(edges.is_some());
        assert_eq!(edges.unwrap().len(), 1);
    }

    #[test]
    fn test_cpg_indices_determinism() {
        let mut cpg = CPG::new();
        
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Symbol,
            OriginRef::Symbol { symbol_id: crate::semantic::model::SymbolId(42) },
            ByteRange::new(0, 10),
        ));
        
        // Build twice
        let indices1 = CPGIndices::build(&cpg);
        let indices2 = CPGIndices::build(&cpg);
        
        assert_eq!(indices1.symbol_to_defs.len(), indices2.symbol_to_defs.len());
    }
}

