//! Code Property Graph - Unified graph model (Step 3.1)
//!
//! **SCHEMA FROZEN**
//!
//! This module defines the unified CPG schema that fuses AST, CFG, and DFG
//! into a single queryable graph. The schema is frozen and immutable.
//!
//! ## Design Principles
//!
//! - All nodes stored in Vec (deterministic order)
//! - All edges stored in Vec (deterministic order)
//! - Sequential, never-reused IDs
//! - Every node has origin reference back to source

pub mod epoch;
pub mod model;
pub mod builder;
pub mod index;
pub mod hash;

pub use model::{CPGNode, CPGEdge, CPGNodeKind, CPGEdgeKind, CPGNodeId, CPGEdgeId};
pub use epoch::CPGEpoch;
