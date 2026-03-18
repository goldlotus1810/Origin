//! # arch — CPU Architecture & Chipset Detection
//!
//! Hỗ trợ tất cả kiến trúc phổ biến:
//!   x86/x64:  PC, Server, Workstation
//!   ARM:      Smartphone, Tablet, Raspberry Pi, Apple Silicon
//!   RISC-V:   SiFive, StarFive, tùy biến
//!   MIPS:     Router, embedded legacy
//!   WASM:     Browser runtime
//!   Xtensa:   ESP32
//!   Thumb:    ARM Cortex-M (MCU)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Architecture — kiến trúc tập lệnh (ISA)
// ─────────────────────────────────────────────────────────────────────────────

/// Kiến trúc tập lệnh CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Architecture {
    /// x86 32-bit (Intel/AMD — PC/Server)
    X86 = 0x01,
    /// x86-64 / AMD64 (PC/Server/Workstation)
    X86_64 = 0x02,
    /// ARM 32-bit (ARMv7 — di động cũ, embedded)
    Arm32 = 0x03,
    /// ARM 64-bit / AArch64 (ARMv8+ — smartphone, Apple M-series, Graviton)
    Arm64 = 0x04,
    /// RISC-V 32-bit (mã nguồn mở — embedded, IoT)
    RiscV32 = 0x05,
    /// RISC-V 64-bit (mã nguồn mở — server, workstation)
    RiscV64 = 0x06,
    /// MIPS (router, embedded legacy)
    Mips = 0x07,
    /// Xtensa (ESP32 — IoT/smart home)
    Xtensa = 0x08,
    /// ARM Thumb/Cortex-M (MCU: STM32, nRF52, RP2040)
    Thumb = 0x09,
    /// WebAssembly (browser runtime)
    Wasm = 0x0A,
    /// Không xác định
    Unknown = 0xFF,
}

impl Architecture {
    /// Detect kiến trúc lúc compile-time.
    #[allow(clippy::needless_return)]
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86")]
        {
            return Architecture::X86;
        }
        #[cfg(target_arch = "x86_64")]
        {
            return Architecture::X86_64;
        }
        #[cfg(target_arch = "arm")]
        {
            return Architecture::Arm32;
        }
        #[cfg(target_arch = "aarch64")]
        {
            return Architecture::Arm64;
        }
        #[cfg(target_arch = "riscv32")]
        {
            return Architecture::RiscV32;
        }
        #[cfg(target_arch = "riscv64")]
        {
            return Architecture::RiscV64;
        }
        #[cfg(target_arch = "mips")]
        {
            return Architecture::Mips;
        }
        #[cfg(target_arch = "wasm32")]
        {
            return Architecture::Wasm;
        }
        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "arm",
            target_arch = "aarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "mips",
            target_arch = "wasm32",
        )))]
        {
            Architecture::Unknown
        }
    }

    /// Bit-width: 32 hay 64.
    pub fn bits(&self) -> u8 {
        match self {
            Self::X86
            | Self::Arm32
            | Self::RiscV32
            | Self::Mips
            | Self::Xtensa
            | Self::Thumb
            | Self::Wasm => 32,
            Self::X86_64 | Self::Arm64 | Self::RiscV64 => 64,
            Self::Unknown => 0,
        }
    }

    /// Có hỗ trợ floating-point hardware không.
    pub fn has_fpu(&self) -> bool {
        match self {
            Self::X86 | Self::X86_64 | Self::Arm64 | Self::RiscV64 => true,
            Self::Arm32 | Self::RiscV32 => true, // thường có, tuỳ variant
            Self::Thumb | Self::Xtensa => false, // MCU: soft-float thường
            Self::Mips | Self::Wasm => true,
            Self::Unknown => false,
        }
    }

    /// Endianness mặc định.
    pub fn is_little_endian(&self) -> bool {
        match self {
            Self::X86
            | Self::X86_64
            | Self::Arm32
            | Self::Arm64
            | Self::RiscV32
            | Self::RiscV64
            | Self::Thumb
            | Self::Wasm => true,
            Self::Mips | Self::Xtensa => false, // MIPS: big-endian mặc định
            Self::Unknown => true,
        }
    }

    /// Tên hiển thị.
    pub fn name(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86-64",
            Self::Arm32 => "ARM32 (ARMv7)",
            Self::Arm64 => "ARM64 (AArch64)",
            Self::RiscV32 => "RISC-V 32",
            Self::RiscV64 => "RISC-V 64",
            Self::Mips => "MIPS",
            Self::Xtensa => "Xtensa (ESP32)",
            Self::Thumb => "ARM Cortex-M (Thumb)",
            Self::Wasm => "WebAssembly",
            Self::Unknown => "Unknown",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ChipsetLayout — cấu trúc chipset
// ─────────────────────────────────────────────────────────────────────────────

/// Cấu trúc chipset / bus topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipsetLayout {
    /// Kiến trúc truyền thống: Northbridge + Southbridge
    /// (CPU ↔ Northbridge ↔ RAM/GPU, Northbridge ↔ Southbridge ↔ I/O)
    NorthSouth,
    /// Platform Controller Hub (Intel từ 5-Series trở đi)
    /// (CPU ↔ PCH trực tiếp, PCH quản lý I/O)
    Pch,
    /// System-on-Chip: CPU + GPU + RAM controller + I/O trên 1 đế chip
    /// (ARM SoC, Apple Silicon, Qualcomm Snapdragon)
    Soc,
    /// Microcontroller Unit: CPU + Flash + SRAM + peripherals trên 1 chip
    /// (ESP32, STM32, RP2040, nRF52)
    Mcu,
    /// Virtual / emulated (WASM, VM)
    Virtual,
    /// Không xác định
    Unknown,
}

impl ChipsetLayout {
    /// Infer chipset layout từ architecture.
    pub fn from_arch(arch: Architecture) -> Self {
        match arch {
            Architecture::X86 => ChipsetLayout::NorthSouth, // legacy PC
            Architecture::X86_64 => ChipsetLayout::Pch,     // modern PC
            Architecture::Arm32 => ChipsetLayout::Soc,      // mobile SoC
            Architecture::Arm64 => ChipsetLayout::Soc,      // modern mobile/server SoC
            Architecture::RiscV32 => ChipsetLayout::Mcu,    // embedded RISC-V
            Architecture::RiscV64 => ChipsetLayout::Soc,    // server RISC-V
            Architecture::Mips => ChipsetLayout::Soc,       // router SoC
            Architecture::Xtensa => ChipsetLayout::Mcu,     // ESP32
            Architecture::Thumb => ChipsetLayout::Mcu,      // Cortex-M MCU
            Architecture::Wasm => ChipsetLayout::Virtual,   // browser
            Architecture::Unknown => ChipsetLayout::Unknown,
        }
    }

    /// Tên hiển thị.
    pub fn name(&self) -> &'static str {
        match self {
            Self::NorthSouth => "Traditional (Northbridge/Southbridge)",
            Self::Pch => "Platform Controller Hub (PCH)",
            Self::Soc => "System-on-Chip (SoC)",
            Self::Mcu => "Microcontroller Unit (MCU)",
            Self::Virtual => "Virtual/Emulated",
            Self::Unknown => "Unknown",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CpuInfo — thông tin CPU cơ bản
// ─────────────────────────────────────────────────────────────────────────────

/// Thông tin CPU — platform impl điền vào.
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// Kiến trúc
    pub arch: Architecture,
    /// Số core (0 = unknown)
    pub cores: u8,
    /// Tốc độ MHz (0 = unknown)
    pub clock_mhz: u32,
    /// Vendor string (ví dụ: "GenuineIntel", "ARM", "sifive")
    pub vendor: String,
    /// Model string
    pub model: String,
    /// Có SIMD (SSE/NEON/RVV) không
    pub has_simd: bool,
    /// Có crypto extension (AES-NI/ARMv8-CE) không
    pub has_crypto: bool,
}

impl CpuInfo {
    /// CPU không xác định — dùng khi probe thất bại.
    pub fn unknown() -> Self {
        Self {
            arch: Architecture::Unknown,
            cores: 0,
            clock_mhz: 0,
            vendor: String::new(),
            model: String::new(),
            has_simd: false,
            has_crypto: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MemoryInfo — thông tin bộ nhớ
// ─────────────────────────────────────────────────────────────────────────────

/// Thông tin bộ nhớ.
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Tổng RAM (bytes). 0 = unknown.
    pub total_bytes: u64,
    /// RAM khả dụng (bytes). 0 = unknown.
    pub available_bytes: u64,
    /// Tổng Flash/Storage (bytes). 0 = unknown hoặc không áp dụng.
    pub storage_bytes: u64,
}

impl MemoryInfo {
    /// Unknown memory.
    pub fn unknown() -> Self {
        Self {
            total_bytes: 0,
            available_bytes: 0,
            storage_bytes: 0,
        }
    }

    /// Tỷ lệ sử dụng RAM (0.0 .. 1.0).
    pub fn usage_ratio(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        let used = self.total_bytes.saturating_sub(self.available_bytes);
        (used as f32) / (self.total_bytes as f32)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PlatformProfile — tổng hợp thông tin platform
// ─────────────────────────────────────────────────────────────────────────────

/// Platform profile — toàn bộ thông tin phần cứng/platform.
///
/// Worker gọi `HalPlatform::profile()` lúc boot → nhận struct này.
/// Chief tổng hợp profiles từ tất cả Worker → SystemManifest.
#[derive(Debug, Clone)]
pub struct PlatformProfile {
    /// CPU info
    pub cpu: CpuInfo,
    /// Chipset layout
    pub chipset: ChipsetLayout,
    /// Memory info
    pub memory: MemoryInfo,
    /// OS hoặc runtime (ví dụ: "linux", "android", "freertos", "wasm")
    pub os: String,
    /// Thiết bị ngoại vi đã phát hiện
    pub peripherals: Vec<String>,
    /// Timestamp khi profile
    pub probed_at: i64,
}

impl PlatformProfile {
    /// Profile trống — dùng khi chưa probe.
    pub fn empty() -> Self {
        Self {
            cpu: CpuInfo::unknown(),
            chipset: ChipsetLayout::Unknown,
            memory: MemoryInfo::unknown(),
            os: String::new(),
            peripherals: Vec::new(),
            probed_at: 0,
        }
    }

    /// Summary 1 dòng.
    pub fn summary(&self) -> String {
        alloc::format!(
            "{} ({}) | {} | {}MB RAM | {} peripherals",
            self.cpu.arch.name(),
            self.chipset.name(),
            if self.os.is_empty() {
                "unknown-os"
            } else {
                &self.os
            },
            self.memory.total_bytes / (1024 * 1024),
            self.peripherals.len(),
        )
    }

    /// Encode profile → compact bytes cho ISL transport.
    ///
    /// Format: [arch:1][chipset:1][cores:1][os_len:1][os:N][periph_count:1]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.cpu.arch as u8);
        buf.push(self.chipset as u8);
        buf.push(self.cpu.cores);
        let os_bytes = self.os.as_bytes();
        buf.push(os_bytes.len() as u8);
        buf.extend_from_slice(os_bytes);
        buf.push(self.peripherals.len() as u8);
        buf
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_detect_not_unknown() {
        let arch = Architecture::detect();
        // Trên test runner (x86_64 hoặc aarch64), phải detect được
        assert_ne!(
            arch,
            Architecture::Unknown,
            "Phải detect được arch: {:?}",
            arch
        );
    }

    #[test]
    fn arch_bits_correct() {
        assert_eq!(Architecture::X86_64.bits(), 64);
        assert_eq!(Architecture::Arm64.bits(), 64);
        assert_eq!(Architecture::Arm32.bits(), 32);
        assert_eq!(Architecture::RiscV32.bits(), 32);
        assert_eq!(Architecture::RiscV64.bits(), 64);
        assert_eq!(Architecture::Wasm.bits(), 32);
        assert_eq!(Architecture::Thumb.bits(), 32);
    }

    #[test]
    fn arch_fpu_flags() {
        assert!(Architecture::X86_64.has_fpu());
        assert!(Architecture::Arm64.has_fpu());
        assert!(!Architecture::Thumb.has_fpu(), "MCU thường no FPU");
        assert!(!Architecture::Xtensa.has_fpu(), "ESP32 soft-float");
    }

    #[test]
    fn arch_endianness() {
        assert!(Architecture::X86_64.is_little_endian());
        assert!(Architecture::Arm64.is_little_endian());
        assert!(Architecture::RiscV64.is_little_endian());
        assert!(
            !Architecture::Mips.is_little_endian(),
            "MIPS big-endian mặc định"
        );
    }

    #[test]
    fn chipset_from_arch() {
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::X86_64),
            ChipsetLayout::Pch
        );
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::Arm64),
            ChipsetLayout::Soc
        );
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::Xtensa),
            ChipsetLayout::Mcu
        );
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::Wasm),
            ChipsetLayout::Virtual
        );
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::RiscV32),
            ChipsetLayout::Mcu
        );
        assert_eq!(
            ChipsetLayout::from_arch(Architecture::RiscV64),
            ChipsetLayout::Soc
        );
    }

    #[test]
    fn cpu_info_unknown() {
        let cpu = CpuInfo::unknown();
        assert_eq!(cpu.arch, Architecture::Unknown);
        assert_eq!(cpu.cores, 0);
    }

    #[test]
    fn memory_usage_ratio() {
        let mem = MemoryInfo {
            total_bytes: 1000,
            available_bytes: 400,
            storage_bytes: 0,
        };
        let ratio = mem.usage_ratio();
        assert!((ratio - 0.6).abs() < 0.01, "60% used: {}", ratio);
    }

    #[test]
    fn memory_usage_ratio_zero_total() {
        let mem = MemoryInfo::unknown();
        assert_eq!(mem.usage_ratio(), 0.0, "0 total → 0 ratio");
    }

    #[test]
    fn platform_profile_summary() {
        let profile = PlatformProfile {
            cpu: CpuInfo {
                arch: Architecture::Arm64,
                cores: 8,
                clock_mhz: 2400,
                vendor: String::from("ARM"),
                model: String::from("Cortex-A78"),
                has_simd: true,
                has_crypto: true,
            },
            chipset: ChipsetLayout::Soc,
            memory: MemoryInfo {
                total_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                available_bytes: 4 * 1024 * 1024 * 1024,
                storage_bytes: 128 * 1024 * 1024 * 1024,
            },
            os: String::from("android"),
            peripherals: alloc::vec![
                String::from("camera0"),
                String::from("gps"),
                String::from("wifi"),
            ],
            probed_at: 1000,
        };
        let s = profile.summary();
        assert!(s.contains("ARM64"), "Summary chứa arch: {}", s);
        assert!(s.contains("SoC"), "Summary chứa chipset: {}", s);
        assert!(s.contains("android"), "Summary chứa OS: {}", s);
        assert!(
            s.contains("3 peripherals"),
            "Summary chứa peripheral count: {}",
            s
        );
    }

    #[test]
    fn platform_profile_to_bytes() {
        let mut profile = PlatformProfile::empty();
        profile.cpu.arch = Architecture::Arm64;
        profile.chipset = ChipsetLayout::Soc;
        profile.cpu.cores = 4;
        profile.os = String::from("linux");
        let bytes = profile.to_bytes();
        assert_eq!(bytes[0], Architecture::Arm64 as u8);
        assert_eq!(bytes[1], ChipsetLayout::Soc as u8);
        assert_eq!(bytes[2], 4); // cores
        assert_eq!(bytes[3], 5); // "linux" = 5 bytes
    }

    #[test]
    fn profile_empty() {
        let p = PlatformProfile::empty();
        assert_eq!(p.cpu.arch, Architecture::Unknown);
        assert!(p.peripherals.is_empty());
        assert_eq!(p.probed_at, 0);
    }

    #[test]
    fn all_arch_names_non_empty() {
        let archs = [
            Architecture::X86,
            Architecture::X86_64,
            Architecture::Arm32,
            Architecture::Arm64,
            Architecture::RiscV32,
            Architecture::RiscV64,
            Architecture::Mips,
            Architecture::Xtensa,
            Architecture::Thumb,
            Architecture::Wasm,
            Architecture::Unknown,
        ];
        for a in archs {
            assert!(!a.name().is_empty(), "{:?} must have name", a);
        }
    }

    #[test]
    fn all_chipset_names_non_empty() {
        let layouts = [
            ChipsetLayout::NorthSouth,
            ChipsetLayout::Pch,
            ChipsetLayout::Soc,
            ChipsetLayout::Mcu,
            ChipsetLayout::Virtual,
            ChipsetLayout::Unknown,
        ];
        for l in layouts {
            assert!(!l.name().is_empty(), "{:?} must have name", l);
        }
    }
}
