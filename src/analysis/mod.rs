//! Analysis passes module (Step 3.4+)
//!
//! Contains bounded, explainable analysis passes:
//! - Pointer/alias analysis (Step 3.4)
//! - Taint propagation (Step 3.5)
//! - Reachability queries (Step 3.6)

pub mod pointer;
pub mod taint;
pub mod reachability;

pub use pointer::{PointerAnalysis, PointsToSet};
pub use taint::{TaintAnalysis, TaintPath};
