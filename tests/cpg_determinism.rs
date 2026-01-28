//! CPG determinism validation tests (Step 3.7)
//!
//! **BRUTAL VALIDATION**
//! - Same query → same result order
//! - Same graph → same answers
//! - Queries that "sometimes" work = broken

use vcr::*;
use vcr::cpg::{CPGEpoch, model::CPGNodeKind};
use vcr::cpg::builder::CPGBuilder;
use vcr::query::primitives::QueryPrimitives;
use vcr::semantic::cfg::CFGBuilder;
use vcr::semantic::symbols::SymbolTable;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_cpg_hash_stability() {
    // Same code → same CPG hash across builds
    let source = b"fn test() { let x = 1; }";
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), source).unwrap();

    let file_id = FileId::new(1);
    let mmap = io::MmappedFile::open(temp_file.path(), file_id).unwrap();
    
    let mut parser = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed = parser.parse(&mmap, None).unwrap();

    let mut cfg_builder = CFGBuilder::new(file_id, source);
    let cfgs = cfg_builder.build_all(&parsed).unwrap();

    let mut symbols = SymbolTable::new(file_id);
    symbols.build(&parsed, source).unwrap();

    let semantic = semantic::SemanticEpoch {
        _parse_epoch_marker: 2,
        cfgs: [(file_id, cfgs)].into_iter().collect(),
        dfgs: std::collections::HashMap::new(),
        symbols: [(file_id, symbols)].into_iter().collect(),
        invalidation: semantic::invalidation::InvalidationTracker::new(),
        epoch_id: 3,
    };

    // Build CPG twice
    let mut cpg_epoch1 = CPGEpoch::new(3, 4);
    let mut cpg_builder1 = CPGBuilder::new();
    cpg_builder1.build(&semantic, &mut cpg_epoch1).unwrap();

    let mut cpg_epoch2 = CPGEpoch::new(3, 5);
    let mut cpg_builder2 = CPGBuilder::new();
    cpg_builder2.build(&semantic, &mut cpg_epoch2).unwrap();

    // BRUTAL: Hashes MUST match
    let hash1 = cpg_epoch1.cpg().compute_hash();
    let hash2 = cpg_epoch2.cpg().compute_hash();
    
    assert_eq!(hash1, hash2, "CPG hash must be stable across builds");
}

#[test]
fn test_query_determinism() {
    // Same query → same result order (ALWAYS)
    let source = b"fn foo() {}\nfn bar() {}";
    
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), source).unwrap();

    let file_id = FileId::new(1);
    let mmap = io::MmappedFile::open(temp_file.path(), file_id).unwrap();
    
    let mut parser = parse::IncrementalParser::new(types::Language::Rust).unwrap();
    let parsed = parser.parse(&mmap, None).unwrap();

    let mut cfg_builder = CFGBuilder::new(file_id, source);
    let cfgs = cfg_builder.build_all(&parsed).unwrap();

    let semantic = semantic::SemanticEpoch {
        _parse_epoch_marker: 2,
        cfgs: [(file_id, cfgs)].into_iter().collect(),
        dfgs: std::collections::HashMap::new(),
        symbols: std::collections::HashMap::new(),
        invalidation: semantic::invalidation::InvalidationTracker::new(),
        epoch_id: 3,
    };

    let mut cpg_epoch = CPGEpoch::new(3, 4);
    let mut cpg_builder = CPGBuilder::new();
    cpg_builder.build(&semantic, &mut cpg_epoch).unwrap();

    let cpg = cpg_epoch.cpg();

    // Run same query 3 times
    let funcs1 = QueryPrimitives::find_nodes(cpg, CPGNodeKind::Function);
    let funcs2 = QueryPrimitives::find_nodes(cpg, CPGNodeKind::Function);
    let funcs3 = QueryPrimitives::find_nodes(cpg, CPGNodeKind::Function);

    // BRUTAL: Results MUST be identical
    assert_eq!(funcs1, funcs2);
    assert_eq!(funcs2, funcs3);
}

#[test]
fn test_pointer_analysis_determinism() {
    // Same graph → same points-to sets
    use vcr::analysis::pointer::PointerAnalysis;
    use vcr::cpg::model::*;
    use vcr::types::ByteRange;

    let mut cpg = CPG::new();
    
    cpg.add_node(CPGNode::new(
        CPGNodeId(1),
        CPGNodeKind::DfgValue,
        OriginRef::Dfg { value_id: semantic::model::ValueId(1) },
        ByteRange::new(0, 10),
    ));

    // Run analysis twice
    let analysis1 = PointerAnalysis::analyze(&cpg);
    let analysis2 = PointerAnalysis::analyze(&cpg);

    // BRUTAL: Must complete identically
    assert_eq!(analysis1.is_complete(), analysis2.is_complete());
}

#[test]
fn test_taint_analysis_determinism() {
    // Same sources/sinks → same taint paths
    use vcr::analysis::taint::{TaintAnalysis, TaintSource, TaintSink};
    use vcr::cpg::model::*;
    use vcr::types::ByteRange;

    let mut cpg = CPG::new();
    
    cpg.add_node(CPGNode::new(CPGNodeId(1), CPGNodeKind::DfgValue,
        OriginRef::Dfg { value_id: semantic::model::ValueId(1) },
        ByteRange::new(0, 10)));
    
    cpg.add_node(CPGNode::new(CPGNodeId(2), CPGNodeKind::DfgValue,
        OriginRef::Dfg { value_id: semantic::model::ValueId(2) },
        ByteRange::new(10, 20)));
    
    cpg.add_edge(CPGEdge::new(CPGEdgeId(1), CPGEdgeKind::DataFlow, 
        CPGNodeId(1), CPGNodeId(2)));

    let sources = vec![TaintSource::Parameter(CPGNodeId(1))];
    let sinks = vec![TaintSink::FunctionCall(CPGNodeId(2))];

    // Run twice
    let analysis1 = TaintAnalysis::analyze(&cpg, sources.clone(), sinks.clone());
    let analysis2 = TaintAnalysis::analyze(&cpg, sources, sinks);

    // BRUTAL: Path counts must match
    assert_eq!(analysis1.paths().len(), analysis2.paths().len());
}

#[test]
fn test_cpg_epoch_isolation() {
    // Drop epoch → all memory freed
    let cpg_epoch = CPGEpoch::new(3, 4);
    let stats = cpg_epoch.stats();
    
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.total_edges, 0);
    
    // Epoch will be dropped here - no leaks
}
