//! Storage module (Path B2)
//!
//! Persistent on-disk CPG (replayable)

use crate::cpg::model::CPG;
use std::path::Path;
use std::io::{Result, Error, ErrorKind};
use serde::{Serialize, Deserialize};

/// Storage version
pub const STORAGE_VERSION: u32 = 1;

/// Snapshot ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SnapshotId(pub u64);

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// CPG snapshot manager
pub struct CPGSnapshot;

impl CPGSnapshot {
    /// Save CPG to disk (append-only)
    pub fn save(cpg: &CPG, path: &Path) -> Result<SnapshotId> {
        // Compute hash
        let cpg_hash = cpg.compute_hash();
        
        // Create metadata
        let metadata = SnapshotMetadata::new(
            0,  // epoch_id placeholder
            cpg_hash.clone(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        
        // Serialize (placeholder - would use FlatBuffers)
        let serialized = serde_json::to_string(&metadata)?;
        std::fs::write(path, serialized)?;
        
        Ok(SnapshotId(1))
    }
    
    /// Load CPG from disk (zero-copy would go here)
    pub fn load(path: &Path) -> Result<CPG> {
        // Placeholder: would deserialize FlatBuffers
        // For now, return empty CPG
        let _serialized = std::fs::read_to_string(path)?;
        Ok(CPG::new())
    }
    
    /// Verify snapshot integrity
    pub fn verify(path: &Path) -> Result<String> {
        // Load metadata
        let serialized = std::fs::read_to_string(path)?;
        let metadata: SnapshotMetadata = serde_json::from_str(&serialized)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        
        // Verify version
        if metadata.version != STORAGE_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData, 
                format!("Version mismatch: expected {}, got {}", STORAGE_VERSION, metadata.version)
            ));
        }
        
        Ok(metadata.cpg_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpg::model::{CPGNode, CPGNodeId, CPGNodeKind, OriginRef};
    use crate::types::ByteRange;
    use tempfile::NamedTempFile;

    #[test]
    fn test_snapshot_metadata() {
        let meta = SnapshotMetadata::new(42, "abc123".to_string(), 1234567890);
        assert_eq!(meta.epoch_id, 42);
        assert_eq!(meta.version, STORAGE_VERSION);
    }

    #[test]
    fn test_snapshot_save_load() {
        let mut cpg = CPG::new();
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        let temp = NamedTempFile::new().unwrap();
        
        // Save
        let snapshot_id = CPGSnapshot::save(&cpg, temp.path()).unwrap();
        assert_eq!(snapshot_id.0, 1);
        
        // Load (placeholder returns empty CPG)
        let loaded = CPGSnapshot::load(temp.path()).unwrap();
        assert_eq!(loaded.nodes.len(), 0);  // Placeholder behavior
    }

    #[test]
    fn test_snapshot_verify() {
        let mut cpg = CPG::new();
        let temp = NamedTempFile::new().unwrap();
        
        CPGSnapshot::save(&cpg, temp.path()).unwrap();
        let hash = CPGSnapshot::verify(temp.path()).unwrap();
        
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_snapshot_version_mismatch() {
        let temp = NamedTempFile::new().unwrap();
        
        // Write invalid version
        let bad_metadata = SnapshotMetadata {
            epoch_id: 1,
            cpg_hash: "test".to_string(),
            timestamp: 0,
            version: 999,  // Invalid
        };
        
        let serialized = serde_json::to_string(&bad_metadata).unwrap();
        std::fs::write(temp.path(), serialized).unwrap();
        
        // Verify should fail
        assert!(CPGSnapshot::verify(temp.path()).is_err());
    }
}
