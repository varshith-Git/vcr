//! Bounded pointer/alias analysis (Step 3.4)
//!
//! **Algorithm**: Andersen-style, flow-insensitive
//! **No heap modeling initially**
//! **No field sensitivity initially**
//!
//! ## Design Principles
//!
//! - Deterministic and monotonic
//! - Capped growth (mark "unknown" if explodes)
//! - Explainable results only
//!
//! ## Not Trying To Be Clever
//!
//! This is **correct but incomplete** > fast and wrong

use crate::cpg::model::{CPG, CPGNodeId, CPGNodeKind, CPGEdgeKind};
use crate::semantic::model::ValueId;
use std::collections::{HashMap, HashSet};

/// Maximum points-to set size before marking "unknown"
const MAX_POINTSTO_SIZE: usize = 100;

/// Pointer analysis results
pub struct PointerAnalysis {
    /// Points-to sets: ValueId → Set of ValueId it may point to
    points_to: HashMap<ValueId, PointsToSet>,
    
    /// Whether analysis completed without overflow
    completed: bool,
}

/// Points-to set for a value
#[derive(Debug, Clone)]
pub enum PointsToSet {
    /// Known set of targets
    Known(HashSet<ValueId>),
    
    /// Unknown (analysis overflow)
    Unknown,
}

impl PointerAnalysis {
    /// Create empty pointer analysis
    pub fn new() -> Self {
        Self {
            points_to: HashMap::new(),
            completed: true,
        }
    }

    /// Run analysis on CPG
    ///
    /// **Bounded**: Will mark "unknown" if growth explodes
    pub fn analyze(cpg: &CPG) -> Self {
        let mut analysis = Self::new();

        // Step 1: Initialize points-to sets for all DFG values
        for node in &cpg.nodes {
            if node.kind == CPGNodeKind::DfgValue {
                if let crate::cpg::model::OriginRef::Dfg { value_id } = node.origin {
                    analysis.points_to.insert(value_id, PointsToSet::Known(HashSet::new()));
                }
            }
        }

        // Step 2: Propagate along DataFlow edges (simplified)
        // In real Andersen's, would iterate to fixed point
        // For now, single pass over edges
        
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;
        
        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;
            
            for edge in &cpg.edges {
                if edge.kind == CPGEdgeKind::DataFlow {
                    // Get source and target value IDs
                    if let (Some(from_node), Some(to_node)) = (cpg.get_node(edge.from), cpg.get_node(edge.to)) {
                        if let (
                            crate::cpg::model::OriginRef::Dfg { value_id: from_id },
                            crate::cpg::model::OriginRef::Dfg { value_id: to_id }
                        ) = (from_node.origin, to_node.origin) {
                            // Propagate: if x → y, then pts(y) ⊇ pts(x)
                            if analysis.propagate_points_to(from_id, to_id) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        if iterations >= MAX_ITERATIONS {
            analysis.completed = false;
        }

        analysis
    }

    /// Propagate points-to set from source to target
    ///
    /// Returns true if target set changed
    fn propagate_points_to(&mut self, from: ValueId, to: ValueId) -> bool {
        // Get from set (clone to avoid borrow issues)
        let from_set = match self.points_to.get(&from) {
            Some(PointsToSet::Known(set)) => set.clone(),
            Some(PointsToSet::Unknown) | None => return false,
        };

        // Get or create to set
        let to_set = self.points_to.entry(to).or_insert_with(|| PointsToSet::Known(HashSet::new()));

        match to_set {
            PointsToSet::Known(set) => {
                let old_size = set.len();
                set.extend(&from_set);
                
                // Check for overflow
                if set.len() > MAX_POINTSTO_SIZE {
                    *to_set = PointsToSet::Unknown;
                    self.completed = false;
                    return true;
                }
                
                set.len() > old_size
            }
            PointsToSet::Unknown => false,
        }
    }

    /// Get points-to set for a value
    pub fn points_to(&self, value: ValueId) -> Option<&PointsToSet> {
        self.points_to.get(&value)
    }

    /// Check if analysis completed without overflow
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Get statistics
    pub fn stats(&self) -> PointerAnalysisStats {
        let mut known_count = 0;
        let mut unknown_count = 0;
        let mut total_edges = 0;

        for set in self.points_to.values() {
            match set {
                PointsToSet::Known(s) => {
                    known_count += 1;
                    total_edges += s.len();
                }
                PointsToSet::Unknown => unknown_count += 1,
            }
        }

        PointerAnalysisStats {
            values_analyzed: self.points_to.len(),
            known_sets: known_count,
            unknown_sets: unknown_count,
            total_points_to_edges: total_edges,
            completed: self.completed,
        }
    }
}

/// Statistics about pointer analysis
#[derive(Debug, Clone)]
pub struct PointerAnalysisStats {
    pub values_analyzed: usize,
    pub known_sets: usize,
    pub unknown_sets: usize,
    pub total_points_to_edges: usize,
    pub completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpg::model::*;
    use crate::types::ByteRange;

    #[test]
    fn test_pointer_analysis_empty() {
        let cpg = CPG::new();
        let analysis = PointerAnalysis::analyze(&cpg);
        
        assert!(analysis.is_complete());
        assert_eq!(analysis.points_to.len(), 0);
    }

    #[test]
    fn test_pointer_analysis_simple() {
        let mut cpg = CPG::new();
        
        // Create two DFG value nodes
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::DfgValue,
            OriginRef::Dfg { value_id: ValueId(1) },
            ByteRange::new(0, 10),
        ));
        
        cpg.add_node(CPGNode::new(
            CPGNodeId(2),
            CPGNodeKind::DfgValue,
            OriginRef::Dfg { value_id: ValueId(2) },
            ByteRange::new(10, 20),
        ));
        
        // Add data flow edge
        cpg.add_edge(CPGEdge::new(
            CPGEdgeId(1),
            CPGEdgeKind::DataFlow,
            CPGNodeId(1),
            CPGNodeId(2),
        ));
        
        let analysis = PointerAnalysis::analyze(&cpg);
        
        assert!(analysis.is_complete());
        assert_eq!(analysis.points_to.len(), 2);
    }

    #[test]
    fn test_pointer_analysis_stats() {
        let cpg = CPG::new();
        let analysis = PointerAnalysis::analyze(&cpg);
        let stats = analysis.stats();
        
        assert_eq!(stats.values_analyzed, 0);
        assert_eq!(stats.known_sets, 0);
        assert_eq!(stats.unknown_sets, 0);
        assert!(stats.completed);
    }
}
