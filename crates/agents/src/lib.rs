//! # agents — ContentEncoder · LearningLoop · SecurityGate · LeoAI · Chief · Worker · Skill
//!
//! L0 instincts + Agent hierarchy:
//!   AAM [tier 0] — stateless (trong memory crate)
//!   LeoAI · Chief [tier 1] — orchestrators
//!   Worker [tier 2] — HomeOS thu nhỏ tại thiết bị
//!   Skill — stateless functions (QT4: 1 Skill = 1 trách nhiệm)
//!
//! ## Module Groups
//!
//! - [`hierarchy`] — Agent tiers: LeoAI, Chief, Worker
//! - [`pipeline`]  — Processing: encoder, learning, gate, book reader
//! - [`skills`]    — Stateless capabilities: instincts, domain skills

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

/// Agent hierarchy: LeoAI (tier 1), Chief (tier 1), Worker (tier 2)
pub mod hierarchy;
/// Processing pipeline: encoder, learning, security gate, book reader
pub mod pipeline;
/// Skills: instincts + domain specializations
pub mod skills;

// Re-exports for backward compatibility
pub use hierarchy::chief;
pub use hierarchy::leo;
pub use hierarchy::worker;
pub use pipeline::book;
pub use pipeline::encoder;
pub use pipeline::gate;
pub use pipeline::learning;
pub use skills::domain_skills;
pub use skills::instinct;
pub use skills::skill;
