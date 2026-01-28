//! Change detection (Step 1.5)
//!
//! Detects what changed between repository snapshots.

use crate::types::{FileId, RepoSnapshot};

/// Type of file change detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    /// File was added
    Added(FileId),
    
    /// File was modified (content hash changed)
    Modified(FileId),
    
    /// File was deleted
    Deleted(FileId),
    
    /// File unchanged
    Unchanged(FileId),
}

/// Change detector between snapshots.
pub struct ChangeDetector {
    previous_snapshot: RepoSnapshot,
}

impl ChangeDetector {
    /// Create a new change detector with a previous snapshot.
    pub fn new(previous_snapshot: RepoSnapshot) -> Self {
        Self { previous_snapshot }
    }

    /// Detect changes between the previous and current snapshot.
    pub fn detect(&self, current: &RepoSnapshot) -> Vec<FileChange> {
        let mut changes = Vec::new();

        // Check for added and modified files
        for (file_id, current_meta) in &current.files {
            match self.previous_snapshot.files.get(file_id) {
                None => {
                    // File is new
                    changes.push(FileChange::Added(*file_id));
                }
                Some(prev_meta) => {
                    // File exists - check if content changed
                    if prev_meta.content_hash != current_meta.content_hash {
                        changes.push(FileChange::Modified(*file_id));
                    } else {
                        changes.push(FileChange::Unchanged(*file_id));
                    }
                }
            }
        }

        // Check for deleted files
        for file_id in self.previous_snapshot.files.keys() {
            if !current.files.contains_key(file_id) {
                changes.push(FileChange::Deleted(*file_id));
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileMetadata, Language};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn make_snapshot(files: Vec<(u64, &str, &str)>) -> RepoSnapshot {
        let mut file_map = HashMap::new();
        
        for (id, path, hash) in files {
            file_map.insert(
                FileId::new(id),
                FileMetadata {
                    path: PathBuf::from(path),
                    size: 0,
                    mtime: SystemTime::UNIX_EPOCH,
                    content_hash: hash.to_string(),
                    language: Some(Language::Rust),
                },
            );
        }

        RepoSnapshot {
            root: PathBuf::from("/test"),
            files: file_map,
            created_at: SystemTime::UNIX_EPOCH,
            snapshot_hash: "test".to_string(),
        }
    }

    #[test]
    fn test_no_changes() {
        let prev = make_snapshot(vec![(1, "a.rs", "hash1")]);
        let curr = make_snapshot(vec![(1, "a.rs", "hash1")]);

        let detector = ChangeDetector::new(prev);
        let changes = detector.detect(&curr);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], FileChange::Unchanged(_)));
    }

    #[test]
    fn test_added_file() {
        let prev = make_snapshot(vec![]);
        let curr = make_snapshot(vec![(1, "a.rs", "hash1")]);

        let detector = ChangeDetector::new(prev);
        let changes = detector.detect(&curr);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], FileChange::Added(_)));
    }

    #[test]
    fn test_modified_file() {
        let prev = make_snapshot(vec![(1, "a.rs", "hash1")]);
        let curr = make_snapshot(vec![(1, "a.rs", "hash2")]);

        let detector = ChangeDetector::new(prev);
        let changes = detector.detect(&curr);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], FileChange::Modified(_)));
    }

    #[test]
    fn test_deleted_file() {
        let prev = make_snapshot(vec![(1, "a.rs", "hash1")]);
        let curr = make_snapshot(vec![]);

        let detector = ChangeDetector::new(prev);
        let changes = detector.detect(&curr);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], FileChange::Deleted(_)));
    }
}
