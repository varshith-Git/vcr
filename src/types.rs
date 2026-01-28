//! Core type definitions for the deterministic kernel.
//!
//! All types are designed for:
//! - Deterministic serialization
//! - No path leakage (use FileId instead)
//! - Immutable snapshots

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Opaque file identifier. Never exposes the underlying path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileId(u64);

impl FileId {
    /// Create a new FileId from a hash.
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }

    /// Get the raw ID value (for internal use only).
    pub(crate) fn as_u64(&self) -> u64 {
        self.0
    }
}

/// A complete snapshot of a repository at a specific point in time.
///
/// Snapshots are:
/// - Deterministic: same repo state → same snapshot
/// - Immutable: never modified after creation
/// - Serializable: can be persisted and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    /// Root directory of the repository
    pub root: PathBuf,
    
    /// Map from FileId to file metadata
    pub files: HashMap<FileId, FileMetadata>,
    
    /// When this snapshot was created
    pub created_at: SystemTime,
    
    /// SHA256 hash of the entire snapshot (for verification)
    pub snapshot_hash: String,
}

impl RepoSnapshot {
    /// Get all file IDs in deterministic order.
    pub fn file_ids(&self) -> Vec<FileId> {
        let mut ids: Vec<_> = self.files.keys().copied().collect();
        ids.sort();
        ids
    }
}

/// Metadata for a single file in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Normalized relative path from repo root
    pub path: PathBuf,
    
    /// File size in bytes
    pub size: u64,
    
    /// Last modified time
    pub mtime: SystemTime,
    
    /// SHA256 hash of file contents (for change detection)
    pub content_hash: String,
    
    /// Detected language (for parser selection)
    pub language: Option<Language>,
}

/// Supported languages for parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust
    Rust,
    // More languages will be added in later phases
}

impl Language {
    /// Get file extension associated with this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
        }
    }

    /// Detect language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            _ => None,
        }
    }
}

/// A parsed file with Tree-sitter.
#[derive(Debug)]
pub struct ParsedFile {
    /// File identifier
    pub file_id: FileId,
    
    /// Tree-sitter parse tree
    pub tree: tree_sitter::Tree,
    
    /// Byte ranges that were parsed
    pub byte_ranges: Vec<ByteRange>,
    
    /// Parse time in microseconds
    pub parse_time_us: u64,
}

/// A byte range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    /// Start byte offset (inclusive)
    pub start: usize,
    
    /// End byte offset (exclusive)
    pub end: usize,
}

impl ByteRange {
    /// Create a new byte range.
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "Invalid byte range");
        Self { start, end }
    }

    /// Get the length of this range.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this range is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Epoch marker for type-safe epoch tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpochMarker(u64);

impl EpochMarker {
    /// Create a new epoch marker.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the next epoch marker.
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}
