//! # silk — Hebbian Learning + Emotional Edges
//!
//! Con nhện = Hebbian Learning
//! Tơ       = Silk (mang màu EmotionTag)
//! Lá       = Node ở Ln-1
//!
//! Edge mang EmotionTag của khoảnh khắc co-activation.
//! Không phải edge trung lập — edge có màu cảm xúc.

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod edge;
pub mod graph;
pub mod hebbian;
pub mod walk;
