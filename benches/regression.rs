//! Benchmark harness (Path B4)
//!
//! Performance regression tracking

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vcr::*;
use vcr::cpg::model::{CPG, CPGNode, CPGNodeId, CPGNodeKind, OriginRef};
use vcr::types::ByteRange;
use vcr::execution::{ExecutionPlan, Stage, Task, TaskId, WorkFragment, Scheduler, DeterministicOrder};

fn bench_cpg_build(c: &mut Criterion) {
    c.bench_function("cpg_build_100_nodes", |b| {
        b.iter(|| {
            let mut cpg = CPG::new();
            for i in 0..100 {
                cpg.add_node(black_box(CPGNode::new(
                    CPGNodeId(i),
                    CPGNodeKind::Function,
                    OriginRef::Function { function_id: semantic::model::FunctionId(i) },
                    ByteRange::new((i * 10) as usize, ((i + 1) * 10) as usize),
                )));
            }
            cpg
        });
    });
}

fn bench_query_execution(c: &mut Criterion) {
    let mut cpg = CPG::new();
    for i in 0..100 {
        cpg.add_node(CPGNode::new(
            CPGNodeId(i),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: semantic::model::FunctionId(i) },
            ByteRange::new((i * 10) as usize, ((i + 1) * 10) as usize),
        ));
    }

    c.bench_function("query_find_nodes", |b| {
        b.iter(|| {
            let task = Task::new(
                TaskId(1),
                WorkFragment::FindNodes { kind: CPGNodeKind::Function },
                vec![],
                0,
            );
            let stage = Stage::new(vec![task], DeterministicOrder::TaskId);
            let mut plan = ExecutionPlan::new();
            plan.add_stage(stage);
            
            let scheduler = Scheduler::new(1);
            black_box(scheduler.execute(&plan, &cpg))
        });
    });
}

fn bench_cpg_hash(c: &mut Criterion) {
    let mut cpg = CPG::new();
    for i in 0..100 {
        cpg.add_node(CPGNode::new(
            CPGNodeId(i),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: semantic::model::FunctionId(i) },
            ByteRange::new((i * 10) as usize, ((i + 1) * 10) as usize),
        ));
    }

    c.bench_function("cpg_hash_100_nodes", |b| {
        b.iter(|| black_box(cpg.compute_hash()));
    });
}

criterion_group!(benches, bench_cpg_build, bench_query_execution, bench_cpg_hash);
criterion_main!(benches);
