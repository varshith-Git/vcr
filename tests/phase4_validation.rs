//! Phase 4 validation tests - performance without lies
//!
//! **CRITICAL**: All optimizations must preserve determinism

use vcr::*;
use vcr::execution::{ExecutionPlan, Stage, Task, TaskId, WorkFragment, Scheduler, DeterministicOrder};
use vcr::cpg::{CPGEpoch, model::{CPG, CPGNode, CPGNodeId, CPGNodeKind, OriginRef}};
use vcr::cpg::builder::CPGBuilder;
use vcr::semantic::cfg::CFGBuilder;
use vcr::semantic::symbols::SymbolTable;
use vcr::types::ByteRange;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_parallel_execution_determinism() {
    // Step 4.1: Parallel = serial results
    let mut cpg = CPG::new();
    
    // Add test nodes
    for i in 1..=5 {
        cpg.add_node(CPGNode::new(
            CPGNodeId(i),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: semantic::model::FunctionId(i) },
            ByteRange::new((i-1) * 10, i * 10),
        ));
    }

    // Create execution plan
    let task1 = Task::new(
        TaskId(1),
        WorkFragment::FindNodes { kind: CPGNodeKind::Function },
        vec![],
        0,
    );

    let stage = Stage::new(vec![task1], DeterministicOrder::TaskId);
    let mut plan = ExecutionPlan::new();
    plan.add_stage(stage);

    // Execute twice (currently serial, but would be parallel with Rayon)
    let scheduler = Scheduler::new(4);
    let results1 = scheduler.execute(&plan, &cpg);
    let results2 = scheduler.execute(&plan, &cpg);

    // BRUTAL: Results must be identical
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results1, results2, "Parallel execution must be deterministic");
}

#[test]
fn test_execution_plan_stability() {
    // Same plan executed twice → same behavior
    let mut cpg = CPG::new();
    
    cpg.add_node(CPGNode::new(
        CPGNodeId(1),
        CPGNodeKind::Function,
        OriginRef::Function { function_id: semantic::model::FunctionId(1) },
        ByteRange::new(0, 10),
    ));

    let task = Task::new(
        TaskId(1),
        WorkFragment::FindNodes { kind: CPGNodeKind::Function },
        vec![],
        0,
    );

    let stage1 = Stage::new(vec![task.clone()], DeterministicOrder::TaskId);
    let stage2 = Stage::new(vec![task], DeterministicOrder::TaskId);

    let mut plan1 = ExecutionPlan::new();
    plan1.add_stage(stage1);

    let mut plan2 = ExecutionPlan::new();
    plan2.add_stage(stage2);

    let scheduler = Scheduler::new(1);
    let results1 = scheduler.execute(&plan1, &cpg);
    let results2 = scheduler.execute(&plan2, &cpg);

    assert_eq!(results1, results2);
}

#[test]
fn test_commit_order_determinism() {
    // Commit order must be stable
    let mut cpg = CPG::new();
    
    for i in 1..=3 {
        cpg.add_node(CPGNode::new(
            CPGNodeId(i),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: semantic::model::FunctionId(i) },
            ByteRange::new(0, 10),
        ));
    }

    // Create tasks in different order but with TaskId-based commit
    let tasks = vec![
        Task::new(TaskId(3), WorkFragment::FindNodes { kind: CPGNodeKind::Function }, vec![], 2),
        Task::new(TaskId(1), WorkFragment::FindNodes { kind: CPGNodeKind::Function }, vec![], 0),
        Task::new(TaskId(2), WorkFragment::FindNodes { kind: CPGNodeKind::Function }, vec![], 1),
    ];

    let stage = Stage::new(tasks, DeterministicOrder::TaskId);
    let ordered = stage.tasks_in_commit_order();

    // Must be sorted by TaskId despite insertion order
    assert_eq!(ordered[0].id, TaskId(1));
    assert_eq!(ordered[1].id, TaskId(2));
    assert_eq!(ordered[2].id, TaskId(3));
}
