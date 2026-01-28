//! Unified CPG model - schema definition (FROZEN)
//!
//! **This schema is immutable. No changes after commit.**

use crate::types::ByteRange;
use crate::semantic::model::{FunctionId, NodeId as CFGNodeId, ValueId as DFGValueId};
use serde::{Deserialize, Serialize};

/// CPG Node ID - deterministic, sequential, never reused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CPGNodeId(pub u64);

/// CPG Edge ID - deterministic, sequential, never reused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CPGEdgeId(pub u64);

/// CPG Node Kinds (6 types - frozen)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CPGNodeKind {
    /// AST node
    AstNode,
    
    /// CFG control flow node
    CfgNode,
    
    /// DFG data flow value
    DfgValue,
    
    /// Symbol (from symbol table)
    Symbol,
    
    /// Function
    Function,
    
    /// File
    File,
}

/// CPG Edge Kinds (8 types - frozen)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CPGEdgeKind {
    /// AST parent-child edge
    AstParent,
    
    /// AST child-parent edge (reverse of AstParent)
    AstChild,
    
    /// Control flow edge
    ControlFlow,
    
    /// Data flow edge
    DataFlow,
    
    /// Symbol definition edge
    Defines,
    
    /// Symbol use edge
    Uses,
    
    /// Function call edge
    Calls,
    
    /// Points-to edge (from pointer analysis)
    PointsTo,
}

/// Reference back to origin (AST/CFG/DFG)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OriginRef {
    /// From AST (byte range in source)
    Ast { range: ByteRange },
    
    /// From CFG
    Cfg { node_id: CFGNodeId },
    
    /// From DFG
    Dfg { value_id: DFGValueId },
    
    /// From symbol table
    Symbol { symbol_id: crate::semantic::model::SymbolId },
    
    /// Function
    Function { function_id: FunctionId },
    
    /// File
    File { file_id: crate::types::FileId },
}

/// Unified CPG Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPGNode {
    /// Unique node ID (deterministic, sequential)
    pub id: CPGNodeId,
    
    /// Node kind
    pub kind: CPGNodeKind,
    
    /// Origin reference (back to AST/CFG/DFG)
    pub origin: OriginRef,
    
    /// Source location (if applicable)
    pub source_range: ByteRange,
    
    /// Optional label (for debugging)
    pub label: Option<String>,
}

impl CPGNode {
    /// Create a new CPG node
    pub fn new(id: CPGNodeId, kind: CPGNodeKind, origin: OriginRef, source_range: ByteRange) -> Self {
        Self {
            id,
            kind,
            origin,
            source_range,
            label: None,
        }
    }

    /// Create with label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

/// Unified CPG Edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPGEdge {
    /// Unique edge ID (deterministic, sequential)
    pub id: CPGEdgeId,
    
    /// Edge kind
    pub kind: CPGEdgeKind,
    
    /// Source node
    pub from: CPGNodeId,
    
    /// Target node
    pub to: CPGNodeId,
}

impl CPGEdge {
    /// Create a new CPG edge
    pub fn new(id: CPGEdgeId, kind: CPGEdgeKind, from: CPGNodeId, to: CPGNodeId) -> Self {
        Self {
            id,
            kind,
            from,
            to,
        }
    }
}

/// CPG - Complete Code Property Graph
///
/// **Storage**: All nodes and edges in Vec (deterministic order)
/// **IDs**: Sequential, never reused
/// **Immutable**: After construction, read-only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPG {
    /// All nodes (in creation order)
    pub nodes: Vec<CPGNode>,
    
    /// All edges (in creation order)
    pub edges: Vec<CPGEdge>,
}

impl CPG {
    /// Create empty CPG
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node
    pub fn add_node(&mut self, node: CPGNode) {
        self.nodes.push(node);
    }

    /// Add an edge
    pub fn add_edge(&mut self, edge: CPGEdge) {
        self.edges.push(edge);
    }

    /// Get node by ID
    pub fn get_node(&self, id: CPGNodeId) -> Option<&CPGNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get edges from a node
    pub fn get_edges_from(&self, from: CPGNodeId) -> Vec<&CPGEdge> {
        self.edges.iter().filter(|e| e.from == from).collect()
    }

    /// Get edges to a node
    pub fn get_edges_to(&self, to: CPGNodeId) -> Vec<&CPGEdge> {
        self.edges.iter().filter(|e| e.to == to).collect()
    }

    /// Get edges of a specific kind
    pub fn get_edges_of_kind(&self, kind: CPGEdgeKind) -> Vec<&CPGEdge> {
        self.edges.iter().filter(|e| e.kind == kind).collect()
    }

    /// Get nodes of a specific kind
    pub fn get_nodes_of_kind(&self, kind: CPGNodeKind) -> Vec<&CPGNode> {
        self.nodes.iter().filter(|n| n.kind == kind).collect()
    }

    /// Get statistics
    pub fn stats(&self) -> CPGStats {
        CPGStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            nodes_by_kind: [
                (CPGNodeKind::AstNode, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::AstNode).count()),
                (CPGNodeKind::CfgNode, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::CfgNode).count()),
                (CPGNodeKind::DfgValue, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::DfgValue).count()),
                (CPGNodeKind::Symbol, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::Symbol).count()),
                (CPGNodeKind::Function, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::Function).count()),
                (CPGNodeKind::File, self.nodes.iter().filter(|n| n.kind == CPGNodeKind::File).count()),
            ].into_iter().collect(),
        }
    }
}

/// CPG statistics
#[derive(Debug, Clone)]
pub struct CPGStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub nodes_by_kind: std::collections::HashMap<CPGNodeKind, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpg_node_creation() {
        let node = CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: FunctionId(42) },
            ByteRange::new(0, 10),
        );

        assert_eq!(node.id, CPGNodeId(1));
        assert_eq!(node.kind, CPGNodeKind::Function);
    }

    #[test]
    fn test_cpg_edge_creation() {
        let edge = CPGEdge::new(
            CPGEdgeId(1),
            CPGEdgeKind::ControlFlow,
            CPGNodeId(1),
            CPGNodeId(2),
        );

        assert_eq!(edge.from, CPGNodeId(1));
        assert_eq!(edge.to, CPGNodeId(2));
    }

    #[test]
    fn test_cpg_storage() {
        let mut cpg = CPG::new();

        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        cpg.add_edge(CPGEdge::new(
            CPGEdgeId(1),
            CPGEdgeKind::Calls,
            CPGNodeId(1),
            CPGNodeId(2),
        ));

        assert_eq!(cpg.nodes.len(), 1);
        assert_eq!(cpg.edges.len(), 1);
    }

    #[test]
    fn test_cpg_queries() {
        let mut cpg = CPG::new();

        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        let functions = cpg.get_nodes_of_kind(CPGNodeKind::Function);
        assert_eq!(functions.len(), 1);
    }
}
