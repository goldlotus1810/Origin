#![allow(missing_docs)]
//! # olang — Core types: MolecularChain, LCA, Registry
//!
//! Không có presets. Không có hardcode.
//! Mọi chain từ UCD lookup hoặc LCA.

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

pub mod alphabet;
pub mod clone;
pub mod compact;
pub mod compiler;
pub mod constants;
pub mod encoder;
pub mod hash;
pub mod ir;
pub mod knowtree;
pub mod lca;
pub mod ling;
pub mod log;
pub mod math;
pub mod molecular;
pub mod precision;
pub mod ed25519;
pub mod qr;
pub mod sha256;
pub mod sha512;
pub mod reader;
pub mod registry;
pub mod self_model;
pub mod semantic;
pub mod separator;
pub mod startup;
pub mod syntax;
pub mod optimize;
pub mod vm;
pub mod writer;
