//! Semantic graph model (Step 2.1)
//!
//! **FROZEN SCHEMA** - Do not modify without revisiting all downstream code.
//!
//! This module defines the core data structures for semantic analysis:
//! - Control Flow Graph (CFG)
//! - Data Flow Graph (DFG)
//! - Stable identifiers
//!
//! All collections use Vec for deterministic ordering.

use crate::types::{ByteRange, FileId};
use serde::{Deserialize, Serialize};

// ============================================================================
// Identifiers (opaque, deterministic)
// ============================================================================

/// Unique identifier for a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionId(pub u64);

/// Unique identifier for a CFG node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// Unique identifier for a DFG value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ValueId(pub u64);

/// Unique identifier for a DFG edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

/// Unique identifier for a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SymbolId(pub u64);

/// Unique identifier for a scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScopeId(pub u64);

// ============================================================================
// Control Flow Graph (CFG)
// ============================================================================

/// CFG node types (minimal set for Phase 2)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CFGNodeKind {
    /// Function entry point
    Entry,
    
    /// Function exit point
    Exit,
    
    /// Single statement (assignment, call, etc.)
    Statement,
    
    /// Conditional branch (if, match arm)
    Branch,
    
    /// Control flow merge point (after if/else, loop join)
    Merge,
    
    /// Loop entry point
    LoopHeader,
}

/// CFG node with stable ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFGNode {
    /// Unique node identifier
    pub id: NodeId,
    
    /// Node type
    pub kind: CFGNodeKind,
    
    /// Source location
    pub source_range: ByteRange,
    
    /// Optional AST snippet for debugging
    pub statement: Option<String>,
}

/// CFG edge kind (control flow semantics)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CFGEdgeKind {
    /// Normal sequential flow
    Normal,
    
    /// True branch of conditional
    True,
    
    /// False branch of conditional
    False,
    
    /// Break out of loop
    Break,
    
    /// Continue to loop header
    Continue,
}

/// Directed CFG edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFGEdge {
    /// Source node
    pub from: NodeId,
    
    /// Target node
    pub to: NodeId,
    
    /// Edge semantics
    pub kind: CFGEdgeKind,
}

/// Complete Control Flow Graph for one function
///
/// **Determinism guarantee:** nodes and edges are stored in Vec with stable ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFG {
    /// Function this CFG belongs to
    pub function_id: FunctionId,
    
    /// File containing this function
    pub file_id: FileId,
    
    /// All nodes in deterministic order
    pub nodes: Vec<CFGNode>,
    
    /// All edges in deterministic order
    pub edges: Vec<CFGEdge>,
    
    /// Entry node ID
    pub entry: NodeId,
    
    /// Exit node ID
    pub exit: NodeId,
}

impl CFG {
    /// Create a new empty CFG
    pub fn new(function_id: FunctionId, file_id: FileId, entry: NodeId, exit: NodeId) -> Self {
        Self {
            function_id,
            file_id,
            nodes: Vec::new(),
            edges: Vec::new(),
            entry,
            exit,
        }
    }

    /// Add a node to the CFG
    pub fn add_node(&mut self, node: CFGNode) {
        self.nodes.push(node);
    }

    /// Add an edge to the CFG
    pub fn add_edge(&mut self, edge: CFGEdge) {
        self.edges.push(edge);
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&CFGNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Compute hash for determinism testing
    pub fn compute_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        // Hash function ID
        hasher.update(self.function_id.0.to_be_bytes());
        
        // Hash all nodes in order
        for node in &self.nodes {
            hasher.update(node.id.0.to_be_bytes());
            hasher.update(format!("{:?}", node.kind).as_bytes());
        }
        
        // Hash all edges in order
        for edge in &self.edges {
            hasher.update(edge.from.0.to_be_bytes());
            hasher.update(edge.to.0.to_be_bytes());
            hasher.update(format!("{:?}", edge.kind).as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================================
// Data Flow Graph (DFG)
// ============================================================================

/// DFG value kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueKind {
    /// Variable (mutable or immutable)
    Variable {
        /// Variable name
        name: String
    },
    
    /// Constant literal
    Constant {
        /// Constant value representation
        value: String
    },
    
    /// Function parameter
    Parameter {
        /// Parameter name
        name: String,
        /// Parameter position in function signature
        position: usize
    },
    
    /// Temporary (intermediate computation result)
    Temporary,
}

/// DFG value (variable, constant, or temporary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DFGValue {
    /// Unique value identifier
    pub id: ValueId,
    
    /// Value kind
    pub kind: ValueKind,
    
    /// Source location
    pub source_range: ByteRange,
}

/// DFG edge type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DFGEdgeKind {
    /// Variable definition (assignment)
    Definition,
    
    /// Variable use (read)
    Use,
    
    /// Phi-like merge at control flow join (not true SSA)
    PhiLike,
}

/// Directed DFG edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DFGEdge {
    /// Source value
    pub from: ValueId,
    
    /// Target value
    pub to: ValueId,
    
    /// Edge semantics
    pub kind: DFGEdgeKind,
}

/// Data Flow Graph for one function
///
/// **Not SSA form:** We track definitions and uses but don't enforce single assignment.
/// Phi-like nodes approximate control flow merges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DFG {
    /// Function this DFG belongs to
    pub function_id: FunctionId,
    
    /// All values in deterministic order
    pub values: Vec<DFGValue>,
    
    /// All edges in deterministic order
    pub edges: Vec<DFGEdge>,
}

impl DFG {
    /// Create a new empty DFG
    pub fn new(function_id: FunctionId) -> Self {
        Self {
            function_id,
            values: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a value to the DFG
    pub fn add_value(&mut self, value: DFGValue) {
        self.values.push(value);
    }

    /// Add an edge to the DFG
    pub fn add_edge(&mut self, edge: DFGEdge) {
        self.edges.push(edge);
    }

    /// Get a value by ID
    pub fn get_value(&self, id: ValueId) -> Option<&DFGValue> {
        self.values.iter().find(|v| v.id == id)
    }

    /// Compute hash for determinism testing
    pub fn compute_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        // Hash function ID
        hasher.update(self.function_id.0.to_be_bytes());
        
        // Hash all values in order
        for value in &self.values {
            hasher.update(value.id.0.to_be_bytes());
            hasher.update(format!("{:?}", value.kind).as_bytes());
        }
        
        // Hash all edges in order
        for edge in &self.edges {
            hasher.update(edge.from.0.to_be_bytes());
            hasher.update(edge.to.0.to_be_bytes());
            hasher.update(format!("{:?}", edge.kind).as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================================
// Schema Documentation
// ============================================================================

/// **FROZEN GRAPH SCHEMA - Phase 2**
///
/// This schema is frozen at the start of Phase 2 implementation.
/// Any modifications require revisiting all downstream code.
///
/// ## CFG Node Types (6 total)
///
/// 1. **Entry** - Function entry point (one per function)
/// 2. **Exit** - Function exit point (one per function)
/// 3. **Statement** - Single statement (assignment, call, etc.)
/// 4. **Branch** - Conditional branch (if, match)
/// 5. **Merge** - Control flow merge point
/// 6. **LoopHeader** - Loop entry point
///
/// ## CFG Edge Types (5 total)
///
/// 1. **Normal** - Sequential flow
/// 2. **True** - True branch of conditional
/// 3. **False** - False branch of conditional
/// 4. **Break** - Break out of loop
/// 5. **Continue** - Continue to loop header
///
/// ## DFG Value Types (4 total)
///
/// 1. **Variable** - Named variable
/// 2. **Constant** - Literal value
/// 3. **Parameter** - Function parameter
/// 4. **Temporary** - Intermediate result
///
/// ## DFG Edge Types (3 total)
///
/// 1. **Definition** - Variable definition
/// 2. **Use** - Variable use
/// 3. **PhiLike** - Control flow merge (not true SSA)
///
/// ## Ordering Guarantees
///
/// - All `Vec` collections maintain insertion order
/// - Node IDs assigned sequentially
/// - Functions processed in lexical file order
/// - Deterministic hash computation
///
/// ## What's NOT Included (Phase 2)
///
/// - SSA form (approximated with PhiLike)
/// - Type information (deferred to Phase 3)
/// - Expression-level CFG nodes (statements only)
/// - Pointer analysis (deferred to Phase 3)
/// - Taint tracking (deferred to Phase 3)
pub struct FrozenSchema;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_hash_determinism() {
        let mut cfg1 = CFG::new(
            FunctionId(1),
            FileId::new(1),
            NodeId(0),
            NodeId(1),
        );
        
        cfg1.add_node(CFGNode {
            id: NodeId(0),
            kind: CFGNodeKind::Entry,
            source_range: ByteRange::new(0, 1),
            statement: None,
        });
        
        cfg1.add_edge(CFGEdge {
            from: NodeId(0),
            to: NodeId(1),
            kind: CFGEdgeKind::Normal,
        });

        let hash1 = cfg1.compute_hash();
        let hash2 = cfg1.compute_hash();

        assert_eq!(hash1, hash2, "CFG hash must be deterministic");
    }

    #[test]
    fn test_dfg_hash_determinism() {
        let mut dfg1 = DFG::new(FunctionId(1));
        
        dfg1.add_value(DFGValue {
            id: ValueId(0),
            kind: ValueKind::Variable { name: "x".to_string() },
            source_range: ByteRange::new(0, 1),
        });

        let hash1 = dfg1.compute_hash();
        let hash2 = dfg1.compute_hash();

        assert_eq!(hash1, hash2, "DFG hash must be deterministic");
    }
}
