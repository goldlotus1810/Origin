#![allow(missing_docs)]
//! # olang — Core types: MolecularChain, LCA, Registry
//!
//! Không có presets. Không có hardcode.
//! Mọi chain từ UCD lookup hoặc LCA.

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

pub mod hash;
pub mod molecular;
pub mod lca;
pub mod encoder;
pub mod registry;
pub mod log;
pub mod writer;
pub mod reader;
pub mod qr;
pub mod ir;
pub mod vm;
pub mod startup;
pub mod clone;
pub mod self_model;
pub mod separator;
pub mod ling;
pub mod compiler;
pub mod compact;
pub mod knowtree;
