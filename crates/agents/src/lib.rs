//! # agents — ContentEncoder · LearningLoop · BookReader · SecurityGate
//!
//! Bản năng L0: kích hoạt tự động khi có bất kỳ input nào.
//! Mọi input → MolecularChain — cùng 1 format.

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod encoder;
pub mod learning;
pub mod book;
pub mod gate;
pub mod worker;
