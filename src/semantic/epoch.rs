//! SemanticEpoch - owns all semantic memory (Step 2.6)

use crate::memory::ParseEpoch;
use crate::semantic::model::{CFG, DFG, FunctionId};
use crate::types::EpochMarker;
use std::collections::HashMap;
use std::sync::Arc;

/// Semantic epoch owns all semantic graphs
pub struct SemanticEpoch {
    _marker: EpochMarker,
    _parse_epoch: Arc<ParseEpoch>,
    _cfgs: HashMap<FunctionId, CFG>,
    _dfgs: HashMap<FunctionId, DFG>,
}

impl SemanticEpoch {
    /// Create a new semantic epoch
    pub fn _new(_marker: EpochMarker, _parse_epoch: Arc<ParseEpoch>) -> Self {
        Self {
            _marker,
            _parse_epoch,
            _cfgs: HashMap::new(),
            _dfgs: HashMap::new(),
        }
    }
}
