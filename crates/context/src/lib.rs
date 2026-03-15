//! # context — ContextEngine
//!
//! f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
//! x    = đạo hàm cảm xúc toàn bộ cuộc trò chuyện

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod emotion;
pub mod intent;
pub mod phrase;
pub mod curve;
pub mod snapshot;
pub mod engine;
pub mod context;
pub mod infer;
pub mod fusion;
