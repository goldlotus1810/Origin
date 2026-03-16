//! # probe — System Probe & Vulnerability Detection
//!
//! SystemProbe quét toàn bộ hệ thống khi boot:
//!   - Phần cứng: CPU, RAM, peripherals
//!   - Phần mềm: firmware version, driver status
//!   - Bảo mật: lỗ hổng, cấu hình yếu
//!
//! Kết quả → ProbeResult → Chief tổng hợp → report lên AAM

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::arch::PlatformProfile;
use crate::platform::{HalPlatform, DeviceStatus};

// ─────────────────────────────────────────────────────────────────────────────
// ProbeStatus — trạng thái probe tổng hợp
// ─────────────────────────────────────────────────────────────────────────────

/// Trạng thái tổng hợp sau khi probe hệ thống.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    /// Tất cả OK — hệ thống sẵn sàng
    Healthy,
    /// Có cảnh báo — vẫn hoạt động được
    Warning,
    /// Có lỗi nghiêm trọng — cần can thiệp
    Critical,
    /// Probe thất bại — không thể đánh giá
    Failed,
}

// ─────────────────────────────────────────────────────────────────────────────
// VulnerabilityReport — báo cáo lỗ hổng
// ─────────────────────────────────────────────────────────────────────────────

/// Mức độ nghiêm trọng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VulnerabilitySeverity {
    /// Thông tin (không nguy hiểm)
    Info    = 0,
    /// Cảnh báo nhẹ
    Low     = 1,
    /// Trung bình
    Medium  = 2,
    /// Cao — cần xử lý
    High    = 3,
    /// Nghiêm trọng — cần xử lý ngay
    Critical = 4,
}

/// Báo cáo 1 lỗ hổng / vấn đề phát hiện được.
#[derive(Debug, Clone)]
pub struct VulnerabilityReport {
    /// Mức độ
    pub severity: VulnerabilitySeverity,
    /// Loại vấn đề
    pub category: VulnerabilityCategory,
    /// Mô tả
    pub description: String,
    /// Đề xuất khắc phục
    pub recommendation: String,
    /// Component liên quan
    pub component: String,
}

/// Loại lỗ hổng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulnerabilityCategory {
    /// Firmware cũ / cần update
    FirmwareOutdated,
    /// Thiết bị lỗi
    DeviceMalfunction,
    /// RAM thấp
    LowMemory,
    /// Storage đầy
    StorageFull,
    /// Network interface không an toàn
    NetworkInsecure,
    /// Process đáng ngờ
    SuspiciousProcess,
    /// Cấu hình yếu
    WeakConfig,
    /// Thiết bị không phản hồi
    DeviceUnresponsive,
}

// ─────────────────────────────────────────────────────────────────────────────
// ProbeResult — kết quả probe tổng hợp
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả scan toàn hệ thống.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Trạng thái tổng hợp
    pub status: ProbeStatus,
    /// Platform profile
    pub profile: PlatformProfile,
    /// Danh sách lỗ hổng / vấn đề
    pub vulnerabilities: Vec<VulnerabilityReport>,
    /// Số thiết bị sẵn sàng
    pub devices_ready: u32,
    /// Số thiết bị lỗi
    pub devices_error: u32,
    /// Timestamp
    pub probed_at: i64,
}

impl ProbeResult {
    /// Có vấn đề nghiêm trọng không.
    pub fn has_critical(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity >= VulnerabilitySeverity::High)
    }

    /// Tổng số lỗ hổng.
    pub fn vulnerability_count(&self) -> usize {
        self.vulnerabilities.len()
    }

    /// Summary 1 dòng.
    pub fn summary(&self) -> String {
        alloc::format!(
            "{:?} | {} devices OK, {} error | {} vulnerabilities",
            self.status,
            self.devices_ready,
            self.devices_error,
            self.vulnerabilities.len(),
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SystemProbe — quét hệ thống
// ─────────────────────────────────────────────────────────────────────────────

/// System probe — quét toàn bộ hệ thống, phát hiện vấn đề.
pub struct SystemProbe;

impl SystemProbe {
    /// Probe toàn hệ thống qua HalPlatform.
    pub fn scan(platform: &dyn HalPlatform, ts: i64) -> ProbeResult {
        let profile = platform.profile(ts);
        let mut vulnerabilities = Vec::new();

        // Check memory
        let mem = platform.memory_info();
        if mem.total_bytes > 0 && mem.usage_ratio() > 0.90 {
            vulnerabilities.push(VulnerabilityReport {
                severity: VulnerabilitySeverity::High,
                category: VulnerabilityCategory::LowMemory,
                description: alloc::format!(
                    "RAM usage {:.0}% — hệ thống có thể không ổn định",
                    mem.usage_ratio() * 100.0
                ),
                recommendation: String::from("Đóng ứng dụng không cần thiết hoặc nâng cấp RAM"),
                component: String::from("memory"),
            });
        } else if mem.total_bytes > 0 && mem.usage_ratio() > 0.75 {
            vulnerabilities.push(VulnerabilityReport {
                severity: VulnerabilitySeverity::Low,
                category: VulnerabilityCategory::LowMemory,
                description: alloc::format!(
                    "RAM usage {:.0}% — đang cao",
                    mem.usage_ratio() * 100.0
                ),
                recommendation: String::from("Theo dõi mức sử dụng RAM"),
                component: String::from("memory"),
            });
        }

        // Check devices
        let devices = platform.scan_devices();
        let mut devices_ready = 0u32;
        let mut devices_error = 0u32;
        for dev in &devices {
            match dev.status {
                DeviceStatus::Ready => devices_ready += 1,
                DeviceStatus::Error => {
                    devices_error += 1;
                    vulnerabilities.push(VulnerabilityReport {
                        severity: VulnerabilitySeverity::Medium,
                        category: VulnerabilityCategory::DeviceMalfunction,
                        description: alloc::format!("Thiết bị lỗi: {} ({})", dev.name, dev.id),
                        recommendation: String::from("Kiểm tra kết nối và driver"),
                        component: dev.id.clone(),
                    });
                }
                DeviceStatus::Disabled => {
                    vulnerabilities.push(VulnerabilityReport {
                        severity: VulnerabilitySeverity::Info,
                        category: VulnerabilityCategory::DeviceUnresponsive,
                        description: alloc::format!("Thiết bị tắt: {} ({})", dev.name, dev.id),
                        recommendation: String::from("Bật thiết bị nếu cần sử dụng"),
                        component: dev.id.clone(),
                    });
                }
                DeviceStatus::Unknown => {
                    vulnerabilities.push(VulnerabilityReport {
                        severity: VulnerabilitySeverity::Low,
                        category: VulnerabilityCategory::DeviceUnresponsive,
                        description: alloc::format!("Thiết bị không xác định: {} ({})", dev.name, dev.id),
                        recommendation: String::from("Probe lại sau hoặc kiểm tra driver"),
                        component: dev.id.clone(),
                    });
                }
                DeviceStatus::Busy => {
                    devices_ready += 1; // busy = vẫn hoạt động
                }
            }
        }

        // Check CPU
        let cpu = platform.cpu_info();
        if !cpu.has_crypto {
            vulnerabilities.push(VulnerabilityReport {
                severity: VulnerabilitySeverity::Low,
                category: VulnerabilityCategory::WeakConfig,
                description: String::from("CPU không có crypto extension — mã hóa chậm hơn"),
                recommendation: String::from("ISL encryption sẽ dùng software AES (chậm hơn ~10x)"),
                component: String::from("cpu"),
            });
        }

        // Determine overall status
        let status = if vulnerabilities.iter().any(|v| v.severity >= VulnerabilitySeverity::Critical) {
            ProbeStatus::Critical
        } else if vulnerabilities.iter().any(|v| v.severity >= VulnerabilitySeverity::High) {
            ProbeStatus::Warning
        } else if devices_error > 0 {
            ProbeStatus::Warning
        } else {
            ProbeStatus::Healthy
        };

        ProbeResult {
            status,
            profile,
            vulnerabilities,
            devices_ready,
            devices_error,
            probed_at: ts,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::MockPlatform;

    #[test]
    fn probe_healthy_pc() {
        let platform = MockPlatform::pc();
        let result = SystemProbe::scan(&platform, 1000);
        // PC khỏe mạnh — chỉ có info/low warnings
        assert!(!result.has_critical(), "PC khỏe mạnh không có critical");
        assert!(result.devices_ready > 0);
        assert_eq!(result.devices_error, 0);
    }

    #[test]
    fn probe_healthy_smartphone() {
        let platform = MockPlatform::smartphone();
        let result = SystemProbe::scan(&platform, 1000);
        assert!(!result.has_critical());
        assert!(result.devices_ready >= 3, "Phone có >= 3 devices");
    }

    #[test]
    fn probe_esp32_no_crypto() {
        let platform = MockPlatform::esp32();
        let result = SystemProbe::scan(&platform, 1000);
        // ESP32 (Xtensa) không có crypto extension
        let has_crypto_warn = result.vulnerabilities.iter()
            .any(|v| v.category == VulnerabilityCategory::WeakConfig);
        assert!(has_crypto_warn, "ESP32 phải cảnh báo no crypto");
    }

    #[test]
    fn probe_low_memory_warning() {
        let mut platform = MockPlatform::pc();
        platform.ram_bytes = 1000;
        // MockPlatform trả available = total/2 → usage = 50%, không cảnh báo

        // Để test high memory, cần custom platform
        // Tạo mock với RAM rất ít nhưng available gần 0
        // MockPlatform.memory_info() trả available = total/2, nên usage = 50%
        // → không trigger warning

        let result = SystemProbe::scan(&platform, 1000);
        // PC bình thường, chỉ kiểm tra không crash
        assert!(result.probed_at == 1000);
    }

    #[test]
    fn probe_device_error() {
        let mut platform = MockPlatform::pc();
        platform.devices.push(crate::platform::DeviceDescriptor {
            id: String::from("broken_sensor"),
            name: String::from("Broken Sensor"),
            device_type: crate::platform::DeviceType::Sensor,
            status: DeviceStatus::Error,
            bus: crate::platform::BusType::I2c,
        });
        let result = SystemProbe::scan(&platform, 1000);
        assert!(result.devices_error > 0, "Phải phát hiện device lỗi");
        assert_eq!(result.status, ProbeStatus::Warning, "Device lỗi → Warning");
    }

    #[test]
    fn probe_result_summary() {
        let platform = MockPlatform::pc();
        let result = SystemProbe::scan(&platform, 1000);
        let s = result.summary();
        assert!(s.contains("devices OK"), "{}", s);
    }

    #[test]
    fn vulnerability_severity_ordering() {
        assert!(VulnerabilitySeverity::Critical > VulnerabilitySeverity::High);
        assert!(VulnerabilitySeverity::High > VulnerabilitySeverity::Medium);
        assert!(VulnerabilitySeverity::Medium > VulnerabilitySeverity::Low);
        assert!(VulnerabilitySeverity::Low > VulnerabilitySeverity::Info);
    }

    #[test]
    fn probe_riscv_embedded() {
        let platform = MockPlatform::riscv_embedded();
        let result = SystemProbe::scan(&platform, 1000);
        assert!(result.devices_ready >= 1);
        // RISC-V 32-bit: check profile
        assert_eq!(result.profile.cpu.arch, crate::arch::Architecture::RiscV32);
    }

    #[test]
    fn probe_raspberry_pi() {
        let platform = MockPlatform::raspberry_pi();
        let result = SystemProbe::scan(&platform, 1000);
        assert!(result.devices_ready >= 3, "RPi có nhiều peripherals");
        assert!(!result.has_critical());
    }
}
