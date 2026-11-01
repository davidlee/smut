//! smut — a permissive notebook for an agent's mutable entity graph.
//!
//! The library is the whole engine: the binary in `src/main.rs` only routes a
//! command line and writes what this crate returns. Each module is exposed here
//! by the phase that builds it.

#![forbid(unsafe_code)]
pub mod entity;
pub mod error;
