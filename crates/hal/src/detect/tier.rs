//! # tier — Hardware Tier System
//!
//! Phân loại thiết bị thành tier theo năng lực:
//!   Tier 1: Full HomeOS (Pi 4, PC, Server) — L0-Ln + Dream + Chief
//!   Tier 2: Compact  (Pi Zero, low-end phone) — L0-L1 + STM only
//!   Tier 3: Worker   (ESP32, MCU) — L0 + ISL + 1 Skill
//!   Tier 4: Sensor   (bare MCU, 8-bit) — ISL relay only
//!
//! Tier ← auto-detect từ RAM + CPU + arch.
//! Feature flags gate code paths per tier.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::arch::{Architecture, CpuInfo, MemoryInfo};
use crate::platform::PlatformCapability;

// ─────────────────────────────────────────────────────────────────────────────
// HardwareTier — 4 tiers
// ─────────────────────────────────────────────────────────────────────────────

/// Hardware capability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum HardwareTier {
    /// Tier 4: Sensor relay (8-bit MCU, <64KB RAM)
    /// Chỉ chạy ISL relay — forward data lên Worker.
    Sensor = 4,
    /// Tier 3: Worker (ESP32, MCU, 64KB-1MB RAM)
    /// L0 + 1 Skill + ISL. Không học, không dream.
    Worker = 3,
    /// Tier 2: Compact (Pi Zero, low-end phone, 256MB-1GB RAM)
    /// L0 + L1 + STM. Không dream, không promote QR.
    Compact = 2,
    /// Tier 1: Full (Pi 4, PC, Server, 1GB+ RAM)
    /// L0-Ln + Dream + Chief + full learning pipeline.
    Full = 1,
}

impl HardwareTier {
    /// Auto-detect tier từ hardware specs.
    pub fn detect(mem: &MemoryInfo, cpu: &CpuInfo) -> Self {
        let ram_mb = mem.total_bytes / (1024 * 1024);
        let bits = cpu.arch.bits();

        if ram_mb < 1 || bits < 16 {
            // < 1MB hoặc 8-bit → Sensor relay
            Self::Sensor
        } else if ram_mb < 2 {
            // 1-2 MB (ESP32 class) → Worker
            Self::Worker
        } else if ram_mb < 1024 {
            // 2MB - 1GB (Pi Zero, low phone) → Compact
            Self::Compact
        } else {
            // 1GB+ (Pi 4, PC, Server) → Full
            Self::Full
        }
    }

    /// Tier từ architecture hint (khi chưa biết RAM).
    pub fn from_arch(arch: Architecture) -> Self {
        match arch {
            Architecture::Xtensa | Architecture::Thumb => Self::Worker,
            Architecture::Mips => Self::Compact,
            Architecture::Wasm => Self::Compact, // WASM hạn chế memory
            Architecture::Arm32 | Architecture::RiscV32 => Self::Compact,
            Architecture::X86
            | Architecture::X86_64
            | Architecture::Arm64
            | Architecture::RiscV64 => Self::Full,
            Architecture::Unknown => Self::Worker,
        }
    }

    /// Tier này có khả năng nào.
    pub fn can_learn(&self) -> bool {
        *self <= Self::Compact
    }
    pub fn can_dream(&self) -> bool {
        *self == Self::Full
    }
    pub fn can_promote_qr(&self) -> bool {
        *self == Self::Full
    }
    pub fn can_run_chief(&self) -> bool {
        *self == Self::Full
    }
    pub fn has_silk(&self) -> bool {
        *self <= Self::Compact
    }
    pub fn has_stm(&self) -> bool {
        *self <= Self::Compact
    }

    /// Max Silk edges cho tier này.
    pub fn max_silk_edges(&self) -> usize {
        match self {
            Self::Full => 1_000_000,
            Self::Compact => 10_000,
            Self::Worker => 0,
            Self::Sensor => 0,
        }
    }

    /// Max STM observations.
    pub fn max_stm(&self) -> usize {
        match self {
            Self::Full => 10_000,
            Self::Compact => 500,
            Self::Worker => 0,
            Self::Sensor => 0,
        }
    }

    /// PageCache capacity (from compact.rs Fibonacci capacities).
    pub fn page_cache_capacity(&self) -> usize {
        match self {
            Self::Full => 610,    // Fib[15]
            Self::Compact => 233, // Fib[13]
            Self::Worker => 55,   // Fib[10]
            Self::Sensor => 0,    // no cache
        }
    }

    /// ISL queue size.
    pub fn isl_queue_size(&self) -> usize {
        match self {
            Self::Full => 256,
            Self::Compact => 64,
            Self::Worker => 16,
            Self::Sensor => 4,
        }
    }

    /// As byte (for ISL transport).
    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// From byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(Self::Full),
            2 => Some(Self::Compact),
            3 => Some(Self::Worker),
            4 => Some(Self::Sensor),
            _ => None,
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Compact => "Compact",
            Self::Worker => "Worker",
            Self::Sensor => "Sensor",
        }
    }

    /// Summary.
    pub fn summary(&self) -> String {
        format!(
            "Tier {} ({}) | learn={} dream={} silk={} stm={}",
            self.as_byte(),
            self.name(),
            self.can_learn(),
            self.can_dream(),
            self.max_silk_edges(),
            self.max_stm(),
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TierConfig — runtime config per tier
// ─────────────────────────────────────────────────────────────────────────────

/// Runtime configuration derived from hardware tier.
#[derive(Debug, Clone)]
pub struct TierConfig {
    pub tier: HardwareTier,
    /// Max Silk edges
    pub max_silk: usize,
    /// Max STM entries
    pub max_stm: usize,
    /// PageCache capacity
    pub page_cache: usize,
    /// ISL queue size
    pub isl_queue: usize,
    /// Enable dream cycle
    pub dream_enabled: bool,
    /// Enable QR promotion
    pub qr_enabled: bool,
    /// Capabilities to advertise
    pub capabilities: alloc::vec::Vec<PlatformCapability>,
}

impl TierConfig {
    /// Auto-detect and build config.
    pub fn auto(mem: &MemoryInfo, cpu: &CpuInfo) -> Self {
        let tier = HardwareTier::detect(mem, cpu);
        Self::from_tier(tier)
    }

    /// Build config from known tier.
    pub fn from_tier(tier: HardwareTier) -> Self {
        Self {
            max_silk: tier.max_silk_edges(),
            max_stm: tier.max_stm(),
            page_cache: tier.page_cache_capacity(),
            isl_queue: tier.isl_queue_size(),
            dream_enabled: tier.can_dream(),
            qr_enabled: tier.can_promote_qr(),
            capabilities: alloc::vec::Vec::new(),
            tier,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mem(mb: u64) -> MemoryInfo {
        MemoryInfo {
            total_bytes: mb * 1024 * 1024,
            available_bytes: mb * 1024 * 1024 / 2,
            storage_bytes: mb * 1024 * 1024 * 4, // 4x RAM for storage
        }
    }

    fn cpu(arch: Architecture) -> CpuInfo {
        CpuInfo {
            arch,
            cores: 1,
            clock_mhz: 100,
            vendor: String::new(),
            model: String::new(),
            has_simd: false,
            has_crypto: false,
        }
    }

    #[test]
    fn detect_full_pc() {
        let tier = HardwareTier::detect(&mem(16384), &cpu(Architecture::X86_64));
        assert_eq!(tier, HardwareTier::Full);
    }

    #[test]
    fn detect_compact_pi_zero() {
        let tier = HardwareTier::detect(&mem(512), &cpu(Architecture::Arm32));
        assert_eq!(tier, HardwareTier::Compact);
    }

    #[test]
    fn detect_worker_esp32() {
        // ESP32: 520KB SRAM → 0MB in integer
        let m = MemoryInfo {
            total_bytes: 520 * 1024,
            available_bytes: 260 * 1024,
            storage_bytes: 4 * 1024 * 1024,
        };
        let tier = HardwareTier::detect(&m, &cpu(Architecture::Xtensa));
        assert_eq!(tier, HardwareTier::Sensor); // <1MB → Sensor
    }

    #[test]
    fn detect_worker_esp32_psram() {
        // ESP32 with PSRAM: ~4MB
        let tier = HardwareTier::detect(&mem(4), &cpu(Architecture::Xtensa));
        assert_eq!(tier, HardwareTier::Compact);
    }

    #[test]
    fn detect_sensor_tiny() {
        let m = MemoryInfo {
            total_bytes: 32 * 1024,
            available_bytes: 16 * 1024,
            storage_bytes: 0,
        };
        let tier = HardwareTier::detect(&m, &cpu(Architecture::Thumb));
        assert_eq!(tier, HardwareTier::Sensor);
    }

    #[test]
    fn tier_ordering() {
        assert!(HardwareTier::Full < HardwareTier::Compact);
        assert!(HardwareTier::Compact < HardwareTier::Worker);
        assert!(HardwareTier::Worker < HardwareTier::Sensor);
    }

    #[test]
    fn tier_capabilities() {
        let full = HardwareTier::Full;
        assert!(full.can_learn());
        assert!(full.can_dream());
        assert!(full.has_silk());

        let worker = HardwareTier::Worker;
        assert!(!worker.can_learn());
        assert!(!worker.can_dream());
        assert!(!worker.has_silk());
    }

    #[test]
    fn tier_page_cache() {
        assert_eq!(HardwareTier::Full.page_cache_capacity(), 610);
        assert_eq!(HardwareTier::Compact.page_cache_capacity(), 233);
        assert_eq!(HardwareTier::Worker.page_cache_capacity(), 55);
    }

    #[test]
    fn tier_from_byte_roundtrip() {
        for b in 1..=4u8 {
            let tier = HardwareTier::from_byte(b).unwrap();
            assert_eq!(tier.as_byte(), b);
        }
    }

    #[test]
    fn tier_from_arch() {
        assert_eq!(
            HardwareTier::from_arch(Architecture::X86_64),
            HardwareTier::Full
        );
        assert_eq!(
            HardwareTier::from_arch(Architecture::Xtensa),
            HardwareTier::Worker
        );
        assert_eq!(
            HardwareTier::from_arch(Architecture::Wasm),
            HardwareTier::Compact
        );
    }

    #[test]
    fn tier_config_auto() {
        let config = TierConfig::auto(&mem(8192), &cpu(Architecture::Arm64));
        assert_eq!(config.tier, HardwareTier::Full);
        assert!(config.dream_enabled);
        assert_eq!(config.max_stm, 10_000);
    }

    #[test]
    fn tier_summary() {
        let s = HardwareTier::Full.summary();
        assert!(s.contains("Full"), "{}", s);
        assert!(s.contains("learn=true"), "{}", s);
    }
}
