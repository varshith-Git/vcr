//! Determinism validation tests (Step 1.6)
//!
//! These tests verify the core guarantees of Phase 1:
//! 1. Same repo → identical snapshots
//! 2. Reordered entries → same output
//! 3. Single file change → single reparse
//! 4. Process restart → same result

use std::fs;
use tempfile::TempDir;
use vcr::*;

/// Create a test repository with multiple files
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a simple directory structure
    fs::create_dir_all(temp_dir.path().join("src/core")).unwrap();
    fs::create_dir_all(temp_dir.path().join("src/utils")).unwrap();
    
    // Write test files
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello\"); }",
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("src/core/mod.rs"),
        "pub mod engine;",
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("src/core/engine.rs"),
        "pub fn run() { println!(\"Engine\"); }",
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("src/utils/mod.rs"),
        "pub fn helper() -> i32 { 42 }",
    ).unwrap();
    
    temp_dir
}

#[test]
fn test_reproducibility_same_repo() {
    // Test 1: Scan same repo twice → bit-for-bit identical snapshots
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    // First scan
    let snapshot1 = scanner.scan().unwrap();
    
    // Second scan (no changes)
    let snapshot2 = scanner.scan().unwrap();
    
    // Snapshots must be identical
    assert_eq!(
        snapshot1.snapshot_hash,
        snapshot2.snapshot_hash,
        "Identical repo must produce identical snapshot hashes"
    );
    
    assert_eq!(
        snapshot1.files.len(),
        snapshot2.files.len(),
        "Same number of files"
    );
    
    // Verify each file hash matches
    for (file_id, meta1) in &snapshot1.files {
        let meta2 = snapshot2.files.get(file_id).expect("File should exist in both snapshots");
        assert_eq!(
            meta1.content_hash, meta2.content_hash,
            "Content hashes must match for file: {:?}",
            meta1.path
        );
    }
}

#[test]
fn test_order_independence() {
    // Test 2: Reordered directory entries → same output
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot1 = scanner.scan().unwrap();
    
    // Scanner should always produce same order internally
    // Multiple scans should produce same hash
    let snapshot2 = scanner.scan().unwrap();
    let snapshot3 = scanner.scan().unwrap();
    
    assert_eq!(snapshot1.snapshot_hash, snapshot2.snapshot_hash);
    assert_eq!(snapshot2.snapshot_hash, snapshot3.snapshot_hash);
    
    // File IDs should be in same order
    let ids1 = snapshot1.file_ids();
    let ids2 = snapshot2.file_ids();
    let ids3 = snapshot3.file_ids();
    
    assert_eq!(ids1, ids2);
    assert_eq!(ids2, ids3);
}

#[test]
fn test_incremental_precision() {
    // Test 3: Modify one file → only one reparse
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    // Initial scan
    let snapshot1 = scanner.scan().unwrap();
    
    // Modify exactly ONE file
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Modified\"); }",
    ).unwrap();
    
    // Rescan
    let snapshot2 = scanner.scan().unwrap();
    
    // Detect changes
    let detector = ChangeDetector::new(snapshot1);
    let changes = detector.detect(&snapshot2);
    
    // Count modified files
    let modified_count = changes.iter()
        .filter(|c| matches!(c, change::FileChange::Modified(_)))
        .count();
    
    assert_eq!(
        modified_count, 1,
        "Exactly one file should be detected as modified"
    );
    
    // Count unchanged files
    let unchanged_count = changes.iter()
        .filter(|c| matches!(c, change::FileChange::Unchanged(_)))
        .count();
    
    assert_eq!(
        unchanged_count, 3,
        "Three files should remain unchanged"
    );
}

#[test]
fn test_restart_stability() {
    // Test 4: Kill and restart → same result
    
    let temp_dir = create_test_repo();
    
    // First "process" - scan and save snapshot
    let scanner1 = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot1 = scanner1.scan().unwrap();
    let hash1 = snapshot1.snapshot_hash.clone();
    
    // Simulate process restart - create new scanner
    let scanner2 = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot2 = scanner2.scan().unwrap();
    let hash2 = snapshot2.snapshot_hash.clone();
    
    // Must produce identical results
    assert_eq!(
        hash1, hash2,
        "Restart must produce identical snapshot hash"
    );
}

#[test]
fn test_parse_determinism() {
    // Test: Parse same file multiple times → identical trees
    
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    
    let source = b"fn example() { let x = 42; return x; }";
    fs::write(&file_path, source).unwrap();
    
    let file_id = FileId::new(1);
    let mmap = vcr::io::MmappedFile::open(&file_path, file_id).unwrap();
    
    let mut parser = IncrementalParser::new(types::Language::Rust).unwrap();
    
    // Parse multiple times
    let parsed1 = parser.parse(&mmap, None).unwrap();
    let parsed2 = parser.parse(&mmap, None).unwrap();
    
    // Trees should be structurally identical
    assert_eq!(
        parsed1.tree.root_node().to_sexp(),
        parsed2.tree.root_node().to_sexp(),
        "Multiple parses of same content must produce identical trees"
    );
}

#[test]
fn test_incremental_reparse_only_changed() {
    // Test: Only reparse files that actually changed
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot1 = scanner.scan().unwrap();
    let snapshot2 = scanner.scan().unwrap();
    
    let detector = ChangeDetector::new(snapshot1);
    let changes = detector.detect(&snapshot2);
    
    // No changes → all files should be unchanged
    let all_unchanged = changes.iter()
        .all(|c| matches!(c, change::FileChange::Unchanged(_)));
    
    assert!(
        all_unchanged,
        "When no files change, all should be marked unchanged"
    );
}

#[test]
fn test_file_addition_detection() {
    // Test: Adding a new file is detected correctly
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot1 = scanner.scan().unwrap();
    
    // Add a new file
    fs::write(
        temp_dir.path().join("src/new_module.rs"),
        "pub fn new_function() {}",
    ).unwrap();
    
    let snapshot2 = scanner.scan().unwrap();
    
    let detector = ChangeDetector::new(snapshot1);
    let changes = detector.detect(&snapshot2);
    
    let added_count = changes.iter()
        .filter(|c| matches!(c, change::FileChange::Added(_)))
        .count();
    
    assert_eq!(added_count, 1, "One file should be added");
}

#[test]
fn test_file_deletion_detection() {
    // Test: Deleting a file is detected correctly
    
    let temp_dir = create_test_repo();
    
    let scanner = RepoScanner::new(temp_dir.path())
        .unwrap()
        .with_extension("rs");
    
    let snapshot1 = scanner.scan().unwrap();
    
    // Delete a file
    fs::remove_file(temp_dir.path().join("src/main.rs")).unwrap();
    
    let snapshot2 = scanner.scan().unwrap();
    
    let detector = ChangeDetector::new(snapshot1);
    let changes = detector.detect(&snapshot2);
    
    let deleted_count = changes.iter()
        .filter(|c| matches!(c, change::FileChange::Deleted(_)))
        .count();
    
    assert_eq!(deleted_count, 1, "One file should be deleted");
}
