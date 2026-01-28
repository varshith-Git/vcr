//! Symbol table implementation

use crate::semantic::model::{FunctionId, ScopeId, SymbolId};
use crate::semantic::symbols::binding::{Scope, ScopeKind, Symbol, SymbolKind};
use crate::types::{ByteRange, FileId, ParsedFile};
use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::Node;

/// Symbol table tracks all symbols and their scopes
pub struct SymbolTable {
    /// File being analyzed
    _file_id: FileId,
    
    /// All scopes (file, function, block)
    scopes: HashMap<ScopeId, Scope>,
    
    /// All symbols
    symbols: HashMap<SymbolId, Symbol>,
    
    /// File-level scope
    file_scope: ScopeId,
    
    /// Function ID → Function scope
    _function_scopes: HashMap<FunctionId, ScopeId>,
    
    /// Counters for ID generation
    next_scope_id: u64,
    next_symbol_id: u64,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new(file_id: FileId) -> Self {
        let file_scope_id = ScopeId(0);
        let mut scopes = HashMap::new();
        scopes.insert(
            file_scope_id,
            Scope::new(file_scope_id, ScopeKind::File, None),
        );

        Self {
            _file_id: file_id,
            scopes,
            symbols: HashMap::new(),
            file_scope: file_scope_id,
            _function_scopes: HashMap::new(),
            next_scope_id: 1,
            next_symbol_id: 0,
        }
    }

    /// Build symbol table from parsed file
    pub fn build(&mut self, parsed: &ParsedFile, source: &[u8]) -> Result<()> {
        let root = parsed.tree.root_node();
        self.visit_node(&root, self.file_scope, source)?;
        Ok(())
    }

    /// Visit a node and extract symbols
    fn visit_node(&mut self, node: &Node, current_scope: ScopeId, source: &[u8]) -> Result<()> {
        match node.kind() {
            "function_item" => {
                self.visit_function(node, current_scope, source)?;
            }
            "let_declaration" => {
                self.visit_let_declaration(node, current_scope, source)?;
            }
            "block" => {
                // Create block scope
                let block_scope = self.new_scope(ScopeKind::Block, Some(current_scope));
                
                // Visit children in block scope
                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        if child.kind() != "{" && child.kind() != "}" {
                            self.visit_node(&child, block_scope, source)?;
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
            _ => {
                // Recursively visit children
                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        self.visit_node(&child, current_scope, source)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Visit a function declaration
    fn visit_function(&mut self, node: &Node, parent_scope: ScopeId, source: &[u8]) -> Result<()> {
        // Extract function name
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.node_text(&name_node, source)
        } else {
            return Ok(());
        };

        // Add function to parent scope
        let symbol_id = self.new_symbol_id();
        let function_symbol = Symbol {
            id: symbol_id,
            name: name.clone(),
            source_range: self.node_range(node),
            scope: parent_scope,
            kind: SymbolKind::Function,
        };

        self.symbols.insert(symbol_id, function_symbol);
        if let Some(scope) = self.scopes.get_mut(&parent_scope) {
            scope.add_binding(name, symbol_id);
        }

        // Create function scope
        let function_scope = self.new_scope(ScopeKind::Function, Some(parent_scope));
        
        // Process parameters
        if let Some(params) = node.child_by_field_name("parameters") {
            self.visit_parameters(&params, function_scope, source)?;
        }

        // Process function body
        if let Some(body) = node.child_by_field_name("body") {
            self.visit_node(&body, function_scope, source)?;
        }

        Ok(())
    }

    /// Visit function parameters
    fn visit_parameters(&mut self, params_node: &Node, scope: ScopeId, source: &[u8]) -> Result<()> {
        let mut cursor = params_node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                
                if child.kind() == "parameter" {
                    // Extract parameter name
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        if let Some(name_node) = pattern.child_by_field_name("name").or(Some(pattern)) {
                            let name = self.node_text(&name_node, source);
                            
                            let symbol_id = self.new_symbol_id();
                            let param_symbol = Symbol {
                                id: symbol_id,
                                name: name.clone(),
                                source_range: self.node_range(&pattern),
                                scope,
                                kind: SymbolKind::Parameter,
                            };

                            self.symbols.insert(symbol_id, param_symbol);
                            if let Some(scope_ref) = self.scopes.get_mut(&scope) {
                                scope_ref.add_binding(name, symbol_id);
                            }
                        }
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Visit a let declaration
    fn visit_let_declaration(&mut self, node: &Node, scope: ScopeId, source: &[u8]) -> Result<()> {
        // Extract variable name
        if let Some(pattern) = node.child_by_field_name("pattern") {
            let name = if pattern.kind() == "identifier" {
                self.node_text(&pattern, source)
            } else {
                // Handle more complex patterns later
                return Ok(());
            };

            let symbol_id = self.new_symbol_id();
            let var_symbol = Symbol {
                id: symbol_id,
                name: name.clone(),
                source_range: self.node_range(node),
                scope,
                kind: SymbolKind::Variable,
            };

            self.symbols.insert(symbol_id, var_symbol);
            if let Some(scope_ref) = self.scopes.get_mut(&scope) {
                scope_ref.add_binding(name, symbol_id);
            }
        }

        Ok(())
    }

    /// Look up a symbol by name in the given scope (walks up parent scopes)
    pub fn lookup(&self, name: &str, scope: ScopeId) -> Option<&Symbol> {
        let mut current_scope = Some(scope);

        while let Some(scope_id) = current_scope {
            if let Some(scope) = self.scopes.get(&scope_id) {
                if let Some(symbol_id) = scope.get_local(name) {
                    return self.symbols.get(&symbol_id);
                }
                current_scope = scope.parent;
            } else {
                break;
            }
        }

        None
    }

    /// Get all symbols in a scope
    pub fn symbols_in_scope(&self, scope: ScopeId) -> Vec<&Symbol> {
        if let Some(scope_ref) = self.scopes.get(&scope) {
            scope_ref
                .bindings()
                .values()
                .filter_map(|id| self.symbols.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get a scope by ID
    pub fn get_scope(&self, scope_id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&scope_id)
    }

    /// Get file scope
    pub fn file_scope(&self) -> ScopeId {
        self.file_scope
    }

    /// Create a new scope
    fn new_scope(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let scope_id = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;

        let scope = Scope::new(scope_id, kind, parent);
        self.scopes.insert(scope_id, scope);

        scope_id
    }

    /// Create a new symbol ID
    fn new_symbol_id(&mut self) -> SymbolId {
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    /// Get byte range for a node
    fn node_range(&self, node: &Node) -> ByteRange {
        ByteRange::new(node.start_byte(), node.end_byte())
    }

    /// Get text content of a node
    fn node_text(&self, node: &Node, source: &[u8]) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        let bytes = &source[start..end];
        String::from_utf8_lossy(bytes).to_string()
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
    fn test_function_symbol() {
        let source = b"fn test() { }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut table = SymbolTable::new(file_id);
        table.build(&parsed, source).unwrap();

        // Should have function "test" in file scope
        let file_scope = table.file_scope();
        let symbol = table.lookup("test", file_scope).unwrap();
        
        assert_eq!(symbol.name, "test");
        assert_eq!(symbol.kind, SymbolKind::Function);
    }

    #[test]
    fn test_parameter_symbol() {
        let source = b"fn test(x: i32) { }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut table = SymbolTable::new(file_id);
        table.build(&parsed, source).unwrap();

        // Find function scope
        let file_scope = table.file_scope();
        let scopes: Vec<_> = table.scopes.values()
            .filter(|s| s.kind == ScopeKind::Function && s.parent == Some(file_scope))
            .collect();
        
        assert!(!scopes.is_empty(), "Should have function scope");
        
        let func_scope = scopes[0].id;
        let param = table.lookup("x", func_scope).unwrap();
        
        assert_eq!(param.name, "x");
        assert_eq!(param.kind, SymbolKind::Parameter);
    }

    #[test]
    fn test_local_variable() {
        let source = b"fn test() { let x = 42; }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut table = SymbolTable::new(file_id);
        table.build(&parsed, source).unwrap();

        // Find block scope
        let block_scopes: Vec<_> = table.scopes.values()
            .filter(|s| s.kind == ScopeKind::Block)
            .collect();
        
        assert!(!block_scopes.is_empty(), "Should have block scope");
        
        let block_scope = block_scopes[0].id;
        let var = table.lookup("x", block_scope).unwrap();
        
        assert_eq!(var.name, "x");
        assert_eq!(var.kind, SymbolKind::Variable);
    }

    #[test]
    fn test_scope_nesting() {
        let source = b"fn test() { let x = 1; { let y = 2; } }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), source).unwrap();

        let file_id = FileId::new(1);
        let mmap = crate::io::MmappedFile::open(temp_file.path(), file_id).unwrap();
        
        let mut parser = IncrementalParser::new(Language::Rust).unwrap();
        let parsed = parser.parse(&mmap, None).unwrap();

        let mut table = SymbolTable::new(file_id);
        table.build(&parsed, source).unwrap();

        // Should have nested block scopes
        let block_scopes: Vec<_> = table.scopes.values()
            .filter(|s| s.kind == ScopeKind::Block)
            .collect();
        
        assert!(block_scopes.len() >= 2, "Should have at least 2 block scopes");
        
        // Inner scope should be able to see outer variable
        let inner_scope = block_scopes.iter()
            .find(|s| s.bindings().contains_key("y"))
            .unwrap();
        
        let x_symbol = table.lookup("x", inner_scope.id);
        assert!(x_symbol.is_some(), "Inner scope should see outer variable 'x'");
    }
}
