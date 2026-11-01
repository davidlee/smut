//! smut — a permissive notebook for an agent's mutable entity graph.
//!
//! The library is the whole engine: the binary in `src/main.rs` only routes a
//! command line and writes what this crate returns. Each module is exposed here
//! by the phase that builds it.

#![forbid(unsafe_code)]
pub mod declare;
pub mod entity;
pub mod error;
pub mod id;
pub mod input;
pub mod learn;
pub mod numeric;
pub mod order;
pub mod render_markdown;
pub mod select;
pub mod spec;
pub mod store;
