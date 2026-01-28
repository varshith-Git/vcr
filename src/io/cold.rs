//! Cold-path I/O with feature-flagged io_uring
//!
//! **Feature**: `cold-path-uring` (Linux-only)
//! **Fallback**: Sync I/O (always available)

use super::IOBackend;
use std::fs;
use std::io::Result;
use std::path::Path;

/// Sync I/O backend (fallback, always available)
pub struct SyncIOBackend;

impl SyncIOBackend {
    pub fn new() -> Self {
        Self
    }
}

impl IOBackend for SyncIOBackend {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path)
    }
    
    fn name(&self) -> &'static str {
        "cold-sync"
    }
}

/// io_uring backend (feature-flagged, Linux-only)
#[cfg(all(target_os = "linux", feature = "cold-path-uring"))]
pub struct UringBackend {
    // Placeholder for io_uring ring
    // Would include: IoUring instance, SQPOLL mode, etc.
}

#[cfg(all(target_os = "linux", feature = "cold-path-uring"))]
impl UringBackend {
    pub fn new() -> Result<Self> {
        // Placeholder: would initialize io_uring
        // SQPOLL only, no IOPOLL
        // Page cache ON
        Ok(Self {})
    }
}

#[cfg(all(target_os = "linux", feature = "cold-path-uring"))]
impl IOBackend for UringBackend {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        // Placeholder: would use io_uring for async read
        // For now, delegate to sync (correct baseline)
        fs::read(path)
    }
    
    fn name(&self) -> &'static str {
        "cold-uring"
    }
}

/// Create cold-path backend with feature detection
pub fn create_cold_backend() -> Box<dyn IOBackend> {
    #[cfg(all(target_os = "linux", feature = "cold-path-uring"))]
    {
        if let Ok(backend) = UringBackend::new() {
            return Box::new(backend);
        }
    }
    
    // Fallback to sync I/O
    Box::new(SyncIOBackend::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sync_backend() {
        let temp = NamedTempFile::new().unwrap();
        let content = b"sync test";
        fs::write(temp.path(), content).unwrap();

        let backend = SyncIOBackend::new();
        let result = backend.read_file(temp.path()).unwrap();
        
        assert_eq!(result, content);
    }

    #[test]
    fn test_cold_backend_creation() {
        // Should always succeed (fallback to sync)
        let backend = create_cold_backend();
        assert!(!backend.name().is_empty());
    }

    #[test]
    fn test_backend_determinism() {
        // Same file read with different backends → identical result
        let temp = NamedTempFile::new().unwrap();
        let content = b"determinism test";
        fs::write(temp.path(), content).unwrap();

        let sync_backend = SyncIOBackend::new();
        let cold_backend = create_cold_backend();

        let result1 = sync_backend.read_file(temp.path()).unwrap();
        let result2 = cold_backend.read_file(temp.path()).unwrap();

        assert_eq!(result1, result2, "Backends must produce identical output");
    }
}
