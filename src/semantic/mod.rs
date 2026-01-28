//! Semantic analysis module (Phase 2)
//!
//! Builds semantic graphs (CFG + DFG) on top of Phase 1's parse trees.
//!
//! ## Core Principle
//!
//! > **Parallel compute is allowed. Semantic commits are not.**
//!
//! All semantic graph construction happens in deterministic order.

pub mod model;
pub mod epoch;
pub mod cfg;
pub mod dfg;
pub mod symbols;
pub mod invalidation;

// Re-export public API
pub use model::{
    CFG, CFGEdge, CFGEdgeKind, CFGNode, CFGNodeKind,
    DFG, DFGEdge, DFGEdgeKind, DFGValue, ValueKind,
    FunctionId, NodeId, ValueId, EdgeId, SymbolId, ScopeId,
};

pub use epoch::SemanticEpoch;
pub use cfg::CFGBuilder;
pub use dfg::DFGBuilder;
pub use symbols::SymbolTable;
pub use invalidation::InvalidationTracker;
