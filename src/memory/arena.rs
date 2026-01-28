//! Simple arena allocator (Step 1.2)
//!
//! Placeholder for arena allocation within epochs.
//! For now, we'll use standard allocation. Can be enhanced later with bumpalo.

/// Placeholder arena allocator.
///
/// In Phase 1, we use standard allocation.
/// Future enhancement: use bumpalo or custom bump allocator.
pub struct Arena {
    // Future: bump allocator state
}

impl Arena {
    /// Create a new arena.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
