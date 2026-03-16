//! # runtime — HomeRuntime + ○{} parser
//!
//! ○(∅) == ○ — tự boot từ hư không.
//! process_one(input) → output — mọi input qua pipeline.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod concurrency;
mod emotion_tests;
pub mod error;
pub mod metrics;
pub mod origin;
pub mod parser;
pub mod response_template;
