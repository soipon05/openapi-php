// src/lib.rs
//
// Exposes the library surface so that:
//   - Integration tests under tests/ can reach `openapi_php::parser`.
//   - The binary target (src/main.rs) can delegate to `openapi_php::cli`.

pub mod cli;
pub mod config;
pub mod generator;
pub mod ir;
pub mod parser;
