//! Epoch-based memory management (Step 1.2)
//!
//! Each epoch owns its memory. When an epoch ends, all memory dies together.

use crate::io::{MmappedFile, SourceFile};
use crate::types::{EpochMarker, FileId};
use std::collections::HashMap;
use std::sync::Arc;

/// Ingestion epoch - owns file discovery and I/O.
pub struct IngestionEpoch {
    marker: EpochMarker,
    mmaps: HashMap<FileId, Arc<MmappedFile>>,
}

impl IngestionEpoch {
    /// Create a new ingestion epoch.
    pub fn new(marker: EpochMarker) -> Self {
        Self {
            marker,
            mmaps: HashMap::new(),
        }
    }

    /// Add a memory-mapped file to this epoch.
    pub fn add_file(&mut self, file: MmappedFile) -> FileId {
        let file_id = file.file_id();
        self.mmaps.insert(file_id, Arc::new(file));
        file_id
    }

    /// Get a file from this epoch.
    pub fn get_file(&self, file_id: FileId) -> Option<Arc<MmappedFile>> {
        self.mmaps.get(&file_id).cloned()
    }

    /// Get the epoch marker.
    pub fn marker(&self) -> EpochMarker {
        self.marker
    }
}

/// Parse epoch - owns parse trees and buffers.
pub struct ParseEpoch {
    marker: EpochMarker,
    ingestion: Arc<IngestionEpoch>,
    // Parse trees will be stored here (Step 1.4)
}

impl ParseEpoch {
    /// Create a new parse epoch.
    pub fn new(marker: EpochMarker, ingestion: Arc<IngestionEpoch>) -> Self {
        Self {
            marker,
            ingestion,
        }
    }

    /// Get the epoch marker.
    pub fn marker(&self) -> EpochMarker {
        self.marker
    }

    /// Get access to the ingestion epoch.
    pub fn ingestion(&self) -> &IngestionEpoch {
        &self.ingestion
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_epoch_lifecycle() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test").unwrap();

        let mut ingestion = IngestionEpoch::new(EpochMarker::new(1));
        let file_id = FileId::new(42);
        let mmap = MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        ingestion.add_file(mmap);
        
        assert!(ingestion.get_file(file_id).is_some());
    }
}
