//! Intermediate Representation (IR) — the typed AST between parsing and code generation.
//!
//! Produced by [`crate::parser::resolve`], consumed by [`crate::generator`] backends.
//! All `$ref` pointers are fully resolved at this stage (except circular refs
//! which appear as [`ResolvedSchema::Ref`]).

pub mod endpoint;
pub mod schema;

pub use endpoint::*;
pub use schema::*;
