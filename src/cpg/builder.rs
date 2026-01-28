//! CPG Builder - deterministic graph fusion (Step 3.2)
//!
//! **MECHANICAL, NOT CLEVER**
//!
//! Fusion order (fixed):
//! 1. Files (lexical order)
//! 2. Functions (lexical order per file)
//! 3. AST nodes (tree order)
//! 4. CFG nodes (program order)
//! 5. DFG values (definition order)

use crate::cpg::model::*;
use crate::cpg::epoch::CPGEpoch;
use crate::semantic::SemanticEpoch;
use crate::types::ByteRange;
use anyhow::Result;

/// CPG Builder - fuses AST + CFG + DFG
pub struct CPGBuilder {
    /// Next node ID
    next_node_id: u64,
    
    /// Next edge ID
    next_edge_id: u64,
}

impl CPGBuilder {
    /// Create a new CPG builder
    pub fn new() -> Self {
        Self {
            next_node_id: 0,
            next_edge_id: 0,
        }
    }

    /// Build CPG from semantic epoch
    ///
    /// **Order is fixed and deterministic**:
    /// 1. Files (sorted by FileId)
    /// 2. Functions (sorted by FunctionId per file)
    /// 3. CFG nodes (program order)
    /// 4. DFG values (definition order)
    pub fn build(&mut self, semantic: &SemanticEpoch, cpg_epoch: &mut CPGEpoch) -> Result<()> {
        let cpg = cpg_epoch.cpg_mut();
        
        // Get all files (sorted for determinism)
        let mut file_ids: Vec<_> = semantic.get_all_file_ids();
        file_ids.sort();
        
        for file_id in file_ids {
            // Step 1: Create file node
            let file_node = CPGNode::new(
                self.next_node_id(),
                CPGNodeKind::File,
                OriginRef::File { file_id },
                ByteRange::new(0, 0),  // Files don't have ranges
            );
            cpg.add_node(file_node);
            
            // Step 2: Get functions for this file (if any)
            if let Some(cfgs) = semantic.get_cfgs(file_id) {
                // Sort CFGs by function ID for determinism
                let mut sorted_cfgs: Vec<_> = cfgs.iter().collect();
                sorted_cfgs.sort_by_key(|cfg| cfg.function_id);
                
                for cfg in sorted_cfgs {
                    // Create function node
                    let func_node = CPGNode::new(
                        self.next_node_id(),
                        CPGNodeKind::Function,
                        OriginRef::Function { function_id: cfg.function_id },
                        ByteRange::new(0, 0),  // CFG doesn't store function range
                    );
                    cpg.add_node(func_node);
                    
                    // Step 3: Process CFG nodes (in order)
                    for cfg_node in &cfg.nodes {
                        let cpg_node = CPGNode::new(
                            self.next_node_id(),
                            CPGNodeKind::CfgNode,
                            OriginRef::Cfg { node_id: cfg_node.id },
                            cfg_node.source_range,
                        ).with_label(format!("{:?}", cfg_node.kind));
                        cpg.add_node(cpg_node);
                    }
                    
                    // Step 4: Process CFG edges
                    for cfg_edge in &cfg.edges {
                        let cpg_edge = CPGEdge::new(
                            self.next_edge_id(),
                            CPGEdgeKind::ControlFlow,
                            CPGNodeId(cfg_edge.from.0),
                            CPGNodeId(cfg_edge.to.0),
                        );
                        cpg.add_edge(cpg_edge);
                    }
                }
            }
            
            // Step 5: Get DFG for this file (if any)
            if let Some(dfgs) = semantic.get_dfgs(file_id) {
                for dfg in dfgs {
                    // Process DFG values (in order)
                    for dfg_value in &dfg.values {
                        let cpg_node = CPGNode::new(
                            self.next_node_id(),
                            CPGNodeKind::DfgValue,
                            OriginRef::Dfg { value_id: dfg_value.id },
                            dfg_value.source_range,
                        ).with_label(format!("{:?}", dfg_value.kind));
                        cpg.add_node(cpg_node);
                    }
                    
                    // Process DFG edges
                    for dfg_edge in &dfg.edges {
                        let cpg_edge = CPGEdge::new(
                            self.next_edge_id(),
                            CPGEdgeKind::DataFlow,
                            CPGNodeId(dfg_edge.from.0),
                            CPGNodeId(dfg_edge.to.0),
                        );
                        cpg.add_edge(cpg_edge);
                    }
                }
            }
            
            // Step 6: Get symbols for this file (if any)
            if let Some(symbol_table) = semantic.get_symbols(file_id) {
                // Process symbols from file scope
                let file_scope = symbol_table.file_scope();
                let symbols = symbol_table.symbols_in_scope(file_scope);
                
                for symbol in symbols {
                    let cpg_node = CPGNode::new(
                        self.next_node_id(),
                        CPGNodeKind::Symbol,
                        OriginRef::Symbol { symbol_id: symbol.id },
                        symbol.source_range,
                    ).with_label(symbol.name.clone());
                    cpg.add_node(cpg_node);
                }
            }
        }
        
        // Rebuild indices after fusion
        cpg_epoch.rebuild_indices();
        
        Ok(())
    }

    /// Get next node ID
    fn next_node_id(&mut self) -> CPGNodeId {
        let id = CPGNodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    /// Get next edge ID
    fn next_edge_id(&mut self) -> CPGEdgeId {
        let id = CPGEdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpg_builder_creation() {
        let builder = CPGBuilder::new();
        assert_eq!(builder.next_node_id, 0);
    }
}
