//! SIMD acceleration module (Phase 4 Step 4.2)
//!
//! **Leaf operations only - no graph traversal**
//!
//! Allowed SIMD:
//! - Node/edge kind filtering
//! - Trigram matching
//! - Set intersections
//!
//! Forbidden SIMD:
//! - Graph traversal
//! - Pointer chasing
//! - Branch-heavy code

pub mod filters;

pub use filters::{filter_by_kind, filter_by_kind_scalar};

/// Check if SIMD is available at runtime
#[cfg(target_arch = "x86_64")]
pub fn simd_available() -> bool {
    is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
pub fn simd_available() -> bool {
    false
}
