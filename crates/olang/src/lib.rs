#![allow(missing_docs)]
//! # olang — Core types: MolecularChain, LCA, Registry
//!
//! Không có presets. Không có hardcode.
//! Mọi chain từ UCD lookup hoặc LCA.
//!
//! ## Module Groups
//!
//! - [`lang`]    — Input: lexer, parser, AST, semantic analysis
//! - [`exec`]    — Logic: IR, VM, compiler, optimizer
//! - [`mol`]     — Core: Molecule, MolecularChain, LCA, encoder
//! - [`storage`] — Persistence: reader, writer, registry, knowtree
//! - [`crypto`]  — Security: SHA, Ed25519, AES-GCM, QR signing
//! - [`math`]    — Symbolic math + adaptive constants
//! - [`nlp`]     — Linguistic modifiers
//! - [`system`]  — Boot sequence, self-model

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

// ── Module Groups ────────────────────────────────────────────────────────────

/// Input layer: lexer → parser → AST → semantic validation
pub mod lang;

/// Execution layer: IR opcodes → VM → compiler → optimizer
pub mod exec;

/// Molecular foundation: 5-byte Molecule, chain, LCA, encoder
pub mod mol;

/// Persistence: append-only files, registry, compact storage, knowledge tree
pub mod storage;

/// Cryptography: SHA-256/512, Ed25519, AES-256-GCM, QR signing
pub mod crypto;

/// Symbolic math engine + adaptive precision constants
pub mod math;

/// Natural language processing: linguistic modifiers
pub mod nlp;

/// System: boot sequence, self-model
pub mod system;

// ── Re-exports (backward compatibility) ──────────────────────────────────────
// External crates can still use `olang::molecular`, `olang::vm`, etc.

/// Lexer/Token
pub use lang::alphabet;
/// Semantic analysis (AST → IR)
pub use lang::semantic;
/// Parser (text → AST)
pub use lang::syntax;

/// IR opcodes
pub use exec::ir;
/// Stack-based VM
pub use exec::vm;
/// Compiler (IR → C/Rust/WASM)
pub use exec::compiler;
/// IR optimizer
pub use exec::optimize;

/// 5-byte Molecule + MolecularChain
pub use mol::molecular;
/// Codepoint → MolecularChain encoder
pub use mol::encoder;
/// Weighted LCA engine
pub use mol::lca;
/// FNV-1a hash utilities
pub use mol::hash;
/// Separator classification
pub use mol::separator;

/// Append-only binary reader
pub use storage::reader;
/// Append-only binary writer
pub use storage::writer;
/// Node registry ledger
pub use storage::registry;
/// Event log
pub use storage::log;
/// L2-Ln compact storage
pub use storage::compact;
/// Knowledge tree
pub use storage::knowtree;
/// Device clone
pub use storage::clone;

/// SHA-256
pub use crypto::sha256;
/// SHA-512
pub use crypto::sha512;
/// Ed25519 signatures
pub use crypto::ed25519;
/// AES-256-GCM encryption
pub use crypto::aes256gcm;
/// QR signing + supersession
pub use crypto::qr;

// math::solver, math::constants, math::precision are accessible via `olang::math::*`
// For backward compat: `olang::constants` and `olang::precision` still work.
/// Adaptive precision constants
pub use math::constants;
/// Precision configuration
pub use math::precision;

/// Linguistic modifiers
pub use nlp::ling;

/// Boot sequence
pub use system::startup;
/// Self-awareness model
pub use system::self_model;
