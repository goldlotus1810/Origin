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

#![no_std]
extern crate alloc;

pub mod arch;
pub mod platform;
pub mod probe;
pub mod security;

// Re-export core types
pub use arch::{Architecture, ChipsetLayout, CpuInfo, MemoryInfo, PlatformProfile};
pub use platform::{HalPlatform, PlatformCapability, DeviceDescriptor, DeviceType, DeviceStatus, MockPlatform};
pub use probe::{SystemProbe, ProbeResult, ProbeStatus, VulnerabilityReport, VulnerabilitySeverity};
pub use security::{SecurityScanner, ProcessInfo, NetworkConnection, ConnectionStatus, ThreatLevel};
