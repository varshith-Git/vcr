//! Valori Deterministic Ingestion Kernel - Phase 1
//! A deterministic file ingestion engine with incremental Tree-sitter parsing.
//!
//! ## Design Principles
//!
//! 1. **Determinism is sacred** - Same input always produces same output
//! 2. **Epoch-based memory** - Clear ownership, automatic cleanup
//! 3. **Incremental everything** - Only reparse what changed
//! 4. **Fail closed** - Crash on divergence, not silent corruption
//!
//! ## Phase 1 Scope
//!
//! This phase is:
//! - Fast, deterministic file ingestion
//! - Incremental parsing with Tree-sitter
//! - Clear memory ownership and lifetimes
//! - Minimal, immutable internal representation
//!
//! This phase is NOT:
//! - A full code analyzer
//! - A security engine
//! - A graph query system
//! - Using SIMD, io_uring, or parallelism

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod change;
pub mod io;
pub mod memory;
pub mod metrics;
pub mod parse;
pub mod repo;
pub mod semantic;  // Phase 2
pub mod cpg;  // Phase 3
pub mod analysis;  // Phase 3
pub mod query;  // Phase 3
pub mod execution;  // Phase 4
pub mod types;

// Re-export public API
pub use types::{FileId, ParsedFile, RepoSnapshot};
pub use repo::RepoScanner;
pub use parse::IncrementalParser;
pub use change::{ChangeDetector, FileChange};
pub use metrics::MetricsCollector;

// Phase 2 exports
pub use semantic::{
    CFG, DFG, SemanticEpoch, CFGBuilder, DFGBuilder,
    FunctionId, NodeId, ValueId,
};
