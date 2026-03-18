//! Emotion state: V/A/D/I dimensions, conversation curve, snapshots.

pub mod context;
pub mod curve;
pub mod affect;
pub mod snapshot;

// Re-export affect.rs public items at this level
// so `crate::emotion::IntentKind` etc. still resolve
pub use affect::*;
