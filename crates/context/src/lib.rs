//! # context — ContextEngine
//!
//! f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
//! x    = đạo hàm cảm xúc toàn bộ cuộc trò chuyện

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod context;
pub mod curve;
pub mod emotion;
pub mod engine;
pub mod fusion;
pub mod infer;
pub mod intent;
pub mod modality;
pub mod phrase;
pub mod snapshot;
pub mod template;
pub mod word_guide;
