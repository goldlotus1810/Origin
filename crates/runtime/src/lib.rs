//! # runtime — HomeRuntime + ○{} parser
//!
//! ○(∅) == ○ — tự boot từ hư không.
//! process_one(input) → output — mọi input qua pipeline.

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod origin;
pub mod parser;
pub mod response_template;
