//! Parallel execution module (Phase 4 Step 4.1)
//!
//! **Model: Parallel Compute, Serial Commit**
//!
//! Queries split into independent fragments
//! → Fragments execute in parallel
//! → Results merged in fixed, deterministic order
//!
//! ## Rules
//! - No shared mutable state
//! - No parallel graph mutation
//! - All commits on one thread, one order

pub mod plan;
pub mod scheduler;
pub mod task;

pub use plan::{ExecutionPlan, Stage, DeterministicOrder};
pub use task::{Task, TaskId, WorkFragment};
pub use scheduler::Scheduler;
