//! openapi-php — OpenAPI 3.x → PHP code generator.
//!
//! Pipeline: `parser` (YAML/JSON → raw types) → `parser::resolve` (→ IR) →
//! `generator` (IR → PHP files via minijinja templates).
//!
//! Library surface exposed for integration tests and the binary entry point.

pub mod cli;
pub mod config;
pub mod generator;
pub mod ir;
pub mod parser;
pub mod php_utils;
