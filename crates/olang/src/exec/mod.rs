//! Execution layer: IR opcodes, VM, compiler, optimizer.
//!
//! OlangProgram → VM execution → VmEvent side effects.
//! Compiler: OlangProgram → C / Rust / WASM text output.

pub mod compiler;
pub mod ir;
pub mod optimize;
pub mod vm;
