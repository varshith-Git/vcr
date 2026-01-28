//! Deterministic repository scanner (Step 1.1)
//!
//! Walks directories in stable order, filters files deterministically,
//! produces reproducible RepoSnapshot.

use crate::types::{FileId, FileMetadata, Language, RepoSnapshot};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Deterministic repository scanner.
///
/// Scans a directory tree and produces a reproducible snapshot.
///
/// # Determinism Guarantees
///
/// - Files are always discovered in lexicographic order
/// - Paths are normalized (canonical, stable separators)
/// - Content hashes ensure change detection
/// - Same repo state → identical snapshot every time
pub struct RepoScanner {
    /// Root directory to scan
    root: PathBuf,
    
    /// File extensions to include (e.g., "rs" for Rust)
    extensions: HashSet<String>,
    
    /// Whether to follow symlinks (default: false for determinism)
    follow_symlinks: bool,
}

impl RepoScanner {
    /// Create a new scanner for the given repository root.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().canonicalize()
            .context("Failed to canonicalize repository root")?;
        
        Ok(Self {
            root,
            extensions: HashSet::new(),
            follow_symlinks: false,
        })
    }

    /// Add a file extension to scan (e.g., "rs", "py", "js").
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.insert(ext.into());
        self
    }

    /// Add multiple extensions at once.
    pub fn with_extensions(mut self, exts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extensions.extend(exts.into_iter().map(Into::into));
        self
    }

    /// Set whether to follow symlinks.
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Scan the repository and produce a deterministic snapshot.
    ///
    /// # Determinism
    ///
    /// - Directory traversal is ordered lexicographically
    /// - File filtering is deterministic
    /// - Hash computation is stable
    pub fn scan(&self) -> Result<RepoSnapshot> {
        let mut files_map = HashMap::new();
        let mut all_paths = Vec::new();

        // Step 1: Collect all file paths
        for entry in WalkDir::new(&self.root)
            .follow_links(self.follow_symlinks)
            .sort_by_file_name() // Lexicographic ordering
        {
            let entry = entry.context("Failed to read directory entry")?;
            
            // Skip directories
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            
            // Filter by extension if specified
            if !self.extensions.is_empty() {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                if !self.extensions.contains(ext) {
                    continue;
                }
            }

            all_paths.push(path.to_path_buf());
        }

        // Step 2: Sort paths for determinism (walkdir sorts per-directory, we want global order)
        all_paths.sort();

        // Step 3: Process each file deterministically
        for path in all_paths {
            let metadata = self.process_file(&path)?;
            let file_id = Self::compute_file_id(&metadata.path);
            files_map.insert(file_id, metadata);
        }

        // Step 4: Compute snapshot hash
        let snapshot_hash = Self::compute_snapshot_hash(&files_map);

        Ok(RepoSnapshot {
            root: self.root.clone(),
            files: files_map,
            created_at: SystemTime::now(),
            snapshot_hash,
        })
    }

    /// Process a single file and extract metadata.
    fn process_file(&self, path: &Path) -> Result<FileMetadata> {
        // Read file contents for hashing
        let contents = fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Hash contents
        let content_hash = Self::hash_bytes(&contents);

        // Get file metadata
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        // Normalize path relative to root
        let relative_path = path.strip_prefix(&self.root)
            .context("Failed to compute relative path")?
            .to_path_buf();

        // Detect language
        let language = path.extension()
            .and_then(|e| e.to_str())
            .and_then(Language::from_extension);

        Ok(FileMetadata {
            path: relative_path,
            size: metadata.len(),
            mtime: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            content_hash,
            language,
        })
    }

    /// Compute a deterministic FileId from a path.
    fn compute_file_id(path: &Path) -> FileId {
        let path_str = path.to_string_lossy();
        let hash = Self::hash_string(&path_str);
        
        // Use first 8 bytes of SHA256 as FileId
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        FileId::new(u64::from_be_bytes(bytes))
    }

    /// Hash bytes with SHA256.
    fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Hash a string with SHA256.
    fn hash_string(s: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Compute overall snapshot hash for verification.
    fn compute_snapshot_hash(files: &HashMap<FileId, FileMetadata>) -> String {
        let mut hasher = Sha256::new();

        // Sort file IDs for determinism
        let mut file_ids: Vec<_> = files.keys().collect();
        file_ids.sort();

        // Hash each file's metadata in order
        for file_id in file_ids {
            let metadata = &files[file_id];
            hasher.update(file_id.as_u64().to_be_bytes());
            hasher.update(metadata.path.to_string_lossy().as_bytes());
            hasher.update(&metadata.size.to_be_bytes());
            hasher.update(metadata.content_hash.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = RepoScanner::new(temp_dir.path()).unwrap();
        let snapshot = scanner.scan().unwrap();
        
        assert_eq!(snapshot.files.len(), 0);
    }

    #[test]
    fn test_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let scanner = RepoScanner::new(temp_dir.path())
            .unwrap()
            .with_extension("rs");
        
        let snapshot = scanner.scan().unwrap();
        assert_eq!(snapshot.files.len(), 1);
    }

    #[test]
    fn test_determinism() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple files
        fs::write(temp_dir.path().join("a.rs"), "// A").unwrap();
        fs::write(temp_dir.path().join("b.rs"), "// B").unwrap();
        fs::write(temp_dir.path().join("c.rs"), "// C").unwrap();

        let scanner = RepoScanner::new(temp_dir.path())
            .unwrap()
            .with_extension("rs");

        // Scan twice
        let snapshot1 = scanner.scan().unwrap();
        let snapshot2 = scanner.scan().unwrap();

        // Should produce identical snapshots
        assert_eq!(snapshot1.snapshot_hash, snapshot2.snapshot_hash);
        assert_eq!(snapshot1.files.len(), snapshot2.files.len());
    }

    #[test]
    fn test_extension_filtering() {
        let temp_dir = TempDir::new().unwrap();
        
        fs::write(temp_dir.path().join("code.rs"), "// Rust").unwrap();
        fs::write(temp_dir.path().join("data.txt"), "data").unwrap();

        let scanner = RepoScanner::new(temp_dir.path())
            .unwrap()
            .with_extension("rs");

        let snapshot = scanner.scan().unwrap();
        
        // Should only find the .rs file
        assert_eq!(snapshot.files.len(), 1);
        
        let file = snapshot.files.values().next().unwrap();
        assert_eq!(file.language, Some(Language::Rust));
    }
}
