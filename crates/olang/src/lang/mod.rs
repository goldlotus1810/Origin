//! Language layer: lexer, parser, AST, semantic analysis.
//!
//! Input text → Token → AST (Stmt/Expr) → validated IR (OlangProgram).

pub mod alphabet;
pub mod semantic;
pub mod syntax;
