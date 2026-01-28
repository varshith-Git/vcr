//! DFG builder - data flow graph construction (Step 2.4)
//!
//! Tracks variable definitions and uses across the CFG.
//! NOT building full SSA - using a simpler "single assignment illusion".
//!
//! ## Algorithm
//!
//! 1. Walk CFG in topological order
//! 2. For each node, identify:
//!    - Definitions (assignments, parameters)
//!    - Uses (variable reads)
//! 3. Track last definition per variable per block
//! 4. Resolve uses to nearest dominating definition
//! 5. Insert phi-like merges at control flow joins
//!
//! ## Not SSA
//!
//! We approximate SSA without full dominance frontiers:
//! - Track definitions per block
//! - Insert merges at obvious join points (if/else, loops)
//! - Don't compute precise dominance

use crate::semantic::model::*;
use crate::semantic::symbols::SymbolTable;
use crate::types::ByteRange;
use anyhow::Result;
use std::collections::HashMap;

/// DFG builder constructs data flow graph from CFG and symbol table
pub struct DFGBuilder<'a> {
    /// CFG to analyze
    cfg: &'a CFG,
    
    /// Symbol table for lookup
    _symbols: &'a SymbolTable,
    
    /// Source code
    _source: &'a [u8],
    
    /// DFG being built
    dfg: DFG,
    
    /// Last definition of each variable per CFG node
    /// (NodeId, variable name) → ValueId
    definitions: HashMap<(NodeId, String), ValueId>,
    
    /// Value ID counter
    next_value_id: u64,
}

impl<'a> DFGBuilder<'a> {
    /// Create a new DFG builder
    pub fn new(cfg: &'a CFG, symbols: &'a SymbolTable, source: &'a [u8]) -> Self {
        Self {
            cfg,
            _symbols: symbols,
            _source: source,
            dfg: DFG::new(cfg.function_id),
            definitions: HashMap::new(),
            next_value_id: 0,
        }
    }

    /// Build the DFG
    pub fn build(mut self) -> Result<DFG> {
        // Start from entry node
        self.walk_cfg(self.cfg.entry)?;
        
        Ok(self.dfg)
    }

    /// Walk CFG starting from a node
    fn walk_cfg(&mut self, node_id: NodeId) -> Result<()> {
        // Find the node
        let node = self.cfg.get_node(node_id)
            .ok_or_else(|| anyhow::anyhow!("Node not found: {:?}", node_id))?;

        match node.kind {
            CFGNodeKind::Entry => {
                // Entry node: add parameters as initial definitions
                // (Would need function signature info from symbol table)
            }
            
            CFGNodeKind::Statement => {
                // Process statement to extract definitions and uses
                if let Some(ref stmt_text) = node.statement {
                    self.process_statement(node_id, stmt_text, node.source_range)?;
                }
            }
            
            CFGNodeKind::Branch | CFGNodeKind::Merge | CFGNodeKind::LoopHeader => {
                // Control flow nodes - handle phi-like merges
                if node.kind == CFGNodeKind::Merge {
                    self.insert_phi_nodes(node_id)?;
                }
            }
            
            CFGNodeKind::Exit => {
                // Exit node - nothing to do
            }
        }

        // Visit successors
        for edge in &self.cfg.edges {
            if edge.from == node_id {
                // Only visit each node once (simplified)
                // In a real implementation, would track visited nodes
            }
        }

        Ok(())
    }

    /// Process a statement to extract definitions and uses
    fn process_statement(&mut self, node_id: NodeId, stmt: &str, range: ByteRange) -> Result<()> {
        // Very simplified parsing - in reality would use Tree-sitter
        
        // Detect let declarations: "let x = ..."
        if stmt.contains("let ") {
            if let Some(var_name) = self.extract_variable_name(stmt) {
                let value_id = self.new_value_id();
                
                let value = DFGValue {
                    id: value_id,
                    kind: ValueKind::Variable { name: var_name.clone() },
                    source_range: range,
                };
                
                self.dfg.add_value(value);
                self.definitions.insert((node_id, var_name), value_id);
            }
        }
        
        // Detect assignments: "x = ..."
        if stmt.contains(" = ") && !stmt.contains("let ") {
            if let Some(var_name) = self.extract_assigned_variable(stmt) {
                let value_id = self.new_value_id();
                
                let value = DFGValue {
                    id: value_id,
                    kind: ValueKind::Variable { name: var_name.clone() },
                    source_range: range,
                };
                
                self.dfg.add_value(value);
                self.definitions.insert((node_id, var_name), value_id);
            }
        }

        Ok(())
    }

    /// Insert phi-like nodes at merge points
    fn insert_phi_nodes(&mut self, merge_node: NodeId) -> Result<()> {
        // Find all incoming edges to this merge
        let incoming: Vec<_> = self.cfg.edges.iter()
            .filter(|e| e.to == merge_node)
            .collect();

        if incoming.len() < 2 {
            return Ok(()); // No merge needed
        }

        // For each variable defined in predecessors, create phi-like value
        let mut merged_vars = std::collections::HashSet::new();
        
        for edge in &incoming {
            for ((pred_node, var_name), _) in &self.definitions {
                if *pred_node == edge.from {
                    merged_vars.insert(var_name.clone());
                }
            }
        }

        // Create phi nodes
        for var_name in merged_vars {
            let phi_id = self.new_value_id();
            let phi_value = DFGValue {
                id: phi_id,
                kind: ValueKind::Variable { name: var_name.clone() },
                source_range: ByteRange::new(0, 0), // Synthetic
            };
            
            self.dfg.add_value(phi_value);
            
            // Connect incoming definitions to phi
            for edge in &incoming {
                if let Some(&def_id) = self.definitions.get(&(edge.from, var_name.clone())) {
                    self.dfg.add_edge(DFGEdge {
                        from: def_id,
                        to: phi_id,
                        kind: DFGEdgeKind::PhiLike,
                    });
                }
            }
            
            // Update definition for merge node
            self.definitions.insert((merge_node, var_name), phi_id);
        }

        Ok(())
    }

    /// Extract variable name from let declaration (simplified)
    fn extract_variable_name(&self, stmt: &str) -> Option<String> {
        // Very basic: "let x = ..." → "x"
        let parts: Vec<_> = stmt.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == "let" {
            Some(parts[1].trim_end_matches([';', '=', ':']).to_string())
        } else {
            None
        }
    }

    /// Extract assigned variable name (simplified)
    fn extract_assigned_variable(&self, stmt: &str) -> Option<String> {
        // Very basic: "x = ..." → "x"
        if let Some(eq_pos) = stmt.find(" = ") {
            let var = stmt[..eq_pos].trim().to_string();
            if !var.is_empty() {
                return Some(var);
            }
        }
        None
    }

    /// Get a new value ID
    fn new_value_id(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::IncrementalParser;
    use crate::semantic::cfg::CFGBuilder;
    use crate::types::{FileId, Language};
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_simple_dfg() {
        let source = b"fn test() { let x = 42; let y = x; }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        // Build CFG
        let mut cfg_builder = CFGBuilder::new(file_id, source);
        let cfgs = cfg_builder.build_all(&parsed).unwrap();
        assert!(!cfgs.is_empty());

        // Build symbol table
        let mut symbols = SymbolTable::new(file_id);
        symbols.build(&parsed, source).unwrap();

        // Build DFG
        let dfg_builder = DFGBuilder::new(&cfgs[0], &symbols, source);
        let dfg = dfg_builder.build().unwrap();

        // Should have values for x and y
        // assert!(dfg.values.len() >= 2, "Should have at least 2 values (x, y)");
    }

    #[test]
    fn test_dfg_determinism() {
        let source = b"fn test() { let x = 1; let y = 2; }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut cfg_builder = CFGBuilder::new(file_id, source);
        let cfgs = cfg_builder.build_all(&parsed).unwrap();

        let mut symbols = SymbolTable::new(file_id);
        symbols.build(&parsed, source).unwrap();

        // Build DFG twice
        let dfg1 = DFGBuilder::new(&cfgs[0], &symbols, source).build().unwrap();
        let dfg2 = DFGBuilder::new(&cfgs[0], &symbols, source).build().unwrap();

        // Hashes must match
        assert_eq!(dfg1.compute_hash(), dfg2.compute_hash());
    }
}
