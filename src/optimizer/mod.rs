//! Query optimizer module (Phase 4 Step 4.3)
//!
//! **Cost-based query optimization**
//! Reorder queries, never reinterpret

pub mod cost;
pub mod planner;

pub use cost::QueryCost;
pub use planner::QueryPlanner;
