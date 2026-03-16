//! # ffi — Platform FFI Stubs
//!
//! C-compatible interface cho Android (JNI), iOS, và native platforms.
//! HomeOS core = no_std Rust. FFI layer bridge → platform-specific code.
//!
//! Architecture:
//!   Android: Java/Kotlin → JNI → C → hal::ffi → HomeRuntime
//!   iOS:     Swift → C-bridge → hal::ffi → HomeRuntime
//!   Desktop: direct Rust call → HomeRuntime
//!   WASM:    wasm-bindgen → HomeRuntime (riêng crate wasm)
//!
//! Mỗi platform implement trait PlatformBridge.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::tier::HardwareTier;
use crate::arch::Architecture;

// ─────────────────────────────────────────────────────────────────────────────
// PlatformBridge — trait cho platform-specific code
// ─────────────────────────────────────────────────────────────────────────────

/// Bridge giữa HomeOS core và platform-specific code.
///
/// Mỗi platform (Android/iOS/Desktop/WASM) implement trait này.
/// HomeOS core gọi qua trait — không biết platform là gì.
pub trait PlatformBridge {
    /// Platform name.
    fn name(&self) -> &str;

    /// Hardware tier.
    fn tier(&self) -> HardwareTier;

    /// Current timestamp (ms since epoch).
    fn timestamp_ms(&self) -> i64;

    /// Read file bytes (platform-specific storage).
    fn read_file(&self, path: &str) -> Option<Vec<u8>>;

    /// Write file bytes.
    fn write_file(&self, path: &str, data: &[u8]) -> bool;

    /// Send notification to user.
    fn notify(&self, title: &str, body: &str);

    /// Log message (platform-specific logging).
    fn log(&self, level: LogLevel, msg: &str);

    /// Get sensor reading (if available).
    fn read_sensor(&self, sensor_id: &str) -> Option<f32>;

    /// Play haptic feedback (mobile only).
    fn haptic(&self, _pattern: HapticPattern) {}

    /// Get display DPI.
    fn display_dpi(&self) -> f32 { 96.0 }

    /// Network available?
    fn network_available(&self) -> bool { true }
}

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Haptic feedback pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticPattern {
    /// Light tap
    Light,
    /// Medium impact
    Medium,
    /// Heavy impact
    Heavy,
    /// Success pattern
    Success,
    /// Warning pattern
    Warning,
    /// Error pattern
    Error,
}

// ─────────────────────────────────────────────────────────────────────────────
// C-FFI structs — for JNI/iOS bridge
// ─────────────────────────────────────────────────────────────────────────────

/// C-compatible response from HomeOS.
///
/// Dùng cho JNI (Android) và C-bridge (iOS).
/// Caller phải free `text_ptr` sau khi dùng.
#[repr(C)]
pub struct FfiResponse {
    /// Response text (UTF-8, null-terminated)
    pub text_ptr: *const u8,
    /// Text length (không tính null terminator)
    pub text_len: u32,
    /// Tone (0=supportive, 1=pause, 2=reinforcing, 3=celebratory, 4=gentle, 5=engaged)
    pub tone: u8,
    /// f(x) value × 1000 (integer -1000..1000)
    pub fx_milli: i16,
    /// Response kind (0=natural, 1=olang, 2=crisis, 3=blocked, 4=system)
    pub kind: u8,
}

/// C-compatible config for HomeOS initialization.
#[repr(C)]
pub struct FfiConfig {
    /// Hardware tier (1-4)
    pub tier: u8,
    /// Storage path (UTF-8, null-terminated)
    pub storage_path_ptr: *const u8,
    pub storage_path_len: u32,
    /// Session ID
    pub session_id: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// DesktopBridge — native desktop implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Desktop bridge — dùng cho testing và native Linux/macOS/Windows.
pub struct DesktopBridge {
    tier: HardwareTier,
}

impl DesktopBridge {
    pub fn new() -> Self {
        let arch = Architecture::detect();
        Self {
            tier: HardwareTier::from_arch(arch),
        }
    }
}

impl Default for DesktopBridge {
    fn default() -> Self { Self::new() }
}

impl PlatformBridge for DesktopBridge {
    fn name(&self) -> &str { "Desktop" }
    fn tier(&self) -> HardwareTier { self.tier }
    fn timestamp_ms(&self) -> i64 { 0 } // real impl: std::time
    fn read_file(&self, _path: &str) -> Option<Vec<u8>> { None } // real impl: std::fs
    fn write_file(&self, _path: &str, _data: &[u8]) -> bool { false }
    fn notify(&self, _title: &str, _body: &str) {} // real impl: desktop notification
    fn log(&self, _level: LogLevel, _msg: &str) {} // real impl: println! or log crate
    fn read_sensor(&self, _sensor_id: &str) -> Option<f32> { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// AndroidBridge — JNI stub
// ─────────────────────────────────────────────────────────────────────────────

/// Android bridge — JNI stub.
///
/// Real implementation sẽ call qua JNI:
/// - Java: `android.hardware.SensorManager`
/// - File: `Context.getFilesDir()`
/// - Notification: `NotificationManager`
pub struct AndroidBridge {
    tier: HardwareTier,
    package_name: String,
}

impl AndroidBridge {
    pub fn new(package_name: &str) -> Self {
        Self {
            tier: HardwareTier::Compact, // most Android = Compact or Full
            package_name: String::from(package_name),
        }
    }

    /// Package name.
    pub fn package(&self) -> &str { &self.package_name }
}

impl PlatformBridge for AndroidBridge {
    fn name(&self) -> &str { "Android" }
    fn tier(&self) -> HardwareTier { self.tier }
    fn timestamp_ms(&self) -> i64 { 0 } // JNI: System.currentTimeMillis()
    fn read_file(&self, _path: &str) -> Option<Vec<u8>> { None } // JNI: FileInputStream
    fn write_file(&self, _path: &str, _data: &[u8]) -> bool { false }
    fn notify(&self, _title: &str, _body: &str) {} // JNI: NotificationManager
    fn log(&self, _level: LogLevel, _msg: &str) {} // JNI: android.util.Log
    fn read_sensor(&self, _sensor_id: &str) -> Option<f32> { None } // JNI: SensorManager
    fn haptic(&self, _pattern: HapticPattern) {} // JNI: Vibrator
    fn display_dpi(&self) -> f32 { 320.0 } // typical Android DPI
}

// ─────────────────────────────────────────────────────────────────────────────
// IosBridge — C-bridge stub
// ─────────────────────────────────────────────────────────────────────────────

/// iOS bridge — C-bridge stub.
///
/// Real implementation sẽ call qua C-bridge:
/// - Swift: `CMMotionManager` (sensors)
/// - File: `FileManager.default.urls(for: .documentDirectory)`
/// - Notification: `UNUserNotificationCenter`
pub struct IosBridge {
    tier: HardwareTier,
    bundle_id: String,
}

impl IosBridge {
    pub fn new(bundle_id: &str) -> Self {
        Self {
            tier: HardwareTier::Full, // iOS devices thường mạnh
            bundle_id: String::from(bundle_id),
        }
    }

    /// Bundle ID.
    pub fn bundle(&self) -> &str { &self.bundle_id }
}

impl PlatformBridge for IosBridge {
    fn name(&self) -> &str { "iOS" }
    fn tier(&self) -> HardwareTier { self.tier }
    fn timestamp_ms(&self) -> i64 { 0 } // C: CFAbsoluteTimeGetCurrent()
    fn read_file(&self, _path: &str) -> Option<Vec<u8>> { None }
    fn write_file(&self, _path: &str, _data: &[u8]) -> bool { false }
    fn notify(&self, _title: &str, _body: &str) {} // C: UNUserNotificationCenter
    fn log(&self, _level: LogLevel, _msg: &str) {} // C: os_log
    fn read_sensor(&self, _sensor_id: &str) -> Option<f32> { None } // C: CMMotionManager
    fn haptic(&self, _pattern: HapticPattern) {} // C: UIImpactFeedbackGenerator
    fn display_dpi(&self) -> f32 { 326.0 } // iPhone Retina
}

// ─────────────────────────────────────────────────────────────────────────────
// EmbeddedBridge — bare-metal stub
// ─────────────────────────────────────────────────────────────────────────────

/// Embedded bridge — ESP32/RPi stub.
pub struct EmbeddedBridge {
    tier: HardwareTier,
    board_name: String,
}

impl EmbeddedBridge {
    pub fn esp32() -> Self {
        Self { tier: HardwareTier::Worker, board_name: String::from("ESP32") }
    }

    pub fn raspberry_pi() -> Self {
        Self { tier: HardwareTier::Full, board_name: String::from("Raspberry Pi") }
    }

    pub fn riscv_mcu() -> Self {
        Self { tier: HardwareTier::Sensor, board_name: String::from("RISC-V MCU") }
    }

    pub fn board(&self) -> &str { &self.board_name }
}

impl PlatformBridge for EmbeddedBridge {
    fn name(&self) -> &str { &self.board_name }
    fn tier(&self) -> HardwareTier { self.tier }
    fn timestamp_ms(&self) -> i64 { 0 } // HAL: timer register
    fn read_file(&self, _path: &str) -> Option<Vec<u8>> { None } // SPI Flash / SD card
    fn write_file(&self, _path: &str, _data: &[u8]) -> bool { false }
    fn notify(&self, _title: &str, _body: &str) {} // LED blink / buzzer
    fn log(&self, _level: LogLevel, _msg: &str) {} // UART serial
    fn read_sensor(&self, _sensor_id: &str) -> Option<f32> { None } // I2C/SPI sensor
    fn display_dpi(&self) -> f32 { 72.0 } // small display / e-ink
    fn network_available(&self) -> bool { false } // WiFi optional
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_bridge() {
        let bridge = DesktopBridge::new();
        assert_eq!(bridge.name(), "Desktop");
        assert!(bridge.network_available());
    }

    #[test]
    fn android_bridge() {
        let bridge = AndroidBridge::new("com.homeos.app");
        assert_eq!(bridge.name(), "Android");
        assert_eq!(bridge.package(), "com.homeos.app");
        assert_eq!(bridge.tier(), HardwareTier::Compact);
        assert!((bridge.display_dpi() - 320.0).abs() < 0.1);
    }

    #[test]
    fn ios_bridge() {
        let bridge = IosBridge::new("com.homeos.ios");
        assert_eq!(bridge.name(), "iOS");
        assert_eq!(bridge.tier(), HardwareTier::Full);
    }

    #[test]
    fn embedded_esp32() {
        let bridge = EmbeddedBridge::esp32();
        assert_eq!(bridge.tier(), HardwareTier::Worker);
        assert_eq!(bridge.board(), "ESP32");
        assert!(!bridge.network_available());
    }

    #[test]
    fn embedded_rpi() {
        let bridge = EmbeddedBridge::raspberry_pi();
        assert_eq!(bridge.tier(), HardwareTier::Full);
    }

    #[test]
    fn embedded_riscv() {
        let bridge = EmbeddedBridge::riscv_mcu();
        assert_eq!(bridge.tier(), HardwareTier::Sensor);
    }

    #[test]
    fn platform_bridge_trait_object() {
        // Verify trait object works (dyn dispatch)
        let d = DesktopBridge::new();
        let a = AndroidBridge::new("test");
        let i = IosBridge::new("test");
        let e = EmbeddedBridge::esp32();
        let bridges: alloc::vec::Vec<&dyn PlatformBridge> = alloc::vec![&d, &a, &i, &e];
        for b in bridges {
            assert!(!b.name().is_empty());
        }
    }

    #[test]
    fn ffi_response_layout() {
        // C-compatible: verify FfiResponse can be created
        let r = FfiResponse {
            text_ptr: core::ptr::null(),
            text_len: 0,
            tone: 5,
            fx_milli: -150,
            kind: 0,
        };
        assert_eq!(r.tone, 5);
        assert_eq!(r.fx_milli, -150);
    }
}
