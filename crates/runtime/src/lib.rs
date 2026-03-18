//! # runtime — HomeRuntime + ○{} parser
//!
//! ○(∅) == ○ — tự boot từ hư không.
//! process_one(input) → output — mọi input qua pipeline.
//!
//! ## Module Groups
//!
//! - [`core`]     — HomeRuntime, ○{} parser, input router
//! - [`output`]   — Response rendering, error types, metrics
//! - [`pipeline`] — Concurrency patterns

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/// Core runtime: HomeRuntime, ○{} parser, router
pub mod core;
/// Output: response templates, errors, metrics
pub mod output;
/// Pipeline: concurrency
pub mod pipeline;

// Re-exports for backward compatibility
pub use core::origin;
pub use core::parser;
pub use core::router;
pub use output::error;
pub use output::metrics;
pub use output::response_template;
pub use pipeline::concurrency;
