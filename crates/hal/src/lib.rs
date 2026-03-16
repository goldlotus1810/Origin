//! # hal — Hardware Abstraction Layer
//!
//! Platform-agnostic traits cho HomeOS chạy trên mọi kiến trúc:
//!   x86/x64 (PC/Server), ARM (di động/SoC), RISC-V (mở/tùy biến),
//!   ESP32/RP2040 (embedded MCU), WASM (browser).
//!
//! Nguyên tắc:
//!   - Trait = interface bất biến
//!   - Impl = platform-specific (mock cho test)
//!   - Worker gọi HAL trait — không biết platform
//!   - no_std compatible — chạy trên bare-metal
//!
//! Kiến trúc chipset:
//!   Traditional (Northbridge/Southbridge) → PCH (single-chip) → SoC (all-in-one)
//!   HAL abstract tất cả — Worker không cần biết chipset layout

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod arch;
pub mod platform;
pub mod probe;
pub mod security;
pub mod driver;
pub mod tier;
pub mod ffi;

// Re-export core types
pub use arch::{Architecture, ChipsetLayout, CpuInfo, MemoryInfo, PlatformProfile};
pub use platform::{HalPlatform, PlatformCapability, DeviceDescriptor, DeviceType, DeviceStatus, MockPlatform};
pub use probe::{SystemProbe, ProbeResult, ProbeStatus, VulnerabilityReport, VulnerabilitySeverity};
pub use security::{SecurityScanner, ProcessInfo, NetworkConnection, ConnectionStatus, ThreatLevel};
pub use driver::{InputEvent, DisplayInfo, DisplayKind, AudioInfo};
pub use tier::{HardwareTier, TierConfig};
pub use ffi::{PlatformBridge, FfiResponse, FfiConfig};
