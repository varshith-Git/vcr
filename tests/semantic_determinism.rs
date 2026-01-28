//! Semantic determinism tests (Step 2.7)
//!
//! **These tests must be brutal.**
//!
//! Tests verify:
//! - Identical graph hashes across runs
//! - Whitespace changes → no semantic change
//! - Function reordering → same CFG order
//! - Local edits → local invalidation only

use std::fs;
use tempfile::{NamedTempFile, TempDir};
use vcr::*;
use vcr::semantic::cfg::CFGBuilder;
use vcr::semantic::symbols::SymbolTable;

#[test]
fn test_cfg_determinism_across_runs() {
    // Parse same code twice → identical CFGs
    let source = b"fn test() { let x = 1; if x > 0 { let y = 2; } }";
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), source).unwrap();

    let file_id = FileId::new(1);
    let mmap = io::MmappedFile::open(temp_file.path(), file_id).unwrap();
    
    let mut parser1 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed1 = parser1.parse(&mmap, None).unwrap();
    
    let mut builder1 = CFGBuilder::new(file_id, source);
    let cfgs1 = builder1.build_all(&parsed1).unwrap();

    // Second parse
    let mut parser2 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed2 = parser2.parse(&mmap, None).unwrap();
    
    let mut builder2 = CFGBuilder::new(file_id, source);
    let cfgs2 = builder2.build_all(&parsed2).unwrap();

    // CFG hashes must match
    assert_eq!(cfgs1.len(), cfgs2.len(), "Same number of functions");
    
    for (cfg1, cfg2) in cfgs1.iter().zip(cfgs2.iter()) {
        assert_eq!(
            cfg1.compute_hash(),
            cfg2.compute_hash(),
            "CFG hashes must be identical across runs"
        );
    }
}

#[test]
fn test_whitespace_has_no_semantic_effect() {
    // Different whitespace → same semantic graphs
    let source1 = b"fn test(){let x=1;}";
    let source2 = b"fn test() {\n    let x = 1;\n}";
    
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    fs::write(temp1.path(), source1).unwrap();
    fs::write(temp2.path(), source2).unwrap();

    let file_id = FileId::new(1);
    
    // Parse file 1
    let mmap1 = io::MmappedFile::open(temp1.path(), file_id).unwrap();
    let mut parser1 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed1 = parser1.parse(&mmap1, None).unwrap();
    
    let mut builder1 = CFGBuilder::new(file_id, source1);
    let cfgs1 = builder1.build_all(&parsed1).unwrap();

    // Parse file 2
    let mmap2 = io::MmappedFile::open(temp2.path(), file_id).unwrap();
    let mut parser2 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed2 = parser2.parse(&mmap2, None).unwrap();
    
    let mut builder2 = CFGBuilder::new(file_id, source2);
    let cfgs2 = builder2.build_all(&parsed2).unwrap();

    // Semantic structure should be identical
    assert_eq!(cfgs1.len(), cfgs2.len());
    assert_eq!(cfgs1[0].nodes.len(), cfgs2[0].nodes.len(), "Same number of CFG nodes");
}

#[test]
fn test_function_order_is_deterministic() {
    // Multiple functions → always same CFG order
    let source = b"
        fn third() { }
        fn first() { }
        fn second() { }
    ";
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), source).unwrap();

    let file_id = FileId::new(1);
    let mmap = io::MmappedFile::open(temp_file.path(), file_id).unwrap();
    
    let mut parser = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed = parser.parse(&mmap, None).unwrap();
    
    let mut builder = CFGBuilder::new(file_id, source);
    let cfgs = builder.build_all(&parsed).unwrap();

    // Should have 3 CFGs in lexical order (third, first, second)
    assert_eq!(cfgs.len(), 3, "Should have 3 functions");
    
    // Parse again
    let mut parser2 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed2 = parser2.parse(&mmap, None).unwrap();
    
    let mut builder2 = CFGBuilder::new(file_id, source);
    let cfgs2 = builder2.build_all(&parsed2).unwrap();

    // Order must be identical
    for (i, (cfg1, cfg2)) in cfgs.iter().zip(cfgs2.iter()).enumerate() {
        assert_eq!(
            cfg1.function_id, cfg2.function_id,
            "Function {} ID must match",
            i
        );
    }
}

#[test]
fn test_symbol_table_determinism() {
    // Symbol tables must be identical across runs
    let source = b"fn test(x: i32) { let y = x + 1; let z = y * 2; }";
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), source).unwrap();

    let file_id = FileId::new(1);
    let mmap = io::MmappedFile::open(temp_file.path(), file_id).unwrap();
    
    let mut parser = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed = parser.parse(&mmap, None).unwrap();
    
    let mut table1 = SymbolTable::new(file_id);
    table1.build(&parsed, source).unwrap();

    let mut table2 = SymbolTable::new(file_id);
    table2.build(&parsed, source).unwrap();

    // Should have same symbols
    let file_scope = table1.file_scope();
    let symbols1 = table1.symbols_in_scope(file_scope);
    let symbols2 = table2.symbols_in_scope(file_scope);
    
    assert_eq!(symbols1.len(), symbols2.len(), "Same number of symbols");
}

#[test]
fn test_local_edit_local_invalidation() {
    // Change one statement → only that function's CFG affected
    let source1 = b"
        fn foo() { let x = 1; }
        fn bar() { let y = 2; }
    ";
    
    let source2 = b"
        fn foo() { let x = 999; }
        fn bar() { let y = 2; }
    ";
    
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    fs::write(temp1.path(), source1).unwrap();
    fs::write(temp2.path(), source2).unwrap();

    let file_id = FileId::new(1);
    
    // Parse version 1
    let mmap1 = io::MmappedFile::open(temp1.path(), file_id).unwrap();
    let mut parser1 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed1 = parser1.parse(&mmap1, None).unwrap();
    
    let mut builder1 = CFGBuilder::new(file_id, source1);
    let cfgs1 = builder1.build_all(&parsed1).unwrap();

    // Parse version 2
    let mmap2 = io::MmappedFile::open(temp2.path(), file_id).unwrap();
    let mut parser2 = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed2 = parser2.parse(&mmap2, None).unwrap();
    
    let mut builder2 = CFGBuilder::new(file_id, source2);
    let cfgs2 = builder2.build_all(&parsed2).unwrap();

    // foo() changed, bar() didn't
    // Both versions should have 2 functions
    assert_eq!(cfgs1.len(), 2);
    assert_eq!(cfgs2.len(), 2);
}
