//! # vsdf — Volumetric SDF + FFR
//!
//! 18 SDF primitives → MolecularChain tọa độ.
//! FFR (Fibonacci Fractal Representation): xoắn ốc Fibonacci
//! trong không gian 5 chiều → địa chỉ vật lý duy nhất.

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod delta;
pub mod ffr;
pub mod fit;
pub mod occlusion;
pub mod physics;
pub mod scene;
pub mod sdf;
pub mod spline;
pub mod vector;
