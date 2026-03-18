//! # context — ContextEngine
//!
//! f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
//! x    = đạo hàm cảm xúc toàn bộ cuộc trò chuyện
//!
//! ## Module Groups
//!
//! - [`emotion`]  — Emotion state: V/A/D/I, curve, snapshots
//! - [`analysis`] — Analysis: inference, intent, engine, fusion
//! - [`language`] — Language: phrases, word lexicon, templates, modality

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

/// Emotion state: dimensions, conversation curve, context, snapshots
pub mod emotion;
/// Analysis pipeline: inference, intent, engine, cross-modal fusion
pub mod analysis;
/// Language layer: phrases, word lexicon, templates, modality
pub mod language;

// Re-exports for backward compatibility
pub use emotion::context;
pub use emotion::curve;
pub use emotion::affect as emotion_mod;
pub use emotion::snapshot;
pub use analysis::engine;
pub use analysis::fusion;
pub use analysis::infer;
pub use analysis::intent;
pub use language::modality;
pub use language::phrase;
pub use language::template;
pub use language::word_guide;
