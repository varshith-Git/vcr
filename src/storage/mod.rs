//! Storage module (Phase 4 Step 4.4)
//!
//! Persistent on-disk CPG (replayable)

/// Storage version
pub const STORAGE_VERSION: u32 = 1;

/// Snapshot metadata
#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    pub epoch_id: u64,
    pub cpg_hash: String,
    pub timestamp: u64,
    pub version: u32,
}

impl SnapshotMetadata {
    pub fn new(epoch_id: u64, cpg_hash: String, timestamp: u64) -> Self {
        Self {
            epoch_id,
            cpg_hash,
            timestamp,
            version: STORAGE_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_metadata() {
        let meta = SnapshotMetadata::new(42, "abc123".to_string(), 1234567890);
        assert_eq!(meta.epoch_id, 42);
        assert_eq!(meta.version, STORAGE_VERSION);
    }
}
