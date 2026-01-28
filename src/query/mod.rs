//! Query engine module (Step 3.6)
//!
//! Contains deterministic query execution primitives

pub mod engine;
pub mod primitives;

pub use engine::{QueryEngine, QueryResult};
pub use primitives::QueryPrimitives;
