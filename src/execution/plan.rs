//! Execution plan - parallel compute, serial commit
//!
//! **Critical**: Results merged in deterministic order

use crate::execution::task::Task;

/// Deterministic ordering for commit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterministicOrder {
    /// Tasks committed in TaskId order
    TaskId,
    
    /// Tasks committed in completion order (with stable tie-breaking)
    Stable,
}

/// Execution stage - parallel tasks with deterministic commit
#[derive(Debug, Clone)]
pub struct Stage {
    /// Tasks that can execute in parallel
    pub parallel_tasks: Vec<Task>,
    
    /// How to order commits
    pub commit_order: DeterministicOrder,
}

impl Stage {
    /// Create a new stage
    pub fn new(parallel_tasks: Vec<Task>, commit_order: DeterministicOrder) -> Self {
        Self {
            parallel_tasks,
            commit_order,
        }
    }

    /// Get tasks sorted by commit order
    pub fn tasks_in_commit_order(&self) -> Vec<&Task> {
        let mut tasks: Vec<_> = self.parallel_tasks.iter().collect();
        
        match self.commit_order {
            DeterministicOrder::TaskId => {
                tasks.sort_by_key(|t| t.id);
            }
            DeterministicOrder::Stable => {
                // Already in stable order (Vec preserves insertion order)
            }
        }
        
        tasks
    }
}

/// Execution plan - multiple stages
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Stages to execute (in order)
    pub stages: Vec<Stage>,
}

impl ExecutionPlan {
    /// Create empty plan
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
        }
    }

    /// Add a stage
    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }

    /// Get total task count
    pub fn task_count(&self) -> usize {
        self.stages.iter().map(|s| s.parallel_tasks.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::task::WorkFragment;

    #[test]
    fn test_stage_creation() {
        let tasks = vec![
            Task::new(
                TaskId(1),
                WorkFragment::FindNodes {
                    kind: crate::cpg::model::CPGNodeKind::Function,
                },
                vec![],
                0,
            ),
        ];

        let stage = Stage::new(tasks, DeterministicOrder::TaskId);
        assert_eq!(stage.parallel_tasks.len(), 1);
    }

    #[test]
    fn test_execution_plan() {
        let mut plan = ExecutionPlan::new();
        
        let stage = Stage::new(vec![], DeterministicOrder::TaskId);
        plan.add_stage(stage);
        
        assert_eq!(plan.stages.len(), 1);
    }

    #[test]
    fn test_commit_order() {
        let tasks = vec![
            Task::new(TaskId(3), WorkFragment::FindNodes {
                kind: crate::cpg::model::CPGNodeKind::Function,
            }, vec![], 0),
            Task::new(TaskId(1), WorkFragment::FindNodes {
                kind: crate::cpg::model::CPGNodeKind::Function,
            }, vec![], 1),
            Task::new(TaskId(2), WorkFragment::FindNodes {
                kind: crate::cpg::model::CPGNodeKind::Function,
            }, vec![], 2),
        ];

        let stage = Stage::new(tasks, DeterministicOrder::TaskId);
        let ordered = stage.tasks_in_commit_order();
        
        // Should be sorted by TaskId
        assert_eq!(ordered[0].id, TaskId(1));
        assert_eq!(ordered[1].id, TaskId(2));
        assert_eq!(ordered[2].id, TaskId(3));
    }
}
