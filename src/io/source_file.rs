//! I/O source file abstraction (Step 1.3)
//!
//! Memory-mapped file reading with opaque FileId.

use crate::types::FileId;
use anyhow::{Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// Trait for reading source files.
pub trait SourceFile {
    /// Get the raw bytes of the file.
    fn bytes(&self) -> &[u8];
    
    /// Get the file identifier.
    fn file_id(&self) -> FileId;
    
    /// Get file size in bytes.
    fn size(&self) -> usize {
        self.bytes().len()
    }
}

/// Memory-mapped file implementation.
pub struct MmappedFile {
    file_id: FileId,
    mmap: Mmap,
}

impl MmappedFile {
    /// Open and memory-map a file.
    pub fn open<P: AsRef<Path>>(path: P, file_id: FileId) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;
        
        // Safety: File is opened read-only and we don't modify it
        let mmap = unsafe {
            Mmap::map(&file)
                .context("Failed to memory-map file")?
        };
        
        Ok(Self { file_id, mmap })
   }
}

impl SourceFile for MmappedFile {
    fn bytes(&self) -> &[u8] {
        &self.mmap
    }
    
    fn file_id(&self) -> FileId {
        self.file_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mmap_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = b"Hello, world!";
        fs::write(temp_file.path(), content).unwrap();

        let file_id = FileId::new(42);
        let mmapped = MmappedFile::open(temp_file.path(), file_id).unwrap();

        assert_eq!(mmapped.bytes(), content);
        assert_eq!(mmapped.file_id(), file_id);
        assert_eq!(mmapped.size(), content.len());
    }
}
