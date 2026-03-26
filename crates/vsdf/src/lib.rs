//! # vsdf — Volumetric SDF + FFR
//!
//! 18 SDF primitives → MolecularChain tọa độ.
//! FFR (Fibonacci Fractal Representation): xoắn ốc Fibonacci
//! trong không gian 5 chiều → địa chỉ vật lý duy nhất.
//!
//! ## Module Groups
//!
//! - [`shape`]    — SDF primitives, fitting, NodeBody
//! - [`render`]   — FFR rendering, scene graph, occlusion
//! - [`dynamics`] — Splines, physics, delta compression, vector math

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

/// SDF shape primitives: 18 generators, fitting, NodeBody
pub mod shape;
/// Rendering: FFR, scene graph, occlusion culling
pub mod render;
/// Dynamics: splines, physics, delta compression, vector math
pub mod dynamics;

// Re-exports for backward compatibility
pub use shape::body;
pub use shape::fit;
pub use shape::sdf;
pub use render::ffr;
pub use render::occlusion;
pub use render::scene;
pub use dynamics::delta;
pub use dynamics::physics;
pub use dynamics::spline;
pub use dynamics::vector;
