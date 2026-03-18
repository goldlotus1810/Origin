//! Symbolic math engine + adaptive precision constants.
//!
//! Solve, derive, integrate, simplify. Constants π, e, φ computed per tier.

pub mod constants;
pub mod precision;
pub mod solver;

// Re-export solver contents at math:: level for backward compat
// (external code uses `olang::math::process_math_command`)
pub use solver::*;
