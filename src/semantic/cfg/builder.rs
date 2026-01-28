//! CFG builder - deterministic construction from Tree-sitter AST (Step 2.2)
//!
//! Walks Tree-sitter parse trees in deterministic order and emits CFG nodes and edges.
//!
//! ## Algorithm
//!
//! 1. Walk AST in parse tree order (never reorder)
//! 2. For each function:
//!    - Create Entry and Exit nodes
//!    - Walk function body sequentially
//!    - Identify control constructs (if, loop, match)
//!    - Emit statement nodes
//!    - Connect edges deterministically
//! 3. Functions processed in lexical file order
//!
//! ## Determinism Guarantees
//!
//! - Node IDs assigned sequentially (never reused)
//! - Nodes emitted in parse tree order
//! - Edges added as encountered (no reordering)
//! - No parallelism, no hash maps for node storage

use crate::semantic::model::*;
use crate::types::{ByteRange, FileId, ParsedFile};
use anyhow::{Context, Result};
use tree_sitter::{Node, TreeCursor};

/// CFG builder for deterministic control flow graph construction
pub struct CFGBuilder<'a> {
    /// File being analyzed
    file_id: FileId,
    
    /// Source code bytes
    source: &'a [u8],
    
    /// Current function being processed
    current_function: Option<FunctionId>,
    
    /// CFG being built
    current_cfg: Option<CFG>,
    
    /// Node ID counter (monotonically increasing)
    next_node_id: u64,
    
    /// Function ID counter
    next_function_id: u64,
}

impl<'a> CFGBuilder<'a> {
    /// Create a new CFG builder
    pub fn new(file_id: FileId, source: &'a [u8]) -> Self {
        Self {
            file_id,
            source,
            current_function: None,
            current_cfg: None,
            next_node_id: 0,
            next_function_id: 0,
        }
    }

    /// Build CFGs for all functions in a parsed file
    pub fn build_all(&mut self, parsed: &ParsedFile) -> Result<Vec<CFG>> {
        let mut cfgs = Vec::new();
        
        // Walk the tree to find all function declarations
        let root = parsed.tree.root_node();
        let mut cursor = root.walk();
        
        // Process functions in parse tree order
        self.visit_node_for_functions(&root, &mut cursor, &mut cfgs)?;
        
        Ok(cfgs)
    }

    /// Visit a node looking for function declarations
    fn visit_node_for_functions(
        &mut self,
        node: &Node,
        cursor: &mut TreeCursor,
        cfgs: &mut Vec<CFG>,
    ) -> Result<()> {
        match node.kind() {
            "function_item" => {
                // Build CFG for this function
                if let Ok(cfg) = self.build_function_cfg(node) {
                    cfgs.push(cfg);
                }
            }
            _ => {
                // Recursively visit children in order
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        self.visit_node_for_functions(&child, cursor, cfgs)?;
                        
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        
        Ok(())
    }

    /// Build CFG for a single function
    fn build_function_cfg(&mut self, function_node: &Node) -> Result<CFG> {
        // Assign function ID
        let function_id = FunctionId(self.next_function_id);
        self.next_function_id += 1;
        self.current_function = Some(function_id);
        
        // Create entry and exit nodes
        let entry_id = self.new_node_id();
        let exit_id = self.new_node_id();
        
        let entry_range = self.node_range(function_node);
        
        let entry_node = CFGNode {
            id: entry_id,
            kind: CFGNodeKind::Entry,
            source_range: entry_range,
            statement: Some("<entry>".to_string()),
        };
        
        let exit_node = CFGNode {
            id: exit_id,
            kind: CFGNodeKind::Exit,
            source_range: entry_range,
            statement: Some("<exit>".to_string()),
        };
        
        // Initialize CFG
        let mut cfg = CFG::new(function_id, self.file_id, entry_id, exit_id);
        cfg.add_node(entry_node);
        cfg.add_node(exit_node);
        
        self.current_cfg = Some(cfg);
        
        // Find function body
        if let Some(body) = function_node.child_by_field_name("body") {
            // Walk the function body
            let last_node = self.walk_block(&body, entry_id)?;
            
            // Connect last statement to exit
            if let Some(ref mut cfg) = self.current_cfg {
                cfg.add_edge(CFGEdge {
                    from: last_node,
                    to: exit_id,
                    kind: CFGEdgeKind::Normal,
                });
            }
        }
        
        // Return the built CFG
        self.current_cfg.take().context("CFG not initialized")
    }

    /// Walk a block of statements
    fn walk_block(&mut self, block_node: &Node, predecessor: NodeId) -> Result<NodeId> {
        let mut current = predecessor;
        
        // Handle block expression specifically
        if block_node.kind() == "block" {
            // Iterate through children in order
            let mut cursor = block_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    
                    // Process each statement (skip braces)
                    if child.kind() != "{" && child.kind() != "}" {
                        if self.is_statement(&child) {
                            current = self.walk_statement(&child, current)?;
                        }
                    }
                    
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        } else {
            // For non-block nodes (single expressions), treat as statement
            if self.is_statement(block_node) {
                current = self.walk_statement(block_node, current)?;
            }
        }
        
        Ok(current)
    }

    /// Walk a single statement
    fn walk_statement(&mut self, stmt_node: &Node, predecessor: NodeId) -> Result<NodeId> {
        // Handle expression_statement wrapper
        let actual_node = if stmt_node.kind() == "expression_statement" {
            // Unwrap to get the actual expression
            if let Some(child) = stmt_node.child(0) {
                child
            } else {
                *stmt_node
            }
        } else {
            *stmt_node
        };
        
        match actual_node.kind() {
            "if_expression" => self.build_if(&actual_node, predecessor),
            "while_expression" => self.build_loop(&actual_node, predecessor, true),
            "loop_expression" => self.build_loop(&actual_node, predecessor, false),
            "match_expression" => self.build_match(&actual_node, predecessor),
            _ => self.build_simple_statement(stmt_node, predecessor),
        }
    }

    /// Build CFG for if expression
    fn build_if(&mut self, if_node: &Node, predecessor: NodeId) -> Result<NodeId> {
        // Create branch node
        let branch_id = self.new_node_id();
        let branch_node = CFGNode {
            id: branch_id,
            kind: CFGNodeKind::Branch,
            source_range: self.node_range(if_node),
            statement: Some(self.node_text(if_node).chars().take(50).collect()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(branch_node);
            cfg.add_edge(CFGEdge {
                from: predecessor,
                to: branch_id,
                kind: CFGEdgeKind::Normal,
            });
        }
        
        // Create merge node
        let merge_id = self.new_node_id();
        let merge_node = CFGNode {
            id: merge_id,
            kind: CFGNodeKind::Merge,
            source_range: self.node_range(if_node),
            statement: Some("<merge>".to_string()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(merge_node);
        }
        
        // Process then branch
        if let Some(then_branch) = if_node.child_by_field_name("consequence") {
            let then_last = self.walk_block(&then_branch, branch_id)?;
            
            if let Some(ref mut cfg) = self.current_cfg {
                // True edge from branch to then block (walk_block handles internal connections)
                cfg.add_edge(CFGEdge {
                    from: then_last,
                    to: merge_id,
                    kind: CFGEdgeKind::Normal,
                });
            }
        }
        
        // Process else branch (if present)
        if let Some(else_branch) = if_node.child_by_field_name("alternative") {
            let else_last = self.walk_block(&else_branch, branch_id)?;
            
            if let Some(ref mut cfg) = self.current_cfg {
                cfg.add_edge(CFGEdge {
                    from: else_last,
                    to: merge_id,
                    kind: CFGEdgeKind::Normal,
                });
            }
        } else {
            // No else branch - false edge goes directly to merge
            if let Some(ref mut cfg) = self.current_cfg {
                cfg.add_edge(CFGEdge {
                    from: branch_id,
                    to: merge_id,
                    kind: CFGEdgeKind::False,
                });
            }
        }
        
        Ok(merge_id)
    }

    /// Build CFG for loop (while or infinite loop)
    fn build_loop(&mut self, loop_node: &Node, predecessor: NodeId, has_condition: bool) -> Result<NodeId> {
        // Create loop header
        let header_id = self.new_node_id();
        let header_node = CFGNode {
            id: header_id,
            kind: CFGNodeKind::LoopHeader,
            source_range: self.node_range(loop_node),
            statement: Some(self.node_text(loop_node).chars().take(50).collect()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(header_node);
            cfg.add_edge(CFGEdge {
                from: predecessor,
                to: header_id,
                kind: CFGEdgeKind::Normal,
            });
        }
        
        // Create merge node (after loop)
        let merge_id = self.new_node_id();
        let merge_node = CFGNode {
            id: merge_id,
            kind: CFGNodeKind::Merge,
            source_range: self.node_range(loop_node),
            statement: Some("<merge>".to_string()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(merge_node);
        }
        
        // Process loop body
        if let Some(body) = loop_node.child_by_field_name("body") {
            let body_last = self.walk_block(&body, header_id)?;
            
            if let Some(ref mut cfg) = self.current_cfg {
                // Body loops back to header
                cfg.add_edge(CFGEdge {
                    from: body_last,
                    to: header_id,
                    kind: CFGEdgeKind::Continue,
                });
                
                // Exit condition (if exists) goes to merge
                if has_condition {
                    cfg.add_edge(CFGEdge {
                        from: header_id,
                        to: merge_id,
                        kind: CFGEdgeKind::Break,
                    });
                }
            }
        }
        
        Ok(merge_id)
    }

    /// Build CFG for match expression
    fn build_match(&mut self, match_node: &Node, predecessor: NodeId) -> Result<NodeId> {
        // Create branch node for match
        let branch_id = self.new_node_id();
        let branch_node = CFGNode {
            id: branch_id,
            kind: CFGNodeKind::Branch,
            source_range: self.node_range(match_node),
            statement: Some("match".to_string()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(branch_node);
            cfg.add_edge(CFGEdge {
                from: predecessor,
                to: branch_id,
                kind: CFGEdgeKind::Normal,
            });
        }
        
        // Create merge node
        let merge_id = self.new_node_id();
        let merge_node = CFGNode {
            id: merge_id,
            kind: CFGNodeKind::Merge,
            source_range: self.node_range(match_node),
            statement: Some("<merge>".to_string()),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(merge_node);
        }
        
        // Process each match arm in order
        if let Some(body) = match_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "match_arm" {
                        if let Some(arm_body) = child.child_by_field_name("value") {
                            let arm_last = self.walk_block(&arm_body, branch_id)?;
                            
                            if let Some(ref mut cfg) = self.current_cfg {
                                cfg.add_edge(CFGEdge {
                                    from: arm_last,
                                    to: merge_id,
                                    kind: CFGEdgeKind::Normal,
                                });
                            }
                        }
                    }
                    
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        
        Ok(merge_id)
    }

    /// Build CFG for simple statement (assignment, call, etc.)
    fn build_simple_statement(&mut self, stmt_node: &Node, predecessor: NodeId) -> Result<NodeId> {
        let stmt_id = self.new_node_id();
        let stmt_node_cfg = CFGNode {
            id: stmt_id,
            kind: CFGNodeKind::Statement,
            source_range: self.node_range(stmt_node),
            statement: Some(self.node_text(stmt_node)),
        };
        
        if let Some(ref mut cfg) = self.current_cfg {
            cfg.add_node(stmt_node_cfg);
            cfg.add_edge(CFGEdge {
                from: predecessor,
                to: stmt_id,
                kind: CFGEdgeKind::Normal,
            });
        }
        
        Ok(stmt_id)
    }

    /// Check if a node represents a statement
    fn is_statement(&self, node: &Node) -> bool {
        match node.kind() {
            // Declarations
            "let_declaration" => true,
            // Statements
            "expression_statement" => true,
            // Control flow
            "if_expression" | "while_expression" | "loop_expression" | 
            "for_expression" | "match_expression" => true,
            // Jump statements
            "return_expression" | "break_expression" | "continue_expression" => true,
            // Default: treat unknown as potential statement
            _ => !matches!(node.kind(), "{" | "}" | "(" | ")" | "," | ";"),
        }
    }

    /// Get a new node ID
    fn new_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    /// Get byte range for a node
    fn node_range(&self, node: &Node) -> ByteRange {
        ByteRange::new(node.start_byte(), node.end_byte())
    }

    /// Get text content of a node (truncated)
    fn node_text(&self, node: &Node) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        let bytes = &self.source[start..end];
        
        String::from_utf8_lossy(bytes)
            .chars()
            .filter(|c| !c.is_whitespace() || *c == ' ')
            .take(100)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::IncrementalParser;
    use crate::types::Language;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_simple_function_cfg() {
        let source = b"fn test() { let x = 42; }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut builder = CFGBuilder::new(file_id, source);
        let cfgs = builder.build_all(&parsed).unwrap();

        assert_eq!(cfgs.len(), 1, "Should have one function");
        
        let cfg = &cfgs[0];
        assert!(cfg.nodes.len() >= 3, "Should have entry, exit, and at least one statement");
        
        // Verify entry and exit exist
        assert_eq!(cfg.nodes[0].kind, CFGNodeKind::Entry);
        assert_eq!(cfg.nodes[1].kind, CFGNodeKind::Exit);
    }

    #[test]
    fn test_if_expression_cfg() {
        let source = b"fn test() { if true { let x = 1; } else { let y = 2; } }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut builder = CFGBuilder::new(file_id, source);
        let cfgs = builder.build_all(&parsed).unwrap();

        assert_eq!(cfgs.len(), 1);
        
        let cfg = &cfgs[0];
        
        // Should have: entry, exit, branch, merge, and statements in both branches
        let has_branch = cfg.nodes.iter().any(|n| n.kind == CFGNodeKind::Branch);
        let has_merge = cfg.nodes.iter().any(|n| n.kind == CFGNodeKind::Merge);
        
        assert!(has_branch, "Should have branch node");
        assert!(has_merge, "Should have merge node");
    }

    #[test]
    fn test_loop_cfg() {
        let source = b"fn test() { loop { break; } }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut builder = CFGBuilder::new(file_id, source);
        let cfgs = builder.build_all(&parsed).unwrap();

        assert_eq!(cfgs.len(), 1);
        
        let cfg = &cfgs[0];
        
        let has_loop_header = cfg.nodes.iter().any(|n| n.kind == CFGNodeKind::LoopHeader);
        assert!(has_loop_header, "Should have loop header node");
    }

    #[test]
    fn test_cfg_determinism() {
        let source = b"fn test() { let x = 1; let y = 2; }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        // Build CFG twice
        let mut builder1 = CFGBuilder::new(file_id, source);
        let cfgs1 = builder1.build_all(&parsed).unwrap();

        let mut builder2 = CFGBuilder::new(file_id, source);
        let cfgs2 = builder2.build_all(&parsed).unwrap();

        // Hashes must be identical
        assert_eq!(cfgs1[0].compute_hash(), cfgs2[0].compute_hash());
    }
}
