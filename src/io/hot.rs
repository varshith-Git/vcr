//! Hot-path I/O (unchanged from Phase 1)
//!
//! mmap + page cache for incremental operations

use super::IOBackend;
use std::fs;
use std::io::Result;
use std::path::Path;

/// Hot-path I/O backend (unchanged)
pub struct HotPathIO;

impl HotPathIO {
    pub fn new() -> Self {
        Self
    }
}

impl IOBackend for HotPathIO {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        // Simple synchronous read (existing behavior)
        fs::read(path)
    }
    
    fn name(&self) -> &'static str {
        "hot-mmap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hot_path_read() {
        let temp = NamedTempFile::new().unwrap();
        let content = b"test data";
        fs::write(temp.path(), content).unwrap();

        let backend = HotPathIO::new();
        let result = backend.read_file(temp.path()).unwrap();
        
        assert_eq!(result, content);
    }
}
