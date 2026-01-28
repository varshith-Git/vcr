//! Task definition for parallel execution
//!
//! Tasks are independent work units that can execute in parallel

use crate::cpg::model::CPGNodeId;


/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub u64);

/// Work fragment - independent computation
#[derive(Debug, Clone)]
pub enum WorkFragment {
    /// Find nodes of a specific kind
    FindNodes {
        kind: crate::cpg::model::CPGNodeKind,
    },
    
    /// Follow edges from a node
    FollowEdges {
        from: Vec<CPGNodeId>,
        kind: crate::cpg::model::CPGEdgeKind,
    },
    
    /// Filter nodes
    Filter {
        nodes: Vec<CPGNodeId>,
        kind: Option<crate::cpg::model::CPGNodeKind>,
    },
    
    /// Intersect two sets
    Intersect {
        a: Vec<CPGNodeId>,
        b: Vec<CPGNodeId>,
    },
}

/// Task with dependencies
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique ID
    pub id: TaskId,
    
    /// Work to perform
    pub work: WorkFragment,
    
    /// Task dependencies (must complete before this task)
    pub dependencies: Vec<TaskId>,
    
    /// Result storage location
    pub result_slot: usize,
}

impl Task {
    /// Create a new task
    pub fn new(id: TaskId, work: WorkFragment, dependencies: Vec<TaskId>, result_slot: usize) -> Self {
        Self {
            id,
            work,
            dependencies,
            result_slot,
        }
    }

    /// Check if task is ready to execute (all dependencies met)
    pub fn is_ready(&self, completed: &std::collections::HashSet<TaskId>) -> bool {
        self.dependencies.iter().all(|dep| completed.contains(dep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            TaskId(1),
            WorkFragment::FindNodes {
                kind: crate::cpg::model::CPGNodeKind::Function,
            },
            vec![],
            0,
        );

        assert_eq!(task.id, TaskId(1));
        assert_eq!(task.dependencies.len(), 0);
    }

    #[test]
    fn test_task_ready() {
        let task = Task::new(
            TaskId(2),
            WorkFragment::FindNodes {
                kind: crate::cpg::model::CPGNodeKind::Function,
            },
            vec![TaskId(1)],
            0,
        );

        let mut completed = std::collections::HashSet::new();
        assert!(!task.is_ready(&completed));

        completed.insert(TaskId(1));
        assert!(task.is_ready(&completed));
    }
}
