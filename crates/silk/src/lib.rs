//! # silk — 3-Layer Silk Architecture
//!
//! 1. **Implicit** (SilkIndex): 37 kênh 5D — 0 bytes edges.
//!    Silk = hệ quả toán học của không gian 5D.
//!    Khi 2 node chia sẻ base value → Silk TỰ TỒN TẠI.
//!
//! 2. **Learned** (HebbianLink): 19 bytes/link.
//!    Hebbian = PHÁT HIỆN kết nối implicit, không TẠO mới.
//!    Emotion nằm trong V+A của node, không trên edge.
//!
//! 3. **Structural** (SilkEdge): parent pointers, backward compat.
//!
//! "Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

#![no_std]
#![allow(missing_docs)]

extern crate alloc;

pub mod edge;
pub mod graph;
pub mod hebbian;
pub mod index;
pub mod walk;

// Re-export core types
pub use graph::{MolSummary, SilkNeighbor};
pub use index::{ImplicitSilk, SilkDim, SilkIndex};
