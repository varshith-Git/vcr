//! Epoch-based memory management (Step 1.2)

pub mod epoch;
pub mod arena;

pub use epoch::{IngestionEpoch, ParseEpoch};
