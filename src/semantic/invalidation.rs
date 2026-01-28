//! Dependency invalidation tracker (Step 2.5)
//!
//! **This is the most critical part of Phase 2.**
//!
//! Tracks dependencies between:
//! - AST byte ranges → CFG nodes
//! - CFG nodes → DFG edges  
//! - DFG edges → dependent facts
//!
//! Enables precise incremental updates:
//! When AST changes, we can determine exactly which semantic facts to rebuild.

use crate::semantic::model::{EdgeId, NodeId};
use crate::types::ByteRange;
use std::collections::HashMap;

/// Invalidation result - what needs to be rebuilt
#[derive(Debug, Clone)]
pub struct InvalidationSet {
    /// CFG nodes that need rebuilding
    pub cfg_nodes: Vec<NodeId>,
    
    /// DFG edges that need rebuilding
    pub dfg_edges: Vec<EdgeId>,
}

impl InvalidationSet {
    /// Create empty invalidation set
    pub fn new() -> Self {
        Self {
            cfg_nodes: Vec::new(),
            dfg_edges: Vec::new(),
        }
    }

    /// Check if anything needs invalidation
    pub fn is_empty(&self) -> bool {
        self.cfg_nodes.is_empty() && self.dfg_edges.is_empty()
    }
}

/// Tracks dependencies for incremental updates
///
/// **Determinism guarantee:** All lookups are deterministic.
/// HashMaps used only for fast lookup, not iteration order.
pub struct InvalidationTracker {
    /// AST byte range → CFG nodes affected by that range
    ast_to_cfg: HashMap<ByteRange, Vec<NodeId>>,
    
    /// CFG node → DFG edges that depend on it
    cfg_to_dfg: HashMap<NodeId, Vec<EdgeId>>,
}

impl InvalidationTracker {
    /// Create a new invalidation tracker
    pub fn new() -> Self {
        Self {
            ast_to_cfg: HashMap::new(),
            cfg_to_dfg: HashMap::new(),
        }
    }

    /// Register that a CFG node depends on an AST range
    pub fn track_ast_to_cfg(&mut self, range: ByteRange, node: NodeId) {
        self.ast_to_cfg
            .entry(range)
            .or_insert_with(Vec::new)
            .push(node);
    }

    /// Register that a DFG edge depends on a CFG node
    pub fn track_cfg_to_dfg(&mut self, node: NodeId, edge: EdgeId) {
        self.cfg_to_dfg
            .entry(node)
            .or_insert_with(Vec::new)
            .push(edge);
    }

    /// Determine what to invalidate given changed AST ranges
    ///
    /// **Algorithm:**
    /// 1. Find all CFG nodes overlapping changed ranges
    /// 2. Find all DFG edges depending on those nodes
    /// 3. Return invalidation set
    pub fn invalidate(&self, changed_ranges: &[ByteRange]) -> InvalidationSet {
        let mut result = InvalidationSet::new();

        // Step 1: Find affected CFG nodes
        for changed_range in changed_ranges {
            // Check for exact matches
            if let Some(nodes) = self.ast_to_cfg.get(changed_range) {
                result.cfg_nodes.extend(nodes);
            }

            // Check for overlaps (conservative)
            for (range, nodes) in &self.ast_to_cfg {
                if ranges_overlap(*range, *changed_range) {
                    result.cfg_nodes.extend(nodes);
                }
            }
        }

        // Deduplicate
        result.cfg_nodes.sort();
        result.cfg_nodes.dedup();

        // Step 2: Propagate to DFG
        for &node_id in &result.cfg_nodes {
            if let Some(edges) = self.cfg_to_dfg.get(&node_id) {
                result.dfg_edges.extend(edges);
            }
        }

        // Deduplicate
        result.dfg_edges.sort();
        result.dfg_edges.dedup();

        result
    }

    /// Get statistics for debugging
    pub fn stats(&self) -> InvalidationStats {
        InvalidationStats {
            ast_ranges: self.ast_to_cfg.len(),
            cfg_nodes: self.ast_to_cfg.values().map(|v| v.len()).sum(),
            dfg_edges: self.cfg_to_dfg.values().map(|v| v.len()).sum(),
        }
    }
}

/// Statistics about invalidation tracking
#[derive(Debug, Clone)]
pub struct InvalidationStats {
    /// Number of AST ranges tracked
    pub ast_ranges: usize,
    
    /// Total CFG nodes tracked
    pub cfg_nodes: usize,
    
    /// Total DFG edges tracked
    pub dfg_edges: usize,
}

/// Check if two byte ranges overlap
fn ranges_overlap(a: ByteRange, b: ByteRange) -> bool {
    // Ranges overlap if neither is completely before the other
    !(a.end <= b.start || b.end <= a.start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidation_tracking() {
        let mut tracker = InvalidationTracker::new();

        // Track some dependencies
        let range1 = ByteRange::new(0, 10);
        let range2 = ByteRange::new(20, 30);
        
        tracker.track_ast_to_cfg(range1, NodeId(1));
        tracker.track_ast_to_cfg(range1, NodeId(2));
        tracker.track_ast_to_cfg(range2, NodeId(3));

        tracker.track_cfg_to_dfg(NodeId(1), EdgeId(10));
        tracker.track_cfg_to_dfg(NodeId(2), EdgeId(11));

        // Change range1 → should invalidate nodes 1, 2 and edges 10, 11
        let inv = tracker.invalidate(&[range1]);
        
        assert!(inv.cfg_nodes.contains(&NodeId(1)));
        assert!(inv.cfg_nodes.contains(&NodeId(2)));
        assert!(!inv.cfg_nodes.contains(&NodeId(3)));

        assert!(inv.dfg_edges.contains(&EdgeId(10)));
        assert!(inv.dfg_edges.contains(&EdgeId(11)));
    }

    #[test]
    fn test_range_overlap() {
        assert!(ranges_overlap(
            ByteRange::new(0, 10),
            ByteRange::new(5, 15)
        ));

        assert!(!ranges_overlap(
            ByteRange::new(0, 10),
            ByteRange::new(10, 20)
        ));

        assert!(ranges_overlap(
            ByteRange::new(0, 100),
            ByteRange::new(50, 60)
        ));
    }

    #[test]
    fn test_empty_invalidation() {
        let tracker = InvalidationTracker::new();
        let inv = tracker.invalidate(&[ByteRange::new(0, 10)]);
        
        assert!(inv.is_empty());
    }

    #[test]
    fn test_stats() {
        let mut tracker = InvalidationTracker::new();
        
        tracker.track_ast_to_cfg(ByteRange::new(0, 10), NodeId(1));
        tracker.track_ast_to_cfg(ByteRange::new(0, 10), NodeId(2));
        tracker.track_cfg_to_dfg(NodeId(1), EdgeId(10));

        let stats = tracker.stats();
        assert_eq!(stats.ast_ranges, 1);
        assert_eq!(stats.cfg_nodes, 2);
        assert_eq!(stats.dfg_edges, 1);
    }
}
