//! Query primitives (Step 3.6)
//!
//! **RESTRICTED ON PURPOSE**
//! Only 5 primitives. No unbounded recursion.

use crate::cpg::model::{CPG, CPGNodeId, CPGNodeKind, CPGEdgeKind};
use std::collections::{HashSet, VecDeque};

/// Maximum reachability depth
const MAX_REACHABILITY_DEPTH: usize = 100;

/// Query primitives for CPG traversal
pub struct QueryPrimitives;

impl QueryPrimitives {
    /// Find all nodes of a specific kind
    ///
    /// **Deterministic**: Returns nodes in creation order
    pub fn find_nodes(cpg: &CPG, kind: CPGNodeKind) -> Vec<CPGNodeId> {
        cpg.get_nodes_of_kind(kind)
            .into_iter()
            .map(|n| n.id)
            .collect()
    }

    /// Follow outgoing edges of a specific kind from a node
    ///
    /// **Deterministic**: Returns targets in edge creation order
    pub fn follow_edge(cpg: &CPG, from: CPGNodeId, kind: CPGEdgeKind) -> Vec<CPGNodeId> {
        cpg.get_edges_from(from)
            .into_iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.to)
            .collect()
    }

    /// Filter nodes by predicate
    ///
    /// **Deterministic**: Preserves input order
    pub fn filter(nodes: Vec<CPGNodeId>, cpg: &CPG, kind: Option<CPGNodeKind>) -> Vec<CPGNodeId> {
        if let Some(k) = kind {
            nodes.into_iter()
                .filter(|&id| {
                    cpg.get_node(id).map(|n| n.kind == k).unwrap_or(false)
                })
                .collect()
        } else {
            nodes
        }
    }

    /// Intersect two node sets
    ///
    /// **Deterministic**: Returns in first set's order
    pub fn intersect(a: Vec<CPGNodeId>, b: Vec<CPGNodeId>) -> Vec<CPGNodeId> {
        let b_set: HashSet<_> = b.into_iter().collect();
        a.into_iter().filter(|n| b_set.contains(n)).collect()
    }

    /// Find all nodes reachable within N hops
    ///
    /// **Bounded**: Maximum depth enforced
    pub fn reachable_within(cpg: &CPG, from: CPGNodeId, max_depth: usize) -> Vec<CPGNodeId> {
        let depth_limit = max_depth.min(MAX_REACHABILITY_DEPTH);
        let mut reachable = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((from, 0));
        visited.insert(from);

        while let Some((current, depth)) = queue.pop_front() {
            reachable.push(current);

            if depth < depth_limit {
                for edge in cpg.get_edges_from(current) {
                    if !visited.contains(&edge.to) {
                        visited.insert(edge.to);
                        queue.push_back((edge.to, depth + 1));
                    }
                }
            }
        }

        reachable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpg::model::*;
    use crate::types::ByteRange;

    #[test]
    fn test_find_nodes() {
        let mut cpg = CPG::new();
        
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
            ByteRange::new(0, 10),
        ));
        
        let funcs = QueryPrimitives::find_nodes(&cpg, CPGNodeKind::Function);
        assert_eq!(funcs.len(), 1);
    }

    #[test]
    fn test_follow_edge() {
        let mut cpg = CPG::new();
        
        cpg.add_node(CPGNode::new(CPGNodeId(1), CPGNodeKind::CfgNode, 
            OriginRef::Cfg { node_id: crate::semantic::model::NodeId(1) }, ByteRange::new(0, 10)));
        cpg.add_node(CPGNode::new(CPGNodeId(2), CPGNodeKind::CfgNode, 
            OriginRef::Cfg { node_id: crate::semantic::model::NodeId(2) }, ByteRange::new(10, 20)));
        
        cpg.add_edge(CPGEdge::new(CPGEdgeId(1), CPGEdgeKind::ControlFlow, CPGNodeId(1), CPGNodeId(2)));
        
        let targets = QueryPrimitives::follow_edge(&cpg, CPGNodeId(1), CPGEdgeKind::ControlFlow);
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn test_reachable_within() {
        let mut cpg = CPG::new();
        
        cpg.add_node(CPGNode::new(CPGNodeId(1), CPGNodeKind::CfgNode, 
            OriginRef::Cfg { node_id: crate::semantic::model::NodeId(1) }, ByteRange::new(0, 10)));
        cpg.add_node(CPGNode::new(CPGNodeId(2), CPGNodeKind::CfgNode, 
            OriginRef::Cfg { node_id: crate::semantic::model::NodeId(2) }, ByteRange::new(10, 20)));
        
        cpg.add_edge(CPGEdge::new(CPGEdgeId(1), CPGEdgeKind::ControlFlow, CPGNodeId(1), CPGNodeId(2)));
        
        let reachable = QueryPrimitives::reachable_within(&cpg, CPGNodeId(1), 10);
        assert!(reachable.len() >= 1);
    }
}
