//! Dependency invalidation tracker (Step 2.5)

use crate::semantic::model::{EdgeId, NodeId};
use crate::types::ByteRange;
use std::collections::HashMap;

/// Tracks dependencies for incremental updates
pub struct InvalidationTracker {
    /// AST byte range → CFG nodes
    _ast_to_cfg: HashMap<ByteRange, Vec<NodeId>>,
    
    /// CFG node → DFG edges
    _cfg_to_dfg: HashMap<NodeId, Vec<EdgeId>>,
}

impl InvalidationTracker {
    /// Create a new invalidation tracker
    pub fn _new() -> Self {
        Self {
            _ast_to_cfg: HashMap::new(),
            _cfg_to_dfg: HashMap::new(),
        }
    }
}
