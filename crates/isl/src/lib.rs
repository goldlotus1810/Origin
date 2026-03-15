//! # isl — ISL Address + Message System
//!
//! ISL = ngôn ngữ địa chỉ của HomeOS
//!
//! ISLAddress: 4 bytes (layer/group/subgroup/index)
//! ISLMessage: 12 bytes base — nhỏ hơn JSON 95.7%
//! ISLCodec:   encode/decode + AES-256-GCM ready
//!
//! Spec từ ARCHITECTURE.md VIII:
//!   "Cùng concept 'bay' = 1 ISL address [L][G][S][I]"
//!   "Worker gửi molecular chain — không raw data"

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod address;
pub mod message;
pub mod codec;
pub mod queue;
