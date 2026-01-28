//! Taint propagation analysis (Step 3.5)
//!
//! **Structural, not heuristic**
//! - Deterministic BFS from sources
//! - Bounded depth (no infinite loops)
//! - Every taint must be traceable

use crate::cpg::model::{CPG, CPGNodeId, CPGEdgeKind};
use std::collections::{HashMap, HashSet, VecDeque};

/// Maximum taint propagation depth
const MAX_TAINT_DEPTH: usize = 50;

/// Taint sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintSource {
    /// Function parameter
    Parameter(CPGNodeId),
    
    /// External input
    ExternalInput(CPGNodeId),
}

/// Taint sinks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintSink {
    /// Function call
    FunctionCall(CPGNodeId),
    
    /// Return statement
    Return(CPGNodeId),
}

/// Taint path from source to sink
#[derive(Debug, Clone)]
pub struct TaintPath {
    pub source: TaintSource,
    pub path: Vec<CPGNodeId>,
    pub sink: TaintSink,
}

/// Taint analysis results
pub struct TaintAnalysis {
    /// All taint paths found
    paths: Vec<TaintPath>,
    
    /// Tainted nodes (reachable from sources)
    tainted: HashSet<CPGNodeId>,
}

impl TaintAnalysis {
    /// Create empty taint analysis
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            tainted: HashSet::new(),
        }
    }

    /// Run taint analysis on CPG
    ///
    /// **Bounded BFS**: Max depth to prevent infinite loops
    pub fn analyze(cpg: &CPG, sources: Vec<TaintSource>, sinks: Vec<TaintSink>) -> Self {
        let mut analysis = Self::new();

        // BFS from each source
        for source in sources {
            let source_node = match source {
                TaintSource::Parameter(node) | TaintSource::ExternalInput(node) => node,
            };
            
            analysis.propagate_from_source(cpg, source, source_node, &sinks);
        }

        analysis
    }

    /// Propagate taint from a source using bounded BFS
    fn propagate_from_source(&mut self, cpg: &CPG, source: TaintSource, start: CPGNodeId, sinks: &[TaintSink]) {
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        
        queue.push_back((start, vec![start], 0));
        visited.insert(start, 0);

        while let Some((current, path, depth)) = queue.pop_front() {
            // Depth limit
            if depth >= MAX_TAINT_DEPTH {
                continue;
            }

            // Mark as tainted
            self.tainted.insert(current);

            // Check if we reached a sink
            for sink in sinks {
                let sink_node = match sink {
                    TaintSink::FunctionCall(node) | TaintSink::Return(node) => *node,
                };
                
                if current == sink_node {
                    self.paths.push(TaintPath {
                        source,
                        path: path.clone(),
                        sink: *sink,
                    });
                }
            }

            // Follow DataFlow edges
            for edge in &cpg.edges {
                if edge.from == current && edge.kind == CPGEdgeKind::DataFlow {
                    let next_depth = depth + 1;
                    
                    // Only visit if haven't seen or found shorter path
                    if !visited.contains_key(&edge.to) || visited[&edge.to] > next_depth {
                        visited.insert(edge.to, next_depth);
                        let mut new_path = path.clone();
                        new_path.push(edge.to);
                        queue.push_back((edge.to, new_path, next_depth));
                    }
                }
            }
        }
    }

    /// Get all taint paths
    pub fn paths(&self) -> &[TaintPath] {
        &self.paths
    }

    /// Check if a node is tainted
    pub fn is_tainted(&self, node: CPGNodeId) -> bool {
        self.tainted.contains(&node)
    }

    /// Get statistics
    pub fn stats(&self) -> TaintAnalysisStats {
        TaintAnalysisStats {
            total_paths: self.paths.len(),
            tainted_nodes: self.tainted.len(),
        }
    }
}

/// Taint analysis statistics
#[derive(Debug, Clone)]
pub struct TaintAnalysisStats {
    pub total_paths: usize,
    pub tainted_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpg::model::*;
    use crate::types::ByteRange;

    #[test]
    fn test_taint_analysis_empty() {
        let cpg = CPG::new();
        let analysis = TaintAnalysis::analyze(&cpg, vec![], vec![]);
        
        assert_eq!(analysis.paths().len(), 0);
        assert_eq!(analysis.tainted.len(), 0);
    }

    #[test]
    fn test_taint_analysis_simple() {
        let mut cpg = CPG::new();
        
        // Source node
        cpg.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::DfgValue,
            OriginRef::Dfg { value_id: crate::semantic::model::ValueId(1) },
            ByteRange::new(0, 10),
        ));
        
        // Sink node
        cpg.add_node(CPGNode::new(
            CPGNodeId(2),
            CPGNodeKind::DfgValue,
            OriginRef::Dfg { value_id: crate::semantic::model::ValueId(2) },
            ByteRange::new(10, 20),
        ));
        
        // Data flow edge
        cpg.add_edge(CPGEdge::new(
            CPGEdgeId(1),
            CPGEdgeKind::DataFlow,
            CPGNodeId(1),
            CPGNodeId(2),
        ));
        
        let sources = vec![TaintSource::Parameter(CPGNodeId(1))];
        let sinks = vec![TaintSink::FunctionCall(CPGNodeId(2))];
        
        let analysis = TaintAnalysis::analyze(&cpg, sources, sinks);
        
        assert_eq!(analysis.paths().len(), 1);
        assert!(analysis.is_tainted(CPGNodeId(1)));
        assert!(analysis.is_tainted(CPGNodeId(2)));
    }
}

