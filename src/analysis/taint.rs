//! Taint propagation analysis (Step 3.5) - STUB
//!
//! Will be implemented in Step 3.5

use crate::cpg::model::CPGNodeId;

/// Taint analysis stub
pub struct TaintAnalysis;

/// Taint path stub
pub struct TaintPath {
    pub source: CPGNodeId,
    pub sink: CPGNodeId,
}
