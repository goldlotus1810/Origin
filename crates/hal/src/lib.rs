//! # hal — Hardware Abstraction Layer
//!
//! Platform-agnostic traits cho HomeOS chạy trên mọi kiến trúc:
//!   x86/x64 (PC/Server), ARM (di động/SoC), RISC-V (mở/tùy biến),
//!   ESP32/RP2040 (embedded MCU), WASM (browser).
//!
//! ## Module Groups
//!
//! - [`detect`]    — Architecture detection, probing, tier classification, security
//! - [`interface`] — Platform traits, FFI bridge, device drivers

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/// Detection: architecture, probing, tier, security
pub mod detect;
/// Hardware interfaces: platform, FFI, drivers
pub mod interface;

// Re-exports for backward compatibility
pub use detect::arch;
pub use detect::probe;
pub use detect::security;
pub use detect::tier;
pub use interface::driver;
pub use interface::ffi;
pub use interface::platform;

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
