//! Symbol bindings and scopes

use crate::semantic::model::{ScopeId, SymbolId};
use crate::types::ByteRange;
use std::collections::HashMap;

/// A symbol binding (variable, parameter, function)
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Unique symbol identifier
    pub id: SymbolId,
    
    /// Symbol name
    pub name: String,
    
    /// Source location where symbol is defined
    pub source_range: ByteRange,
    
    /// Scope this symbol belongs to
    pub scope: ScopeId,
    
    /// Symbol kind
    pub kind: SymbolKind,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Function definition
    Function,
    
    /// Function parameter
    Parameter,
    
    /// Local variable
    Variable,
    
    /// Constant
    Constant,
}

/// Lexical scope (file, function, or block)
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique scope identifier
    pub id: ScopeId,
    
    /// Parent scope (None for file scope)
    pub parent: Option<ScopeId>,
    
    /// Scope kind
    pub kind: ScopeKind,
    
    /// Symbol name → Symbol ID
    bindings: HashMap<String, SymbolId>,
}

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// File/module scope
    File,
    
    /// Function scope
    Function,
    
    /// Block scope (within function)
    Block,
}

impl Scope {
    /// Create a new scope
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            parent,
            kind,
            bindings: HashMap::new(),
        }
    }

    /// Add a binding to this scope
    pub fn add_binding(&mut self, name: String, symbol_id: SymbolId) {
        self.bindings.insert(name, symbol_id);
    }

    /// Look up a symbol in this scope (does not search parent scopes)
    pub fn get_local(&self, name: &str) -> Option<SymbolId> {
        self.bindings.get(name).copied()
    }

    /// Get all bindings in this scope
    pub fn bindings(&self) -> &HashMap<String, SymbolId> {
        &self.bindings
    }
}
