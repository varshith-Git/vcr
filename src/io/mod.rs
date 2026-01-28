//! I/O abstraction layer (Path B1)
//!
//! **Split**: Hot path (mmap) vs Cold path (async bulk)
//!
//! Hot path: Incremental edits, queries (unchanged from Phase 1)
//! Cold path: Large repo ingestion (new, optional acceleration)

// Existing Phase 1 I/O (unchanged)
pub mod source_file;

// Path B1: New I/O abstraction
pub mod hot;
pub mod cold;

// Phase 1 exports (unchanged)
pub use source_file::{MmappedFile, SourceFile};

use std::path::Path;
use std::io::Result;

/// I/O mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOMode {
    /// Hot path - mmap + page cache (incremental edits)
    Hot,
    
    /// Cold path - async bulk reads (large ingestion)
    Cold,
    
    /// Auto-detect based on operation
    Auto,
}

/// I/O backend abstraction
pub trait IOBackend: Send + Sync {
    /// Read file contents
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Backend name (for diagnostics)
    fn name(&self) -> &'static str;
}

/// Create I/O backend for given mode
pub fn create_backend(mode: IOMode) -> Box<dyn IOBackend> {
    match mode {
        IOMode::Hot => Box::new(hot::HotPathIO::new()),
        IOMode::Cold => cold::create_cold_backend(),
        IOMode::Auto => Box::new(hot::HotPathIO::new()), // Default to hot for now
    }
}
