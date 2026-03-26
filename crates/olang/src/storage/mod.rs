//! Persistence layer: append-only binary files, registry, knowledge tree.
//!
//! origin.olang reader/writer, Registry ledger, compact L2-Ln storage.

pub mod clone;
pub mod compact;
pub mod knowtree;
pub mod log;
pub mod reader;
pub mod registry;
pub mod writer;
