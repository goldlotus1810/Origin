//! # worker — Worker Agent
//!
//! Worker = HomeOS thu nhỏ tại thiết bị.
//! KHÔNG phải adapter — LÀ HomeOS tại thiết bị.
//!
//! Nguyên tắc:
//!   Silent by default — wake on ISL message
//!   Xử lý local → encode → báo cáo molecular chain (không raw data)
//!   Chief nhận chain → decode ngay → hiểu ngay
//!   Im lặng khi không có gì
//!
//! Worker types:
//!   Sensor    — đọc sensor → encode → report
//!   Actuator  — nhận cmd → execute → ack
//!   Camera    — frame → FFR → chain → report
//!   Network   — packet → detect → alert

extern crate alloc;
use alloc::vec::Vec;
use isl::address::ISLAddress;
use isl::message::{ISLFrame, ISLMessage, MsgType};
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// WorkerKind
// ─────────────────────────────────────────────────────────────────────────────

/// Loại Worker.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkerKind {
    Sensor = 0x01,   // đọc sensor
    Actuator = 0x02, // điều khiển thiết bị
    Camera = 0x03,   // xử lý hình ảnh
    Network = 0x04,  // mạng/bảo mật
    Generic = 0xFF,
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkerState
// ─────────────────────────────────────────────────────────────────────────────

/// Trạng thái của Worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    /// Im lặng, chờ event
    Sleeping,
    /// Đang xử lý
    Active,
    /// Lỗi
    Error,
}

// ─────────────────────────────────────────────────────────────────────────────
// SensorReading — input thô từ sensor
// ─────────────────────────────────────────────────────────────────────────────

/// Đọc từ sensor.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_id: u8,
    pub value: f32,
    pub unit: SensorUnit,
    pub timestamp: i64,
}

/// Đơn vị đo.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorUnit {
    Temperature = 0x01, // °C
    Humidity = 0x02,    // %
    Light = 0x03,       // lux
    Motion = 0x04,      // 0/1
    Sound = 0x05,       // dB
    Distance = 0x06,    // cm
    Pressure = 0x07,    // hPa
    Custom = 0xFF,
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkerEvent — sự kiện Worker nhận/tạo ra
// ─────────────────────────────────────────────────────────────────────────────

/// Sự kiện Worker.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Sensor có reading mới
    SensorData(SensorReading),
    /// Nhận ISL command từ Chief
    ISLCommand(ISLMessage),
    /// Camera frame (motion_score, brightness)
    CameraFrame { motion_score: f32, brightness: f32 },
    /// Network packet summary (bytes_in, anomaly_score 0.0..1.0)
    NetworkPacket { bytes_in: u32, anomaly_score: f32 },
    /// Timer tick (heartbeat nội bộ)
    Tick { ts: i64 },
    /// Wake up từ sleep
    Wake,
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkerReport — báo cáo gửi lên Chief
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả xử lý — gửi lên Chief dưới dạng ISLFrame.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct WorkerReport {
    pub frame: ISLFrame,
    pub emotion: EmotionTag, // cảm xúc của sự kiện (VD: nhiệt độ cao → arousal cao)
}

// ─────────────────────────────────────────────────────────────────────────────
// Worker
// ─────────────────────────────────────────────────────────────────────────────

/// Worker Agent — HomeOS thu nhỏ tại thiết bị.
///
/// Worker profiles (từ spec):
///   Sensor   = L0 + SensorSkill
///   Actuator = L0 + ActuatorSkill (door thêm SecuritySkill)
///   Camera   = L0 + FFR + vSDF + InverseRenderSkill
///   Network  = L0 + NetworkSkill + ImmunitySkill
#[allow(missing_docs)]
pub struct Worker {
    pub addr: ISLAddress,
    pub chief: ISLAddress,
    pub kind: WorkerKind,
    pub state: WorkerState,
    /// Outbox — chờ gửi lên Chief
    pub outbox: Vec<WorkerReport>,
    /// ISL inbox — commands from Chief via ISL
    inbox: Vec<ISLFrame>,
    /// Event count
    pub events: u32,
    /// Door worker: security locked state
    pub security_locked: bool,
    /// Network worker: cumulative anomaly score
    pub anomaly_accumulator: f32,
    /// Camera worker: consecutive motion frames
    pub motion_streak: u32,
}

impl Worker {
    /// Tạo Worker mới.
    pub fn new(addr: ISLAddress, chief: ISLAddress, kind: WorkerKind) -> Self {
        Self {
            addr,
            chief,
            kind,
            state: WorkerState::Sleeping,
            outbox: Vec::new(),
            inbox: Vec::new(),
            events: 0,
            security_locked: false,
            anomaly_accumulator: 0.0,
            motion_streak: 0,
        }
    }

    /// Xử lý một event.
    ///
    /// Quy trình:
    ///   1. Wake
    ///   2. Encode sự kiện → MolecularChain
    ///   3. Tính EmotionTag từ context
    ///   4. Đóng gói ISLFrame
    ///   5. Push vào outbox
    ///   6. Sleep lại
    pub fn process(&mut self, event: WorkerEvent, ts: i64) {
        self.state = WorkerState::Active;
        self.events += 1;

        match event {
            WorkerEvent::SensorData(reading) => {
                self.process_sensor(reading, ts);
            }
            WorkerEvent::ISLCommand(msg) => {
                self.process_command(msg, ts);
            }
            WorkerEvent::CameraFrame {
                motion_score,
                brightness,
            } => {
                self.process_camera(motion_score, brightness, ts);
            }
            WorkerEvent::NetworkPacket {
                bytes_in,
                anomaly_score,
            } => {
                self.process_network(bytes_in, anomaly_score, ts);
            }
            WorkerEvent::Tick { ts } => {
                // Silent = không gửi tick rỗng
                let _ = ts;
            }
            WorkerEvent::Wake => {}
        }

        self.state = WorkerState::Sleeping; // sleep ngay sau khi xử lý
    }

    /// Flush outbox — lấy tất cả reports để gửi.
    pub fn flush(&mut self) -> Vec<WorkerReport> {
        let mut out = Vec::new();
        core::mem::swap(&mut self.outbox, &mut out);
        out
    }

    /// Có reports chờ gửi không.
    pub fn has_reports(&self) -> bool {
        !self.outbox.is_empty()
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn process_sensor(&mut self, reading: SensorReading, _ts: i64) {
        // Encode reading → payload bytes (không raw float — encode thành 3B)
        let payload = encode_sensor_payload(&reading);
        let emotion = sensor_emotion(&reading);

        let msg = ISLMessage::with_payload(self.addr, self.chief, MsgType::ChainPayload, payload);
        // Body = sensor metadata: [sensor_id, unit, value_hi, value_lo]
        let body = encode_sensor_body(&reading);
        let frame = ISLFrame::with_body(msg, body);

        self.outbox.push(WorkerReport { frame, emotion });
    }

    fn process_command(&mut self, cmd: ISLMessage, _ts: i64) {
        if cmd.msg_type != MsgType::ActuatorCmd {
            return;
        }

        // Door worker: security check trước khi thực thi
        if self.kind == WorkerKind::Actuator && self.security_locked {
            // Reject — gửi NACK thay vì ACK
            let nack = ISLMessage::nack(self.addr, cmd.from, MsgType::ActuatorCmd);
            let frame = ISLFrame::bare(nack);
            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag {
                    valence: -0.50,
                    arousal: 0.60,
                    dominance: 0.80,
                    intensity: 0.50,
                },
            });
            return;
        }

        let ack = ISLMessage::ack(self.addr, cmd.from, MsgType::ActuatorCmd);
        let frame = ISLFrame::bare(ack);
        self.outbox.push(WorkerReport {
            frame,
            emotion: EmotionTag::NEUTRAL,
        });
    }

    /// Camera worker: process frame, detect motion.
    ///
    /// Profile: L0 + FFR + vSDF + InverseRenderSkill
    /// Worker gửi motion chain (molecular) — KHÔNG raw pixels.
    fn process_camera(&mut self, motion_score: f32, brightness: f32, ts: i64) {
        if self.kind != WorkerKind::Camera {
            return;
        }

        // Track motion streak
        if motion_score > 0.3 {
            self.motion_streak += 1;
        } else {
            self.motion_streak = 0;
        }

        // Chỉ gửi report khi có motion đáng kể (threshold: score > 0.3)
        // Silent by default — không gửi frame rỗng
        if motion_score <= 0.3 {
            return;
        }

        let payload = [
            self.addr.index,              // camera id
            0x04,                         // unit = Motion
            (motion_score * 255.0) as u8, // quantized motion
        ];
        let msg = ISLMessage::with_payload(self.addr, self.chief, MsgType::ChainPayload, payload);

        // Body: motion_score(f32) + brightness(f32) + streak(u32) + timestamp(i64)
        let mut body = Vec::with_capacity(20);
        body.push(self.addr.index); // sensor_id
        body.push(0x04); // unit = Motion
        body.extend_from_slice(&motion_score.to_be_bytes());
        body.extend_from_slice(&brightness.to_be_bytes());
        body.extend_from_slice(&self.motion_streak.to_be_bytes());
        body.extend_from_slice(&ts.to_be_bytes());
        let frame = ISLFrame::with_body(msg, body);

        let emotion = if self.motion_streak > 5 {
            // Sustained motion → higher urgency
            EmotionTag {
                valence: -0.20,
                arousal: 0.85,
                dominance: 0.40,
                intensity: 0.75,
            }
        } else {
            EmotionTag {
                valence: 0.0,
                arousal: 0.60,
                dominance: 0.50,
                intensity: 0.45,
            }
        };

        self.outbox.push(WorkerReport { frame, emotion });
    }

    /// Network worker: monitor traffic, detect anomalies.
    ///
    /// Profile: L0 + NetworkSkill + ImmunitySkill
    /// anomaly_score > 0.7 → Emergency (immediate escalation)
    /// anomaly_score > 0.4 → report to Chief
    /// anomaly_score ≤ 0.4 → silent
    fn process_network(&mut self, bytes_in: u32, anomaly_score: f32, _ts: i64) {
        if self.kind != WorkerKind::Network {
            return;
        }

        // Accumulate anomaly (exponential moving average)
        self.anomaly_accumulator = self.anomaly_accumulator * 0.7 + anomaly_score * 0.3;

        // Emergency: high anomaly → immediate alert
        if anomaly_score > 0.7 {
            let msg = ISLMessage::emergency(self.addr, (anomaly_score * 255.0) as u8);
            let frame = ISLFrame::bare(msg);
            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag {
                    valence: -0.70,
                    arousal: 0.95,
                    dominance: 0.20,
                    intensity: 0.90,
                },
            });
            return;
        }

        // Moderate anomaly → report to Chief
        if anomaly_score > 0.4 {
            let payload = [
                self.addr.index,
                0xFF, // network unit
                (anomaly_score * 255.0) as u8,
            ];
            let msg =
                ISLMessage::with_payload(self.addr, self.chief, MsgType::ChainPayload, payload);
            let mut body = Vec::with_capacity(12);
            body.push(self.addr.index);
            body.push(0xFF);
            body.extend_from_slice(&anomaly_score.to_be_bytes());
            body.extend_from_slice(&bytes_in.to_be_bytes());
            let frame = ISLFrame::with_body(msg, body);

            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag {
                    valence: -0.30,
                    arousal: 0.65,
                    dominance: 0.35,
                    intensity: 0.55,
                },
            });
        }
        // ≤ 0.4 → silent (no report)
    }

    // ── ISL Inbox ──────────────────────────────────────────────────────────

    /// Receive an ISL frame into the inbox (called by Chief via router).
    pub fn receive_isl(&mut self, frame: ISLFrame) {
        self.inbox.push(frame);
    }

    /// Poll inbox — process all pending ISL commands.
    ///
    /// Drains inbox, dispatches each frame as WorkerEvent::ISLCommand.
    /// Returns number of commands processed.
    pub fn poll_inbox(&mut self, ts: i64) -> u32 {
        if self.inbox.is_empty() {
            return 0;
        }
        let frames: Vec<ISLFrame> = core::mem::take(&mut self.inbox);
        let count = frames.len() as u32;
        for frame in frames {
            self.process(WorkerEvent::ISLCommand(frame.header), ts);
        }
        count
    }

    /// Number of pending ISL messages in inbox.
    pub fn inbox_len(&self) -> usize {
        self.inbox.len()
    }

    /// Lock/unlock security (door worker).
    pub fn set_security_lock(&mut self, locked: bool) {
        self.security_locked = locked;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Encoding helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Encode sensor value → 3 bytes payload.
///
/// [0] = sensor_id
/// [1] = unit
/// [2] = value quantized 0..255 (0=min, 255=max theo unit)
fn encode_sensor_payload(r: &SensorReading) -> [u8; 3] {
    let quantized = quantize_value(r.value, r.unit);
    [r.sensor_id, r.unit as u8, quantized]
}

/// Encode sensor body (chi tiết hơn payload).
fn encode_sensor_body(r: &SensorReading) -> Vec<u8> {
    let f_bytes = r.value.to_be_bytes();
    let ts_bytes = r.timestamp.to_be_bytes();
    let mut b = Vec::with_capacity(13);
    b.push(r.sensor_id);
    b.push(r.unit as u8);
    b.extend_from_slice(&f_bytes);
    b.extend_from_slice(&ts_bytes);
    b
}

/// Quantize sensor value → 0..255 theo unit.
fn quantize_value(val: f32, unit: SensorUnit) -> u8 {
    let (min, max) = match unit {
        SensorUnit::Temperature => (-40.0, 85.0),
        SensorUnit::Humidity => (0.0, 100.0),
        SensorUnit::Light => (0.0, 10000.0),
        SensorUnit::Motion => (0.0, 1.0),
        SensorUnit::Sound => (0.0, 120.0),
        SensorUnit::Distance => (0.0, 500.0),
        SensorUnit::Pressure => (800.0, 1200.0),
        SensorUnit::Custom => (0.0, 255.0),
    };
    let norm = ((val - min) / (max - min)).clamp(0.0, 1.0);
    (norm * 255.0) as u8
}

/// Tính EmotionTag từ sensor reading.
///
/// Nguyên tắc: cảm xúc của sự kiện vật lý
///   Nhiệt độ cao → arousal cao
///   Chuyển động → arousal cao, valence neutral
///   Tiếng ồn lớn → arousal cao, valence thấp
fn sensor_emotion(r: &SensorReading) -> EmotionTag {
    match r.unit {
        SensorUnit::Temperature => {
            let temp = r.value;
            if temp > 35.0 {
                EmotionTag {
                    valence: -0.30,
                    arousal: 0.70,
                    dominance: 0.30,
                    intensity: 0.55,
                }
            } else if temp < 10.0 {
                EmotionTag {
                    valence: -0.20,
                    arousal: 0.60,
                    dominance: 0.30,
                    intensity: 0.45,
                }
            } else {
                EmotionTag {
                    valence: 0.10,
                    arousal: 0.30,
                    dominance: 0.60,
                    intensity: 0.15,
                }
            }
        }
        SensorUnit::Motion => {
            if r.value > 0.5 {
                EmotionTag {
                    valence: 0.0,
                    arousal: 0.75,
                    dominance: 0.50,
                    intensity: 0.60,
                }
            } else {
                EmotionTag::NEUTRAL
            }
        }
        SensorUnit::Sound => {
            let db = r.value;
            if db > 80.0 {
                EmotionTag {
                    valence: -0.40,
                    arousal: 0.85,
                    dominance: 0.25,
                    intensity: 0.70,
                }
            } else if db > 60.0 {
                EmotionTag {
                    valence: -0.10,
                    arousal: 0.50,
                    dominance: 0.50,
                    intensity: 0.30,
                }
            } else {
                EmotionTag::NEUTRAL
            }
        }
        _ => EmotionTag::NEUTRAL,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DriverProbe — kiểm tra phần cứng
// ─────────────────────────────────────────────────────────────────────────────

/// Trạng thái phần cứng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareStatus {
    /// Phần cứng sẵn sàng
    Ready,
    /// Phần cứng không phản hồi
    Unreachable,
    /// Phần cứng lỗi (error code)
    Error(u8),
    /// Chưa kiểm tra
    Unknown,
}

/// Kết quả probe phần cứng.
#[derive(Debug, Clone)]
pub struct DriverProbeResult {
    /// Loại Worker
    pub kind: WorkerKind,
    /// Trạng thái
    pub status: HardwareStatus,
    /// Tên driver / device ID
    pub device_id: alloc::string::String,
    /// Capabilities: sensor units hỗ trợ, actuator types, etc.
    pub capabilities: Vec<u8>,
    /// Timestamp khi probe
    pub probed_at: i64,
}

/// DriverProbe — Worker kiểm tra phần cứng của mình khi boot.
///
/// Mỗi Worker type khai báo hardware cần → probe → report lên Chief.
/// Chief tổng hợp → SystemManifest biết thiết bị nào đang sống/chết.
///
/// Hai chế độ:
///   1. Software probe (hiện tại): từ internal state
///   2. HAL probe: từ HalPlatform trait → phần cứng thật
impl Worker {
    /// Probe phần cứng — Worker tự kiểm tra device mình quản lý.
    ///
    /// Trả DriverProbeResult cho Chief tổng hợp.
    /// Trong môi trường thật: gọi HAL interface.
    /// Hiện tại: report từ internal state (software probe).
    pub fn probe_hardware(&self, ts: i64) -> DriverProbeResult {
        let device_id = match self.kind {
            WorkerKind::Sensor => alloc::format!("sensor@{}", self.addr),
            WorkerKind::Actuator => alloc::format!("actuator@{}", self.addr),
            WorkerKind::Camera => alloc::format!("camera@{}", self.addr),
            WorkerKind::Network => alloc::format!("network@{}", self.addr),
            WorkerKind::Generic => alloc::format!("generic@{}", self.addr),
        };

        let capabilities = match self.kind {
            WorkerKind::Sensor => alloc::vec![
                SensorUnit::Temperature as u8,
                SensorUnit::Humidity as u8,
                SensorUnit::Light as u8,
                SensorUnit::Motion as u8,
                SensorUnit::Sound as u8,
            ],
            WorkerKind::Actuator => alloc::vec![0x01, 0x02], // on/off, dim
            WorkerKind::Camera => alloc::vec![0x01],         // video stream
            WorkerKind::Network => alloc::vec![0x01, 0x02],  // monitor, firewall
            WorkerKind::Generic => alloc::vec![],
        };

        let status = match self.state {
            WorkerState::Sleeping | WorkerState::Active => HardwareStatus::Ready,
            WorkerState::Error => HardwareStatus::Error(0x01),
        };

        DriverProbeResult {
            kind: self.kind,
            status,
            device_id,
            capabilities,
            probed_at: ts,
        }
    }

    /// Encode probe result → ISLFrame để gửi lên Chief.
    pub fn probe_report(&self, ts: i64) -> WorkerReport {
        let probe = self.probe_hardware(ts);
        let status_byte = match probe.status {
            HardwareStatus::Ready => 0x01,
            HardwareStatus::Unreachable => 0x02,
            HardwareStatus::Error(c) => 0x80 | c,
            HardwareStatus::Unknown => 0x00,
        };

        let mut body = Vec::new();
        body.push(self.kind as u8);
        body.push(status_byte);
        body.push(probe.capabilities.len() as u8);
        body.extend_from_slice(&probe.capabilities);

        let msg = ISLMessage::new(self.addr, self.chief, MsgType::Ack);
        let frame = ISLFrame { header: msg, body };

        WorkerReport {
            frame,
            emotion: EmotionTag::NEUTRAL,
        }
    }

    /// HAL probe — probe qua HalPlatform trait (phần cứng thật hoặc mock).
    ///
    /// Trả DriverProbeResult với capabilities từ platform HAL thay vì hardcode.
    pub fn probe_with_hal(&self, platform: &dyn hal::HalPlatform, ts: i64) -> DriverProbeResult {
        let devices = platform.scan_devices();

        let device_id = alloc::format!("{}@{}", self.kind_name(), self.addr);

        // Map platform capabilities → Worker capabilities
        let capabilities = match self.kind {
            WorkerKind::Sensor => {
                let mut caps = Vec::new();
                if platform.has_capability(hal::PlatformCapability::SensorRead) {
                    caps.push(SensorUnit::Temperature as u8);
                    caps.push(SensorUnit::Humidity as u8);
                }
                if platform.has_capability(hal::PlatformCapability::I2c) {
                    caps.push(SensorUnit::Pressure as u8);
                }
                if platform.has_capability(hal::PlatformCapability::Gpio) {
                    caps.push(SensorUnit::Motion as u8);
                }
                caps
            }
            WorkerKind::Actuator => {
                let mut caps = Vec::new();
                if platform.has_capability(hal::PlatformCapability::ActuatorCtrl) {
                    caps.push(0x01); // on/off
                    caps.push(0x02); // dim
                }
                if platform.has_capability(hal::PlatformCapability::Gpio) {
                    caps.push(0x03); // gpio toggle
                }
                caps
            }
            WorkerKind::Camera => {
                let mut caps = Vec::new();
                if platform.has_capability(hal::PlatformCapability::Camera) {
                    caps.push(0x01); // video
                }
                caps
            }
            WorkerKind::Network => {
                let mut caps = Vec::new();
                if platform.has_capability(hal::PlatformCapability::NetworkMon) {
                    caps.push(0x01); // monitor
                }
                if platform.has_capability(hal::PlatformCapability::Wifi) {
                    caps.push(0x02); // wifi
                }
                if platform.has_capability(hal::PlatformCapability::Bluetooth) {
                    caps.push(0x03); // bluetooth
                }
                caps
            }
            WorkerKind::Generic => Vec::new(),
        };

        // Status: check if relevant devices exist and are ready
        let status = self.check_device_status(&devices);

        DriverProbeResult {
            kind: self.kind,
            status,
            device_id,
            capabilities,
            probed_at: ts,
        }
    }

    /// HAL probe report → ISLFrame (gửi lên Chief).
    pub fn probe_report_hal(&self, platform: &dyn hal::HalPlatform, ts: i64) -> WorkerReport {
        let probe = self.probe_with_hal(platform, ts);
        let status_byte = match probe.status {
            HardwareStatus::Ready => 0x01,
            HardwareStatus::Unreachable => 0x02,
            HardwareStatus::Error(c) => 0x80 | c,
            HardwareStatus::Unknown => 0x00,
        };

        let mut body = Vec::new();
        body.push(self.kind as u8);
        body.push(status_byte);
        body.push(probe.capabilities.len() as u8);
        body.extend_from_slice(&probe.capabilities);

        // Thêm arch byte cho Chief biết platform
        let arch = platform.architecture();
        body.push(arch as u8);

        let msg = ISLMessage::new(self.addr, self.chief, MsgType::Ack);
        let frame = ISLFrame { header: msg, body };

        WorkerReport {
            frame,
            emotion: EmotionTag::NEUTRAL,
        }
    }

    /// Tên loại Worker.
    fn kind_name(&self) -> &'static str {
        match self.kind {
            WorkerKind::Sensor => "sensor",
            WorkerKind::Actuator => "actuator",
            WorkerKind::Camera => "camera",
            WorkerKind::Network => "network",
            WorkerKind::Generic => "generic",
        }
    }

    /// Kiểm tra status từ danh sách devices của platform.
    fn check_device_status(&self, devices: &[hal::DeviceDescriptor]) -> HardwareStatus {
        let target_type = match self.kind {
            WorkerKind::Sensor => hal::DeviceType::Sensor,
            WorkerKind::Actuator => hal::DeviceType::Actuator,
            WorkerKind::Camera => hal::DeviceType::Camera,
            WorkerKind::Network => hal::DeviceType::Network,
            WorkerKind::Generic => return HardwareStatus::Unknown,
        };

        let relevant: Vec<&hal::DeviceDescriptor> = devices
            .iter()
            .filter(|d| d.device_type == target_type)
            .collect();

        if relevant.is_empty() {
            return HardwareStatus::Unreachable;
        }

        // Nếu tất cả error → Error, nếu có ít nhất 1 Ready → Ready
        let any_ready = relevant
            .iter()
            .any(|d| d.status == hal::DeviceStatus::Ready);
        let any_error = relevant
            .iter()
            .any(|d| d.status == hal::DeviceStatus::Error);

        if any_ready {
            HardwareStatus::Ready
        } else if any_error {
            HardwareStatus::Error(0x01)
        } else {
            HardwareStatus::Unknown
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SensorNormalizer — EMA smoothing + calibration
// ─────────────────────────────────────────────────────────────────────────────

/// Sensor normalization configuration per unit type.
#[derive(Debug, Clone, Copy)]
pub struct SensorCalibration {
    /// Physical minimum for this sensor type
    pub min: f32,
    /// Physical maximum for this sensor type
    pub max: f32,
    /// EMA smoothing factor α ∈ (0, 1]. Higher = more responsive.
    pub ema_alpha: f32,
    /// Minimum change (absolute) to trigger a report (dead band)
    pub dead_band: f32,
}

impl SensorCalibration {
    /// Default calibration for a sensor unit.
    pub fn for_unit(unit: SensorUnit) -> Self {
        match unit {
            SensorUnit::Temperature => Self {
                min: -40.0,
                max: 85.0,
                ema_alpha: 0.3,
                dead_band: 0.5, // 0.5°C change minimum
            },
            SensorUnit::Humidity => Self {
                min: 0.0,
                max: 100.0,
                ema_alpha: 0.3,
                dead_band: 1.0, // 1% change
            },
            SensorUnit::Light => Self {
                min: 0.0,
                max: 10000.0,
                ema_alpha: 0.4,
                dead_band: 50.0, // 50 lux
            },
            SensorUnit::Motion => Self {
                min: 0.0,
                max: 1.0,
                ema_alpha: 0.5, // responsive
                dead_band: 0.1,
            },
            SensorUnit::Sound => Self {
                min: 0.0,
                max: 120.0,
                ema_alpha: 0.4,
                dead_band: 2.0, // 2 dB
            },
            SensorUnit::Distance => Self {
                min: 0.0,
                max: 500.0,
                ema_alpha: 0.3,
                dead_band: 5.0, // 5 cm
            },
            SensorUnit::Pressure => Self {
                min: 800.0,
                max: 1200.0,
                ema_alpha: 0.2, // slow — pressure changes slowly
                dead_band: 1.0, // 1 hPa
            },
            SensorUnit::Custom => Self {
                min: 0.0,
                max: 255.0,
                ema_alpha: 0.3,
                dead_band: 1.0,
            },
        }
    }
}

/// Per-sensor smoothing state.
#[derive(Debug, Clone)]
struct SensorState {
    /// EMA-smoothed value
    smoothed: f32,
    /// Last reported value (for dead band comparison)
    last_reported: f32,
    /// Number of samples received
    sample_count: u32,
    /// Calibration config
    cal: SensorCalibration,
}

/// Sensor normalizer: EMA smoothing + dead-band filtering + calibration.
///
/// Buffers raw sensor readings, applies exponential moving average,
/// and only emits when the smoothed value changes beyond the dead band.
/// This prevents noisy sensors from flooding the pipeline with reports.
pub struct SensorNormalizer {
    /// Per-sensor states indexed by (sensor_id, unit)
    states: Vec<(u8, SensorUnit, SensorState)>,
}

impl SensorNormalizer {
    /// Create empty normalizer.
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// Process a raw reading → returns smoothed value and whether to report.
    ///
    /// Returns `(smoothed_value, should_report)`:
    /// - `smoothed_value`: EMA-filtered value
    /// - `should_report`: true if change exceeds dead band
    pub fn process(&mut self, reading: &SensorReading) -> (f32, bool) {
        let state = self.get_or_create(reading.sensor_id, reading.unit);

        if state.sample_count == 0 {
            // First sample: initialize
            state.smoothed = reading.value;
            state.last_reported = reading.value;
            state.sample_count = 1;
            return (reading.value, true); // Always report first reading
        }

        // EMA: smoothed = α × new + (1 - α) × old
        let alpha = state.cal.ema_alpha;
        state.smoothed = alpha * reading.value + (1.0 - alpha) * state.smoothed;
        state.sample_count += 1;

        // Dead band: only report if smoothed value changed enough
        let delta = (state.smoothed - state.last_reported).abs();
        let should_report = delta >= state.cal.dead_band;
        if should_report {
            state.last_reported = state.smoothed;
        }

        (state.smoothed, should_report)
    }

    /// Normalize a value to [0.0, 1.0] range using calibration bounds.
    pub fn normalize(&self, sensor_id: u8, unit: SensorUnit, value: f32) -> f32 {
        let cal = self
            .states
            .iter()
            .find(|(id, u, _)| *id == sensor_id && *u == unit)
            .map(|(_, _, s)| s.cal)
            .unwrap_or_else(|| SensorCalibration::for_unit(unit));
        ((value - cal.min) / (cal.max - cal.min)).clamp(0.0, 1.0)
    }

    /// Get sample count for a sensor.
    pub fn sample_count(&self, sensor_id: u8, unit: SensorUnit) -> u32 {
        self.states
            .iter()
            .find(|(id, u, _)| *id == sensor_id && *u == unit)
            .map(|(_, _, s)| s.sample_count)
            .unwrap_or(0)
    }

    /// Get smoothed value for a sensor (None if never seen).
    pub fn smoothed_value(&self, sensor_id: u8, unit: SensorUnit) -> Option<f32> {
        self.states
            .iter()
            .find(|(id, u, _)| *id == sensor_id && *u == unit)
            .map(|(_, _, s)| s.smoothed)
    }

    /// Override calibration for a specific sensor.
    pub fn set_calibration(&mut self, sensor_id: u8, unit: SensorUnit, cal: SensorCalibration) {
        let state = self.get_or_create(sensor_id, unit);
        state.cal = cal;
    }

    fn get_or_create(&mut self, sensor_id: u8, unit: SensorUnit) -> &mut SensorState {
        let idx = self
            .states
            .iter()
            .position(|(id, u, _)| *id == sensor_id && *u == unit);
        if let Some(i) = idx {
            &mut self.states[i].2
        } else {
            self.states.push((
                sensor_id,
                unit,
                SensorState {
                    smoothed: 0.0,
                    last_reported: 0.0,
                    sample_count: 0,
                    cal: SensorCalibration::for_unit(unit),
                },
            ));
            let last = self.states.len() - 1;
            &mut self.states[last].2
        }
    }
}

impl Default for SensorNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn chief() -> ISLAddress {
        ISLAddress::new(0, 0, 0, 0)
    }
    fn waddr() -> ISLAddress {
        ISLAddress::new(1, 5, 0, 1)
    }

    fn worker() -> Worker {
        Worker::new(waddr(), chief(), WorkerKind::Sensor)
    }

    fn temp_reading(val: f32) -> SensorReading {
        SensorReading {
            sensor_id: 1,
            value: val,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        }
    }

    // ── Worker state ──────────────────────────────────────────────────────────

    #[test]
    fn worker_starts_sleeping() {
        let w = worker();
        assert_eq!(w.state, WorkerState::Sleeping);
        assert_eq!(w.events, 0);
    }

    #[test]
    fn worker_sleeps_after_event() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(25.0)), 1000);
        assert_eq!(
            w.state,
            WorkerState::Sleeping,
            "Worker sleep ngay sau khi xử lý"
        );
    }

    #[test]
    fn worker_silent_on_tick() {
        let mut w = worker();
        w.process(WorkerEvent::Tick { ts: 1000 }, 1000);
        assert!(!w.has_reports(), "Tick rỗng không tạo report");
    }

    // ── Sensor processing ─────────────────────────────────────────────────────

    #[test]
    fn sensor_creates_report() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(22.0)), 1000);
        assert!(w.has_reports(), "Sensor data → phải có report");
        assert_eq!(w.outbox.len(), 1);
    }

    #[test]
    fn sensor_report_correct_addresses() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(22.0)), 1000);
        let report = &w.outbox[0];
        assert_eq!(report.frame.header.from, waddr(), "from = worker addr");
        assert_eq!(report.frame.header.to, chief(), "to   = chief addr");
    }

    #[test]
    fn sensor_report_has_body() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(22.0)), 1000);
        let report = &w.outbox[0];
        assert!(!report.frame.body.is_empty(), "Body chứa sensor data");
        // Body: sensor_id(1) + unit(1) + f32(4) + i64(8) = 13 bytes
        assert_eq!(report.frame.body.len(), 14); // sensor_id(1)+unit(1)+f32(4)+i64(8)
    }

    #[test]
    fn sensor_emotion_hot() {
        let reading = temp_reading(40.0); // nóng
        let emo = sensor_emotion(&reading);
        assert!(emo.arousal > 0.5, "Nóng → arousal cao: {}", emo.arousal);
        assert!(emo.valence < 0.0, "Nóng → valence âm: {}", emo.valence);
    }

    #[test]
    fn sensor_emotion_normal_temp() {
        let reading = temp_reading(22.0); // bình thường
        let emo = sensor_emotion(&reading);
        assert!(
            emo.intensity < 0.3,
            "Bình thường → intensity thấp: {}",
            emo.intensity
        );
    }

    #[test]
    fn sensor_emotion_motion() {
        let r = SensorReading {
            sensor_id: 2,
            value: 1.0,
            unit: SensorUnit::Motion,
            timestamp: 1,
        };
        let e = sensor_emotion(&r);
        assert!(e.arousal > 0.5, "Chuyển động → arousal cao: {}", e.arousal);
    }

    // ── Quantization ──────────────────────────────────────────────────────────

    #[test]
    fn quantize_temperature_range() {
        // -40°C → 0, 85°C → 255
        assert_eq!(quantize_value(-40.0, SensorUnit::Temperature), 0);
        assert_eq!(quantize_value(85.0, SensorUnit::Temperature), 255);
        // 22.5°C ≈ giữa
        let mid = quantize_value(22.5, SensorUnit::Temperature);
        assert!(mid > 100 && mid < 155, "22.5°C ≈ giữa: {}", mid);
    }

    #[test]
    fn quantize_humidity() {
        assert_eq!(quantize_value(0.0, SensorUnit::Humidity), 0);
        assert_eq!(quantize_value(100.0, SensorUnit::Humidity), 255);
        assert_eq!(quantize_value(50.0, SensorUnit::Humidity), 127);
    }

    // ── Flush ─────────────────────────────────────────────────────────────────

    #[test]
    fn flush_clears_outbox() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(20.0)), 1000);
        w.process(WorkerEvent::SensorData(temp_reading(21.0)), 2000);
        let reports = w.flush();
        assert_eq!(reports.len(), 2);
        assert!(!w.has_reports(), "Outbox rỗng sau flush");
    }

    // ── Actuator ──────────────────────────────────────────────────────────────

    #[test]
    fn actuator_command_creates_ack() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        let cmd_msg = ISLMessage::actuator(chief(), waddr(), 0x01, 0xFF);
        w.process(WorkerEvent::ISLCommand(cmd_msg), 1000);
        assert!(w.has_reports(), "ActuatorCmd → ACK report");
        let report = &w.outbox[0];
        assert_eq!(report.frame.header.msg_type, MsgType::Ack);
        assert_eq!(report.frame.header.to, chief());
    }

    // ── Wire size ─────────────────────────────────────────────────────────────

    #[test]
    fn report_wire_size_small() {
        let mut w = worker();
        w.process(WorkerEvent::SensorData(temp_reading(22.0)), 1000);
        let report = w.flush().into_iter().next().unwrap();
        let bytes = report.frame.to_bytes();
        // 12B header + 2B len + 14B body = 28B total
        assert_eq!(bytes.len(), 28, "Wire size: {}", bytes.len());
        // So sánh với JSON: {"sensor_id":1,"value":22.0,"unit":"Temperature","timestamp":1000}
        // = ~70+ bytes
        assert!(bytes.len() < 40, "Wire size nhỏ hơn JSON đáng kể");
    }

    // ── Camera worker ────────────────────────────────────────────────────────

    #[test]
    fn camera_reports_on_motion() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        w.process(
            WorkerEvent::CameraFrame {
                motion_score: 0.8,
                brightness: 100.0,
            },
            1000,
        );
        assert!(w.has_reports(), "Motion > 0.3 → report");
        assert_eq!(w.motion_streak, 1);
    }

    #[test]
    fn camera_silent_no_motion() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        w.process(
            WorkerEvent::CameraFrame {
                motion_score: 0.1,
                brightness: 50.0,
            },
            1000,
        );
        assert!(!w.has_reports(), "Motion ≤ 0.3 → silent");
        assert_eq!(w.motion_streak, 0);
    }

    #[test]
    fn camera_motion_streak_tracking() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        for i in 0..3 {
            w.process(
                WorkerEvent::CameraFrame {
                    motion_score: 0.6,
                    brightness: 80.0,
                },
                i * 100,
            );
        }
        assert_eq!(w.motion_streak, 3);
        // Break streak
        w.process(
            WorkerEvent::CameraFrame {
                motion_score: 0.1,
                brightness: 80.0,
            },
            400,
        );
        assert_eq!(w.motion_streak, 0);
    }

    #[test]
    fn camera_sustained_motion_high_urgency() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        for i in 0..7 {
            w.process(
                WorkerEvent::CameraFrame {
                    motion_score: 0.8,
                    brightness: 90.0,
                },
                i * 100,
            );
        }
        assert!(w.motion_streak > 5, "Streak > 5");
        let reports = w.flush();
        let last = &reports[reports.len() - 1];
        assert!(
            last.emotion.arousal > 0.80,
            "Sustained motion → high arousal"
        );
    }

    // ── Network worker ───────────────────────────────────────────────────────

    #[test]
    fn network_emergency_on_high_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 1000,
                anomaly_score: 0.9,
            },
            1000,
        );
        assert!(w.has_reports());
        let report = &w.outbox[0];
        assert_eq!(
            report.frame.header.msg_type,
            MsgType::Emergency,
            "High anomaly → Emergency"
        );
    }

    #[test]
    fn network_report_moderate_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 500,
                anomaly_score: 0.5,
            },
            1000,
        );
        assert!(w.has_reports());
        let report = &w.outbox[0];
        assert_eq!(
            report.frame.header.msg_type,
            MsgType::ChainPayload,
            "Moderate → report"
        );
    }

    #[test]
    fn network_silent_low_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 100,
                anomaly_score: 0.2,
            },
            1000,
        );
        assert!(!w.has_reports(), "Low anomaly → silent");
    }

    #[test]
    fn network_anomaly_accumulator() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        // EMA: acc = acc * 0.7 + score * 0.3
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 100,
                anomaly_score: 0.5,
            },
            1000,
        );
        assert!(
            (w.anomaly_accumulator - 0.15).abs() < 0.01,
            "acc = 0*0.7 + 0.5*0.3 = 0.15"
        );
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 100,
                anomaly_score: 0.5,
            },
            2000,
        );
        // 0.15*0.7 + 0.5*0.3 = 0.105 + 0.15 = 0.255
        assert!((w.anomaly_accumulator - 0.255).abs() < 0.01);
    }

    // ── Door worker (security) ───────────────────────────────────────────────

    #[test]
    fn door_locked_rejects_command() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        w.set_security_lock(true);
        let cmd = ISLMessage::actuator(chief(), waddr(), 0x01, 0xFF);
        w.process(WorkerEvent::ISLCommand(cmd), 1000);

        assert!(w.has_reports());
        let report = &w.outbox[0];
        assert_eq!(report.frame.header.msg_type, MsgType::Nack, "Locked → NACK");
    }

    #[test]
    fn door_unlocked_accepts_command() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        w.set_security_lock(false);
        let cmd = ISLMessage::actuator(chief(), waddr(), 0x01, 0xFF);
        w.process(WorkerEvent::ISLCommand(cmd), 1000);

        let report = &w.outbox[0];
        assert_eq!(report.frame.header.msg_type, MsgType::Ack, "Unlocked → ACK");
    }

    // ── Non-matching kind ignored ────────────────────────────────────────────

    #[test]
    fn sensor_ignores_camera_event() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        w.process(
            WorkerEvent::CameraFrame {
                motion_score: 0.9,
                brightness: 100.0,
            },
            1000,
        );
        assert!(!w.has_reports(), "Sensor worker ignores CameraFrame");
    }

    #[test]
    fn sensor_ignores_network_event() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        w.process(
            WorkerEvent::NetworkPacket {
                bytes_in: 1000,
                anomaly_score: 0.9,
            },
            1000,
        );
        assert!(!w.has_reports(), "Sensor worker ignores NetworkPacket");
    }

    // ── DriverProbe ────────────────────────────────────────────────────────────

    #[test]
    fn probe_sensor_ready() {
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let probe = w.probe_hardware(1000);
        assert_eq!(probe.status, HardwareStatus::Ready);
        assert_eq!(probe.kind, WorkerKind::Sensor);
        assert!(
            !probe.capabilities.is_empty(),
            "Sensor phải khai báo capabilities"
        );
        assert!(probe.device_id.contains("sensor"), "Device ID chứa loại");
    }

    #[test]
    fn probe_actuator_ready() {
        let w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        let probe = w.probe_hardware(1000);
        assert_eq!(probe.status, HardwareStatus::Ready);
        assert_eq!(probe.capabilities.len(), 2, "Actuator: on/off + dim");
    }

    #[test]
    fn probe_error_state() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        w.state = WorkerState::Error;
        let probe = w.probe_hardware(1000);
        assert!(matches!(probe.status, HardwareStatus::Error(_)));
    }

    #[test]
    fn probe_report_creates_frame() {
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let report = w.probe_report(1000);
        assert_eq!(report.frame.header.from, waddr());
        assert_eq!(report.frame.header.to, chief());
        assert!(!report.frame.body.is_empty(), "Probe report phải có body");
        // Body: kind(1) + status(1) + cap_len(1) + caps(N)
        assert!(report.frame.body.len() >= 3);
    }

    #[test]
    fn probe_network_capabilities() {
        let w = Worker::new(waddr(), chief(), WorkerKind::Network);
        let probe = w.probe_hardware(1000);
        assert_eq!(probe.capabilities.len(), 2, "Network: monitor + firewall");
    }

    // ── HAL probe ─────────────────────────────────────────────────────────────

    #[test]
    fn hal_probe_sensor_on_esp32() {
        let platform = hal::MockPlatform::esp32();
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let probe = w.probe_with_hal(&platform, 1000);
        assert_eq!(probe.status, HardwareStatus::Ready, "ESP32 có sensor");
        assert!(!probe.capabilities.is_empty(), "ESP32 sensor caps từ HAL");
        assert!(probe.device_id.contains("sensor"));
    }

    #[test]
    fn hal_probe_camera_on_smartphone() {
        let platform = hal::MockPlatform::smartphone();
        let w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        let probe = w.probe_with_hal(&platform, 1000);
        assert_eq!(probe.status, HardwareStatus::Ready, "Phone có camera");
        assert!(!probe.capabilities.is_empty(), "Camera cap từ HAL");
    }

    #[test]
    fn hal_probe_camera_on_esp32_unreachable() {
        let platform = hal::MockPlatform::esp32();
        let w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        let probe = w.probe_with_hal(&platform, 1000);
        // ESP32 không có camera device → Unreachable
        assert_eq!(probe.status, HardwareStatus::Unreachable);
    }

    #[test]
    fn hal_probe_network_on_pc() {
        let platform = hal::MockPlatform::pc();
        let w = Worker::new(waddr(), chief(), WorkerKind::Network);
        let probe = w.probe_with_hal(&platform, 1000);
        assert_eq!(probe.status, HardwareStatus::Ready);
        assert!(!probe.capabilities.is_empty());
    }

    #[test]
    fn hal_probe_actuator_on_rpi() {
        let platform = hal::MockPlatform::raspberry_pi();
        let w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        let probe = w.probe_with_hal(&platform, 1000);
        // RPi không có actuator device trong mock → Unreachable
        // Nhưng có GPIO capability
        assert!(!probe.capabilities.is_empty() || probe.status == HardwareStatus::Unreachable);
    }

    #[test]
    fn hal_probe_report_contains_arch() {
        let platform = hal::MockPlatform::esp32();
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let report = w.probe_report_hal(&platform, 1000);
        // Body cuối = arch byte
        let body = &report.frame.body;
        let arch_byte = body[body.len() - 1];
        assert_eq!(arch_byte, hal::Architecture::Xtensa as u8);
    }

    #[test]
    fn hal_probe_riscv_sensor() {
        let platform = hal::MockPlatform::riscv_embedded();
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let probe = w.probe_with_hal(&platform, 1000);
        assert_eq!(probe.status, HardwareStatus::Ready, "RISC-V có sensor");
    }

    // ── SensorNormalizer ──────────────────────────────────────────────────────

    // ── ISL inbox ──────────────────────────────────────────────────────────

    #[test]
    fn worker_inbox_starts_empty() {
        let w = worker();
        assert_eq!(w.inbox_len(), 0);
    }

    #[test]
    fn worker_receive_isl_queues() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        let cmd = ISLMessage::actuator(chief(), waddr(), 0x01, 0xFF);
        let frame = ISLFrame::bare(cmd);
        w.receive_isl(frame);
        assert_eq!(w.inbox_len(), 1);
    }

    #[test]
    fn worker_poll_inbox_processes_commands() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        let cmd = ISLMessage::actuator(chief(), waddr(), 0x01, 0xFF);
        let frame = ISLFrame::bare(cmd);
        w.receive_isl(frame);
        let processed = w.poll_inbox(1000);
        assert_eq!(processed, 1);
        assert_eq!(w.inbox_len(), 0, "Inbox drained after poll");
        assert!(w.has_reports(), "Command → ACK report");
    }

    #[test]
    fn worker_poll_inbox_empty_noop() {
        let mut w = worker();
        let processed = w.poll_inbox(1000);
        assert_eq!(processed, 0);
    }

    #[test]
    fn worker_poll_inbox_multiple_commands() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Actuator);
        for i in 0..3 {
            let cmd = ISLMessage::actuator(chief(), waddr(), 0x01, i);
            w.receive_isl(ISLFrame::bare(cmd));
        }
        assert_eq!(w.inbox_len(), 3);
        let processed = w.poll_inbox(1000);
        assert_eq!(processed, 3);
        assert_eq!(w.outbox.len(), 3, "3 commands → 3 ACKs");
    }

    #[test]
    fn normalizer_first_reading_reports() {
        let mut norm = SensorNormalizer::new();
        let r = SensorReading {
            sensor_id: 1,
            value: 22.0,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        let (smoothed, should_report) = norm.process(&r);
        assert!(should_report, "First reading always reports");
        assert!((smoothed - 22.0).abs() < 0.01);
        assert_eq!(norm.sample_count(1, SensorUnit::Temperature), 1);
    }

    #[test]
    fn normalizer_ema_smoothing() {
        let mut norm = SensorNormalizer::new();
        // α=0.3 for temperature
        let r1 = SensorReading {
            sensor_id: 1,
            value: 20.0,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        norm.process(&r1); // smoothed = 20.0

        let r2 = SensorReading {
            sensor_id: 1,
            value: 30.0,
            unit: SensorUnit::Temperature,
            timestamp: 2000,
        };
        let (smoothed, _) = norm.process(&r2);
        // EMA: 0.3 * 30 + 0.7 * 20 = 9 + 14 = 23.0
        assert!(
            (smoothed - 23.0).abs() < 0.01,
            "EMA: expected 23.0, got {}",
            smoothed
        );
    }

    #[test]
    fn normalizer_dead_band_suppresses_small_changes() {
        let mut norm = SensorNormalizer::new();
        // Temperature dead_band = 0.5°C
        let r1 = SensorReading {
            sensor_id: 1,
            value: 22.0,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        norm.process(&r1); // first → report

        // Small change (0.1°C) → after EMA, delta < 0.5
        let r2 = SensorReading {
            sensor_id: 1,
            value: 22.1,
            unit: SensorUnit::Temperature,
            timestamp: 2000,
        };
        let (_, should_report) = norm.process(&r2);
        assert!(!should_report, "Small change suppressed by dead band");
    }

    #[test]
    fn normalizer_dead_band_passes_large_changes() {
        let mut norm = SensorNormalizer::new();
        let r1 = SensorReading {
            sensor_id: 1,
            value: 22.0,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        norm.process(&r1);

        // Large change (10°C) → delta >> 0.5
        let r2 = SensorReading {
            sensor_id: 1,
            value: 32.0,
            unit: SensorUnit::Temperature,
            timestamp: 2000,
        };
        let (_, should_report) = norm.process(&r2);
        assert!(should_report, "Large change passes dead band");
    }

    #[test]
    fn normalizer_normalize_range() {
        let mut norm = SensorNormalizer::new();
        // Process one reading to create state
        let r = SensorReading {
            sensor_id: 1,
            value: 22.5,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        norm.process(&r);

        // Temperature: min=-40, max=85 → 22.5 maps to (22.5+40)/125 = 0.5
        let n = norm.normalize(1, SensorUnit::Temperature, 22.5);
        assert!((n - 0.5).abs() < 0.01, "22.5°C ≈ 0.5 normalized: {}", n);

        // Clamp below min
        let n_low = norm.normalize(1, SensorUnit::Temperature, -50.0);
        assert!((n_low - 0.0).abs() < 0.01, "Below min → 0.0");

        // Clamp above max
        let n_high = norm.normalize(1, SensorUnit::Temperature, 100.0);
        assert!((n_high - 1.0).abs() < 0.01, "Above max → 1.0");
    }

    #[test]
    fn normalizer_multiple_sensors() {
        let mut norm = SensorNormalizer::new();
        let r_temp = SensorReading {
            sensor_id: 1,
            value: 22.0,
            unit: SensorUnit::Temperature,
            timestamp: 1000,
        };
        let r_hum = SensorReading {
            sensor_id: 2,
            value: 60.0,
            unit: SensorUnit::Humidity,
            timestamp: 1000,
        };
        norm.process(&r_temp);
        norm.process(&r_hum);

        assert_eq!(norm.sample_count(1, SensorUnit::Temperature), 1);
        assert_eq!(norm.sample_count(2, SensorUnit::Humidity), 1);
        assert!(norm.smoothed_value(1, SensorUnit::Temperature).is_some());
        assert!(norm.smoothed_value(2, SensorUnit::Humidity).is_some());
    }

    #[test]
    fn normalizer_custom_calibration() {
        let mut norm = SensorNormalizer::new();
        norm.set_calibration(
            5,
            SensorUnit::Custom,
            SensorCalibration {
                min: 0.0,
                max: 100.0,
                ema_alpha: 0.5,
                dead_band: 5.0,
            },
        );
        let r = SensorReading {
            sensor_id: 5,
            value: 50.0,
            unit: SensorUnit::Custom,
            timestamp: 1000,
        };
        let (smoothed, _) = norm.process(&r);
        assert!((smoothed - 50.0).abs() < 0.01);
        let n = norm.normalize(5, SensorUnit::Custom, 50.0);
        assert!((n - 0.5).abs() < 0.01);
    }
}
