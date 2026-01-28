//! Task scheduler - parallel execution, serial commit
//!
//! **Critical**: All commits happen on one thread in deterministic order

use crate::cpg::model::{CPG, CPGNodeId};
use crate::execution::plan::{ExecutionPlan, DeterministicOrder};
use crate::execution::task::{Task, TaskId, WorkFragment};
use crate::query::primitives::QueryPrimitives;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Query result
pub type QueryResult = Vec<CPGNodeId>;

/// Scheduler for parallel execution
pub struct Scheduler {
    /// Thread pool size
    thread_count: usize,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count: thread_count.max(1),
        }
    }

    /// Execute a plan
    ///
    /// **Deterministic**: Same plan + CPG = same result
    pub fn execute(&self, plan: &ExecutionPlan, cpg: &CPG) -> Vec<QueryResult> {
        let mut results = Vec::new();

        // Execute each stage in order
        for stage in &plan.stages {
            let stage_results = self.execute_stage(stage, cpg);
            results.extend(stage_results);
        }

        results
    }

    /// Execute a single stage
    fn execute_stage(&self, stage: &crate::execution::plan::Stage, cpg: &CPG) -> Vec<QueryResult> {
        let task_count = stage.parallel_tasks.len();
        
        // Result storage (one slot per task)
        let results: Arc<Mutex<HashMap<usize, QueryResult>>> = Arc::new(Mutex::new(HashMap::new()));
        
        // For now, execute serially (parallel execution with rayon would go here)
        // This is the **correct** serial baseline for validation
        for task in &stage.parallel_tasks {
            let result = self.execute_task(task, cpg);
            results.lock().unwrap().insert(task.result_slot, result);
        }
        
        // Commit in deterministic order
        let tasks_ordered = stage.tasks_in_commit_order();
        let results_lock = results.lock().unwrap();
        
        tasks_ordered
            .iter()
            .map(|task| results_lock.get(&task.result_slot).cloned().unwrap_or_default())
            .collect()
    }

    /// Execute a single task
    fn execute_task(&self, task: &Task, cpg: &CPG) -> QueryResult {
        match &task.work {
            WorkFragment::FindNodes { kind } => {
                QueryPrimitives::find_nodes(cpg, *kind)
            }
            WorkFragment::FollowEdges { from, kind } => {
                let mut result = Vec::new();
                for node in from {
                    result.extend(QueryPrimitives::follow_edge(cpg, *node, *kind));
                }
                result
            }
            WorkFragment::Filter { nodes, kind } => {
                QueryPrimitives::filter(nodes.clone(), cpg, *kind)
            }
            WorkFragment::Intersect { a, b } => {
                QueryPrimitives::intersect(a.clone(), b.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::plan::Stage;
    use crate::cpg::model::*;
    use crate::types::ByteRange;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new(4);
        assert_eq!(scheduler.thread_count, 4);
    }

    #[test]
    fn test_execute_simple_plan() {
        let mut cpg = CPG::new();
        
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        let task = Task::new(
            TaskId(1),
            WorkFragment::FindNodes {
                kind: CPGNodeKind::Function,
            },
            vec![],
            0,
        );

        let stage = Stage::new(vec![task], DeterministicOrder::TaskId);
        
        let mut plan = ExecutionPlan::new();
        plan.add_stage(stage);

        let scheduler = Scheduler::new(1);
        let results = scheduler.execute(&plan, &cpg);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 1);
    }
}
