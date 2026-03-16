#![allow(missing_docs)]
//! # olang — Core types: MolecularChain, LCA, Registry
//!
//! Không có presets. Không có hardcode.
//! Mọi chain từ UCD lookup hoặc LCA.

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

pub mod clone;
pub mod compact;
pub mod compiler;
pub mod encoder;
pub mod hash;
pub mod ir;
pub mod knowtree;
pub mod lca;
pub mod ling;
pub mod log;
pub mod molecular;
pub mod qr;
pub mod reader;
pub mod registry;
pub mod self_model;
pub mod separator;
pub mod startup;
pub mod vm;
pub mod writer;
