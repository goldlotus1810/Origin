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
pub mod driver;
pub mod ffi;
pub mod platform;
pub mod probe;
pub mod security;
pub mod tier;

// Re-export core types
pub use arch::{Architecture, ChipsetLayout, CpuInfo, MemoryInfo, PlatformProfile};
pub use driver::{AudioInfo, DisplayInfo, DisplayKind, InputEvent};
pub use ffi::{FfiConfig, FfiResponse, PlatformBridge};
pub use platform::{
    DeviceDescriptor, DeviceResult, DeviceStatus, DeviceType, HalPlatform, MockPlatform,
    PlatformCapability,
};
pub use probe::{
    ProbeResult, ProbeStatus, SystemProbe, VulnerabilityReport, VulnerabilitySeverity,
};
pub use security::{
    ConnectionStatus, NetworkConnection, ProcessInfo, SecurityScanner, ThreatLevel,
};
pub use tier::{HardwareTier, TierConfig};
