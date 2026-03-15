//! # agents — ContentEncoder · LearningLoop · SecurityGate · LeoAI · Chief · Worker · Skill
//!
//! L0 instincts + Agent hierarchy:
//!   AAM [tier 0] — stateless (trong memory crate)
//!   LeoAI · Chief [tier 1] — orchestrators
//!   Worker [tier 2] — HomeOS thu nhỏ tại thiết bị
//!   Skill — stateless functions (QT4: 1 Skill = 1 trách nhiệm)
//!
//! Mọi input → MolecularChain — cùng 1 format.

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod encoder;
pub mod learning;
pub mod book;
pub mod gate;
pub mod skill;
pub mod worker;
pub mod chief;
pub mod leo;
pub mod instinct;
