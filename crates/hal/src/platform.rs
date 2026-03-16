//! # platform — Platform abstraction traits
//!
//! Trait `HalPlatform` = interface duy nhất giữa HomeOS core và phần cứng.
//! Worker gọi trait → platform impl trả kết quả.
//!
//! Platform impl:
//!   MockPlatform  — test (no_std, in-memory)
//!   LinuxPlatform — PC Linux (đọc /proc, /sys)
//!   AndroidPlatform — Android (JNI bridge)
//!   EspPlatform   — ESP32 (register access)
//!   WasmPlatform  — Browser (Web APIs)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::arch::{Architecture, ChipsetLayout, CpuInfo, MemoryInfo, PlatformProfile};

// ─────────────────────────────────────────────────────────────────────────────
// PlatformCapability — khả năng platform hỗ trợ
// ─────────────────────────────────────────────────────────────────────────────

/// Khả năng mà platform cung cấp.
/// Worker kiểm tra trước khi sử dụng tính năng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlatformCapability {
    /// Đọc sensor (temperature, humidity, etc.)
    SensorRead   = 0x01,
    /// Điều khiển actuator (relay, motor)
    ActuatorCtrl = 0x02,
    /// Camera/video capture
    Camera       = 0x03,
    /// Network monitoring
    NetworkMon   = 0x04,
    /// File system access
    FileSystem   = 0x05,
    /// Process listing / monitoring
    ProcessList  = 0x06,
    /// GPIO access (embedded)
    Gpio         = 0x07,
    /// I2C bus (embedded)
    I2c          = 0x08,
    /// SPI bus (embedded)
    Spi          = 0x09,
    /// UART serial (embedded)
    Uart         = 0x0A,
    /// Bluetooth
    Bluetooth    = 0x0B,
    /// WiFi
    Wifi         = 0x0C,
    /// USB host
    Usb          = 0x0D,
    /// Display output
    Display      = 0x0E,
    /// Audio input/output
    Audio        = 0x0F,
}

// ─────────────────────────────────────────────────────────────────────────────
// DeviceDescriptor — mô tả thiết bị ngoại vi
// ─────────────────────────────────────────────────────────────────────────────

/// Mô tả 1 thiết bị ngoại vi phát hiện được.
#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    /// ID unique trên platform (bus address, USB path, etc.)
    pub id: String,
    /// Tên thiết bị
    pub name: String,
    /// Loại thiết bị
    pub device_type: DeviceType,
    /// Trạng thái hiện tại
    pub status: DeviceStatus,
    /// Bus kết nối
    pub bus: BusType,
}

/// Loại thiết bị ngoại vi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Sensor (đọc dữ liệu)
    Sensor,
    /// Actuator (điều khiển)
    Actuator,
    /// Camera
    Camera,
    /// Network interface (wifi, ethernet, BLE)
    Network,
    /// Storage (SD, flash, SSD)
    Storage,
    /// Display
    Display,
    /// Audio
    Audio,
    /// GPIO pin
    GpioPin,
    /// Khác
    Other,
}

/// Trạng thái thiết bị.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    /// Sẵn sàng
    Ready,
    /// Đang bận
    Busy,
    /// Lỗi
    Error,
    /// Tắt / không hoạt động
    Disabled,
    /// Chưa xác định
    Unknown,
}

/// Bus kết nối.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    /// I2C
    I2c,
    /// SPI
    Spi,
    /// UART
    Uart,
    /// USB
    Usb,
    /// PCI/PCIe (PC)
    Pcie,
    /// SDIO (WiFi trên mobile)
    Sdio,
    /// Internal (SoC integrated)
    Internal,
    /// Virtual (emulated)
    Virtual,
}

// ─────────────────────────────────────────────────────────────────────────────
// HalPlatform — trait chính
// ─────────────────────────────────────────────────────────────────────────────

/// HAL Platform trait — interface giữa HomeOS và phần cứng thật.
///
/// Mỗi platform (Linux, Android, ESP32, WASM) implement trait này.
/// Worker gọi trait methods — không biết đang chạy trên platform nào.
pub trait HalPlatform {
    /// Tên platform.
    fn name(&self) -> &str;

    /// Detect kiến trúc CPU.
    fn architecture(&self) -> Architecture;

    /// Lấy thông tin CPU.
    fn cpu_info(&self) -> CpuInfo;

    /// Lấy thông tin bộ nhớ.
    fn memory_info(&self) -> MemoryInfo;

    /// Scan thiết bị ngoại vi.
    fn scan_devices(&self) -> Vec<DeviceDescriptor>;

    /// Kiểm tra platform có capability hay không.
    fn has_capability(&self, cap: PlatformCapability) -> bool;

    /// Lấy danh sách capabilities.
    fn capabilities(&self) -> Vec<PlatformCapability>;

    /// Tổng hợp platform profile.
    fn profile(&self, ts: i64) -> PlatformProfile {
        let cpu = self.cpu_info();
        let arch = self.architecture();
        let memory = self.memory_info();
        let devices = self.scan_devices();
        let peripherals = devices.iter().map(|d| d.name.clone()).collect();

        PlatformProfile {
            cpu,
            chipset: ChipsetLayout::from_arch(arch),
            memory,
            os: String::from(self.name()),
            peripherals,
            probed_at: ts,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MockPlatform — test implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Mock platform cho testing — không cần phần cứng thật.
pub struct MockPlatform {
    /// Giả lập arch
    pub arch: Architecture,
    /// Giả lập devices
    pub devices: Vec<DeviceDescriptor>,
    /// Giả lập capabilities
    pub caps: Vec<PlatformCapability>,
    /// Giả lập RAM
    pub ram_bytes: u64,
    /// Giả lập cores
    pub cores: u8,
}

impl MockPlatform {
    /// Mock platform x86_64 PC.
    pub fn pc() -> Self {
        Self {
            arch: Architecture::X86_64,
            devices: alloc::vec![
                DeviceDescriptor {
                    id: String::from("eth0"),
                    name: String::from("Ethernet"),
                    device_type: DeviceType::Network,
                    status: DeviceStatus::Ready,
                    bus: BusType::Pcie,
                },
                DeviceDescriptor {
                    id: String::from("sda"),
                    name: String::from("SSD"),
                    device_type: DeviceType::Storage,
                    status: DeviceStatus::Ready,
                    bus: BusType::Pcie,
                },
            ],
            caps: alloc::vec![
                PlatformCapability::FileSystem,
                PlatformCapability::NetworkMon,
                PlatformCapability::ProcessList,
                PlatformCapability::Usb,
                PlatformCapability::Display,
                PlatformCapability::Audio,
            ],
            ram_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            cores: 8,
        }
    }

    /// Mock platform ARM64 smartphone.
    pub fn smartphone() -> Self {
        Self {
            arch: Architecture::Arm64,
            devices: alloc::vec![
                DeviceDescriptor {
                    id: String::from("cam0"),
                    name: String::from("Camera"),
                    device_type: DeviceType::Camera,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
                DeviceDescriptor {
                    id: String::from("wifi0"),
                    name: String::from("WiFi"),
                    device_type: DeviceType::Network,
                    status: DeviceStatus::Ready,
                    bus: BusType::Sdio,
                },
                DeviceDescriptor {
                    id: String::from("ble0"),
                    name: String::from("Bluetooth"),
                    device_type: DeviceType::Network,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
                DeviceDescriptor {
                    id: String::from("sensor_accel"),
                    name: String::from("Accelerometer"),
                    device_type: DeviceType::Sensor,
                    status: DeviceStatus::Ready,
                    bus: BusType::I2c,
                },
            ],
            caps: alloc::vec![
                PlatformCapability::Camera,
                PlatformCapability::NetworkMon,
                PlatformCapability::Bluetooth,
                PlatformCapability::Wifi,
                PlatformCapability::SensorRead,
                PlatformCapability::Display,
                PlatformCapability::Audio,
            ],
            ram_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            cores: 8,
        }
    }

    /// Mock platform ESP32 (IoT device).
    pub fn esp32() -> Self {
        Self {
            arch: Architecture::Xtensa,
            devices: alloc::vec![
                DeviceDescriptor {
                    id: String::from("gpio_dht22"),
                    name: String::from("DHT22 Temp/Humidity"),
                    device_type: DeviceType::Sensor,
                    status: DeviceStatus::Ready,
                    bus: BusType::I2c,
                },
                DeviceDescriptor {
                    id: String::from("gpio_relay"),
                    name: String::from("Relay Module"),
                    device_type: DeviceType::Actuator,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
            ],
            caps: alloc::vec![
                PlatformCapability::SensorRead,
                PlatformCapability::ActuatorCtrl,
                PlatformCapability::Gpio,
                PlatformCapability::I2c,
                PlatformCapability::Spi,
                PlatformCapability::Uart,
                PlatformCapability::Wifi,
            ],
            ram_bytes: 520 * 1024, // 520KB SRAM
            cores: 2,
        }
    }

    /// Mock platform RISC-V embedded.
    pub fn riscv_embedded() -> Self {
        Self {
            arch: Architecture::RiscV32,
            devices: alloc::vec![
                DeviceDescriptor {
                    id: String::from("i2c_bme280"),
                    name: String::from("BME280 Environment"),
                    device_type: DeviceType::Sensor,
                    status: DeviceStatus::Ready,
                    bus: BusType::I2c,
                },
            ],
            caps: alloc::vec![
                PlatformCapability::SensorRead,
                PlatformCapability::Gpio,
                PlatformCapability::I2c,
                PlatformCapability::Uart,
            ],
            ram_bytes: 256 * 1024, // 256KB
            cores: 1,
        }
    }

    /// Mock platform Raspberry Pi (ARM64 Linux).
    pub fn raspberry_pi() -> Self {
        Self {
            arch: Architecture::Arm64,
            devices: alloc::vec![
                DeviceDescriptor {
                    id: String::from("gpio"),
                    name: String::from("GPIO Header"),
                    device_type: DeviceType::GpioPin,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
                DeviceDescriptor {
                    id: String::from("cam_csi"),
                    name: String::from("CSI Camera"),
                    device_type: DeviceType::Camera,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
                DeviceDescriptor {
                    id: String::from("eth0"),
                    name: String::from("Ethernet"),
                    device_type: DeviceType::Network,
                    status: DeviceStatus::Ready,
                    bus: BusType::Internal,
                },
                DeviceDescriptor {
                    id: String::from("wlan0"),
                    name: String::from("WiFi"),
                    device_type: DeviceType::Network,
                    status: DeviceStatus::Ready,
                    bus: BusType::Sdio,
                },
            ],
            caps: alloc::vec![
                PlatformCapability::Gpio,
                PlatformCapability::I2c,
                PlatformCapability::Spi,
                PlatformCapability::Uart,
                PlatformCapability::Camera,
                PlatformCapability::NetworkMon,
                PlatformCapability::FileSystem,
                PlatformCapability::ProcessList,
                PlatformCapability::Usb,
                PlatformCapability::Bluetooth,
                PlatformCapability::Wifi,
                PlatformCapability::Display,
                PlatformCapability::Audio,
            ],
            ram_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            cores: 4,
        }
    }
}

impl HalPlatform for MockPlatform {
    fn name(&self) -> &str { "mock" }

    fn architecture(&self) -> Architecture { self.arch }

    fn cpu_info(&self) -> CpuInfo {
        CpuInfo {
            arch: self.arch,
            cores: self.cores,
            clock_mhz: 0,
            vendor: String::from("mock"),
            model: String::from("mock"),
            has_simd: self.arch.bits() == 64,
            has_crypto: matches!(self.arch, Architecture::X86_64 | Architecture::Arm64),
        }
    }

    fn memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total_bytes: self.ram_bytes,
            available_bytes: self.ram_bytes / 2,
            storage_bytes: 0,
        }
    }

    fn scan_devices(&self) -> Vec<DeviceDescriptor> {
        self.devices.clone()
    }

    fn has_capability(&self, cap: PlatformCapability) -> bool {
        self.caps.contains(&cap)
    }

    fn capabilities(&self) -> Vec<PlatformCapability> {
        self.caps.clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_pc_profile() {
        let platform = MockPlatform::pc();
        let profile = platform.profile(1000);
        assert_eq!(profile.cpu.arch, Architecture::X86_64);
        assert_eq!(profile.chipset, ChipsetLayout::Pch);
        assert_eq!(profile.cpu.cores, 8);
        assert!(profile.peripherals.len() >= 2);
    }

    #[test]
    fn mock_smartphone_profile() {
        let platform = MockPlatform::smartphone();
        let profile = platform.profile(1000);
        assert_eq!(profile.cpu.arch, Architecture::Arm64);
        assert_eq!(profile.chipset, ChipsetLayout::Soc);
        assert!(platform.has_capability(PlatformCapability::Camera));
        assert!(platform.has_capability(PlatformCapability::Bluetooth));
        assert!(!platform.has_capability(PlatformCapability::Gpio), "Phone thường không expose GPIO");
    }

    #[test]
    fn mock_esp32_profile() {
        let platform = MockPlatform::esp32();
        let profile = platform.profile(1000);
        assert_eq!(profile.cpu.arch, Architecture::Xtensa);
        assert_eq!(profile.chipset, ChipsetLayout::Mcu);
        assert!(platform.has_capability(PlatformCapability::Gpio));
        assert!(platform.has_capability(PlatformCapability::I2c));
        assert!(!platform.has_capability(PlatformCapability::FileSystem), "ESP32 không có filesystem");
        assert!(profile.memory.total_bytes < 1024 * 1024, "ESP32 < 1MB RAM");
    }

    #[test]
    fn mock_riscv_profile() {
        let platform = MockPlatform::riscv_embedded();
        let profile = platform.profile(1000);
        assert_eq!(profile.cpu.arch, Architecture::RiscV32);
        assert_eq!(profile.chipset, ChipsetLayout::Mcu);
        assert_eq!(profile.cpu.cores, 1);
    }

    #[test]
    fn mock_rpi_profile() {
        let platform = MockPlatform::raspberry_pi();
        let profile = platform.profile(1000);
        assert_eq!(profile.cpu.arch, Architecture::Arm64);
        assert!(platform.has_capability(PlatformCapability::Gpio));
        assert!(platform.has_capability(PlatformCapability::Camera));
        assert!(platform.has_capability(PlatformCapability::NetworkMon));
        assert_eq!(profile.cpu.cores, 4);
    }

    #[test]
    fn device_descriptor_types() {
        let platform = MockPlatform::smartphone();
        let devices = platform.scan_devices();
        let camera = devices.iter().find(|d| d.device_type == DeviceType::Camera);
        assert!(camera.is_some(), "Smartphone phải có camera");
        let cam = camera.unwrap();
        assert_eq!(cam.status, DeviceStatus::Ready);
    }

    #[test]
    fn profile_summary_format() {
        let platform = MockPlatform::pc();
        let profile = platform.profile(1000);
        let s = profile.summary();
        assert!(s.contains("x86-64"), "{}", s);
        assert!(s.contains("PCH"), "{}", s);
    }

    #[test]
    fn capabilities_list() {
        let platform = MockPlatform::esp32();
        let caps = platform.capabilities();
        assert!(caps.contains(&PlatformCapability::SensorRead));
        assert!(caps.contains(&PlatformCapability::Wifi));
        assert!(!caps.contains(&PlatformCapability::Camera));
    }

    #[test]
    fn default_profile_impl() {
        // Test default trait method profile()
        let platform = MockPlatform::pc();
        let profile = platform.profile(42);
        assert_eq!(profile.probed_at, 42);
        assert_eq!(profile.os, "mock");
    }
}
