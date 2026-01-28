//! Symbol table - lexical scoping without types (Step 2.3)
//!
//! Tracks lexical bindings across file, function, and block scopes.
//! No type information - just name → source location mappings.
//!
//! ## Scope Hierarchy
//!
//! - File scope: top-level items (functions, structs, etc.)
//! - Function scope: function parameters
//! - Block scope: local variables within blocks
//!
//! ## Immutability
//!
//! All bindings are immutable within a SemanticEpoch.
//! Changing a binding requires creating a new epoch.

pub mod table;
pub mod binding;

pub use table::SymbolTable;
pub use binding::{Symbol, Scope, SymbolKind, ScopeKind};
