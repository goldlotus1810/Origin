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
use isl::message::{ISLMessage, ISLFrame, MsgType};
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// WorkerKind
// ─────────────────────────────────────────────────────────────────────────────

/// Loại Worker.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkerKind {
    Sensor   = 0x01, // đọc sensor
    Actuator = 0x02, // điều khiển thiết bị
    Camera   = 0x03, // xử lý hình ảnh
    Network  = 0x04, // mạng/bảo mật
    Generic  = 0xFF,
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
    pub value:     f32,
    pub unit:      SensorUnit,
    pub timestamp: i64,
}

/// Đơn vị đo.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorUnit {
    Temperature = 0x01, // °C
    Humidity    = 0x02, // %
    Light       = 0x03, // lux
    Motion      = 0x04, // 0/1
    Sound       = 0x05, // dB
    Distance    = 0x06, // cm
    Pressure    = 0x07, // hPa
    Custom      = 0xFF,
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
    pub frame:   ISLFrame,
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
    pub addr:    ISLAddress,
    pub chief:   ISLAddress,
    pub kind:    WorkerKind,
    pub state:   WorkerState,
    /// Outbox — chờ gửi lên Chief
    pub outbox:  Vec<WorkerReport>,
    /// Event count
    pub events:  u32,
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
            addr, chief, kind,
            state:  WorkerState::Sleeping,
            outbox: Vec::new(),
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
            WorkerEvent::CameraFrame { motion_score, brightness } => {
                self.process_camera(motion_score, brightness, ts);
            }
            WorkerEvent::NetworkPacket { bytes_in, anomaly_score } => {
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
    pub fn has_reports(&self) -> bool { !self.outbox.is_empty() }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn process_sensor(&mut self, reading: SensorReading, _ts: i64) {
        // Encode reading → payload bytes (không raw float — encode thành 3B)
        let payload = encode_sensor_payload(&reading);
        let emotion = sensor_emotion(&reading);

        let msg   = ISLMessage::with_payload(
            self.addr, self.chief, MsgType::ChainPayload, payload
        );
        // Body = sensor metadata: [sensor_id, unit, value_hi, value_lo]
        let body  = encode_sensor_body(&reading);
        let frame = ISLFrame::with_body(msg, body);

        self.outbox.push(WorkerReport { frame, emotion });
    }

    fn process_command(&mut self, cmd: ISLMessage, _ts: i64) {
        if cmd.msg_type != MsgType::ActuatorCmd { return; }

        // Door worker: security check trước khi thực thi
        if self.kind == WorkerKind::Actuator && self.security_locked {
            // Reject — gửi NACK thay vì ACK
            let nack = ISLMessage::nack(self.addr, cmd.from, MsgType::ActuatorCmd);
            let frame = ISLFrame::bare(nack);
            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag { valence: -0.50, arousal: 0.60, dominance: 0.80, intensity: 0.50 },
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
        if self.kind != WorkerKind::Camera { return; }

        // Track motion streak
        if motion_score > 0.3 {
            self.motion_streak += 1;
        } else {
            self.motion_streak = 0;
        }

        // Chỉ gửi report khi có motion đáng kể (threshold: score > 0.3)
        // Silent by default — không gửi frame rỗng
        if motion_score <= 0.3 { return; }

        let payload = [
            self.addr.index,        // camera id
            0x04,                      // unit = Motion
            (motion_score * 255.0) as u8, // quantized motion
        ];
        let msg = ISLMessage::with_payload(self.addr, self.chief, MsgType::ChainPayload, payload);

        // Body: motion_score(f32) + brightness(f32) + streak(u32) + timestamp(i64)
        let mut body = Vec::with_capacity(20);
        body.push(self.addr.index); // sensor_id
        body.push(0x04);              // unit = Motion
        body.extend_from_slice(&motion_score.to_be_bytes());
        body.extend_from_slice(&brightness.to_be_bytes());
        body.extend_from_slice(&self.motion_streak.to_be_bytes());
        body.extend_from_slice(&ts.to_be_bytes());
        let frame = ISLFrame::with_body(msg, body);

        let emotion = if self.motion_streak > 5 {
            // Sustained motion → higher urgency
            EmotionTag { valence: -0.20, arousal: 0.85, dominance: 0.40, intensity: 0.75 }
        } else {
            EmotionTag { valence: 0.0, arousal: 0.60, dominance: 0.50, intensity: 0.45 }
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
        if self.kind != WorkerKind::Network { return; }

        // Accumulate anomaly (exponential moving average)
        self.anomaly_accumulator = self.anomaly_accumulator * 0.7 + anomaly_score * 0.3;

        // Emergency: high anomaly → immediate alert
        if anomaly_score > 0.7 {
            let msg = ISLMessage::emergency(self.addr, (anomaly_score * 255.0) as u8);
            let frame = ISLFrame::bare(msg);
            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag { valence: -0.70, arousal: 0.95, dominance: 0.20, intensity: 0.90 },
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
            let msg = ISLMessage::with_payload(self.addr, self.chief, MsgType::ChainPayload, payload);
            let mut body = Vec::with_capacity(12);
            body.push(self.addr.index);
            body.push(0xFF);
            body.extend_from_slice(&anomaly_score.to_be_bytes());
            body.extend_from_slice(&bytes_in.to_be_bytes());
            let frame = ISLFrame::with_body(msg, body);

            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag { valence: -0.30, arousal: 0.65, dominance: 0.35, intensity: 0.55 },
            });
        }
        // ≤ 0.4 → silent (no report)
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
        SensorUnit::Humidity    => (0.0, 100.0),
        SensorUnit::Light       => (0.0, 10000.0),
        SensorUnit::Motion      => (0.0, 1.0),
        SensorUnit::Sound       => (0.0, 120.0),
        SensorUnit::Distance    => (0.0, 500.0),
        SensorUnit::Pressure    => (800.0, 1200.0),
        SensorUnit::Custom      => (0.0, 255.0),
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
                EmotionTag { valence: -0.30, arousal: 0.70, dominance: 0.30, intensity: 0.55 }
            } else if temp < 10.0 {
                EmotionTag { valence: -0.20, arousal: 0.60, dominance: 0.30, intensity: 0.45 }
            } else {
                EmotionTag { valence: 0.10, arousal: 0.30, dominance: 0.60, intensity: 0.15 }
            }
        }
        SensorUnit::Motion => {
            if r.value > 0.5 {
                EmotionTag { valence: 0.0, arousal: 0.75, dominance: 0.50, intensity: 0.60 }
            } else {
                EmotionTag::NEUTRAL
            }
        }
        SensorUnit::Sound => {
            let db = r.value;
            if db > 80.0 {
                EmotionTag { valence: -0.40, arousal: 0.85, dominance: 0.25, intensity: 0.70 }
            } else if db > 60.0 {
                EmotionTag { valence: -0.10, arousal: 0.50, dominance: 0.50, intensity: 0.30 }
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
    pub kind:     WorkerKind,
    /// Trạng thái
    pub status:   HardwareStatus,
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
impl Worker {
    /// Probe phần cứng — Worker tự kiểm tra device mình quản lý.
    ///
    /// Trả DriverProbeResult cho Chief tổng hợp.
    /// Trong môi trường thật: gọi HAL interface.
    /// Hiện tại: report từ internal state (software probe).
    pub fn probe_hardware(&self, ts: i64) -> DriverProbeResult {
        let device_id = match self.kind {
            WorkerKind::Sensor   => alloc::format!("sensor@{}", self.addr),
            WorkerKind::Actuator => alloc::format!("actuator@{}", self.addr),
            WorkerKind::Camera   => alloc::format!("camera@{}", self.addr),
            WorkerKind::Network  => alloc::format!("network@{}", self.addr),
            WorkerKind::Generic  => alloc::format!("generic@{}", self.addr),
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
            WorkerKind::Camera   => alloc::vec![0x01],       // video stream
            WorkerKind::Network  => alloc::vec![0x01, 0x02], // monitor, firewall
            WorkerKind::Generic  => alloc::vec![],
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
            HardwareStatus::Ready       => 0x01,
            HardwareStatus::Unreachable => 0x02,
            HardwareStatus::Error(c)    => 0x80 | c,
            HardwareStatus::Unknown     => 0x00,
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
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn chief() -> ISLAddress { ISLAddress::new(0, 0, 0, 0) }
    fn waddr() -> ISLAddress { ISLAddress::new(1, 5, 0, 1) }

    fn worker() -> Worker {
        Worker::new(waddr(), chief(), WorkerKind::Sensor)
    }

    fn temp_reading(val: f32) -> SensorReading {
        SensorReading { sensor_id: 1, value: val, unit: SensorUnit::Temperature, timestamp: 1000 }
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
        assert_eq!(w.state, WorkerState::Sleeping, "Worker sleep ngay sau khi xử lý");
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
        assert_eq!(report.frame.header.to,   chief(), "to   = chief addr");
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
        let emo     = sensor_emotion(&reading);
        assert!(emo.arousal > 0.5, "Nóng → arousal cao: {}", emo.arousal);
        assert!(emo.valence < 0.0, "Nóng → valence âm: {}", emo.valence);
    }

    #[test]
    fn sensor_emotion_normal_temp() {
        let reading = temp_reading(22.0); // bình thường
        let emo     = sensor_emotion(&reading);
        assert!(emo.intensity < 0.3, "Bình thường → intensity thấp: {}", emo.intensity);
    }

    #[test]
    fn sensor_emotion_motion() {
        let r = SensorReading { sensor_id: 2, value: 1.0, unit: SensorUnit::Motion, timestamp: 1 };
        let e = sensor_emotion(&r);
        assert!(e.arousal > 0.5, "Chuyển động → arousal cao: {}", e.arousal);
    }

    // ── Quantization ──────────────────────────────────────────────────────────

    #[test]
    fn quantize_temperature_range() {
        // -40°C → 0, 85°C → 255
        assert_eq!(quantize_value(-40.0, SensorUnit::Temperature), 0);
        assert_eq!(quantize_value(85.0,  SensorUnit::Temperature), 255);
        // 22.5°C ≈ giữa
        let mid = quantize_value(22.5, SensorUnit::Temperature);
        assert!(mid > 100 && mid < 155, "22.5°C ≈ giữa: {}", mid);
    }

    #[test]
    fn quantize_humidity() {
        assert_eq!(quantize_value(0.0,   SensorUnit::Humidity), 0);
        assert_eq!(quantize_value(100.0, SensorUnit::Humidity), 255);
        assert_eq!(quantize_value(50.0,  SensorUnit::Humidity), 127);
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
        let mut w   = Worker::new(waddr(), chief(), WorkerKind::Actuator);
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
        let bytes  = report.frame.to_bytes();
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
        w.process(WorkerEvent::CameraFrame { motion_score: 0.8, brightness: 100.0 }, 1000);
        assert!(w.has_reports(), "Motion > 0.3 → report");
        assert_eq!(w.motion_streak, 1);
    }

    #[test]
    fn camera_silent_no_motion() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        w.process(WorkerEvent::CameraFrame { motion_score: 0.1, brightness: 50.0 }, 1000);
        assert!(!w.has_reports(), "Motion ≤ 0.3 → silent");
        assert_eq!(w.motion_streak, 0);
    }

    #[test]
    fn camera_motion_streak_tracking() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        for i in 0..3 {
            w.process(WorkerEvent::CameraFrame { motion_score: 0.6, brightness: 80.0 }, i * 100);
        }
        assert_eq!(w.motion_streak, 3);
        // Break streak
        w.process(WorkerEvent::CameraFrame { motion_score: 0.1, brightness: 80.0 }, 400);
        assert_eq!(w.motion_streak, 0);
    }

    #[test]
    fn camera_sustained_motion_high_urgency() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Camera);
        for i in 0..7 {
            w.process(WorkerEvent::CameraFrame { motion_score: 0.8, brightness: 90.0 }, i * 100);
        }
        assert!(w.motion_streak > 5, "Streak > 5");
        let reports = w.flush();
        let last = &reports[reports.len() - 1];
        assert!(last.emotion.arousal > 0.80, "Sustained motion → high arousal");
    }

    // ── Network worker ───────────────────────────────────────────────────────

    #[test]
    fn network_emergency_on_high_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(WorkerEvent::NetworkPacket { bytes_in: 1000, anomaly_score: 0.9 }, 1000);
        assert!(w.has_reports());
        let report = &w.outbox[0];
        assert_eq!(report.frame.header.msg_type, MsgType::Emergency, "High anomaly → Emergency");
    }

    #[test]
    fn network_report_moderate_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(WorkerEvent::NetworkPacket { bytes_in: 500, anomaly_score: 0.5 }, 1000);
        assert!(w.has_reports());
        let report = &w.outbox[0];
        assert_eq!(report.frame.header.msg_type, MsgType::ChainPayload, "Moderate → report");
    }

    #[test]
    fn network_silent_low_anomaly() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        w.process(WorkerEvent::NetworkPacket { bytes_in: 100, anomaly_score: 0.2 }, 1000);
        assert!(!w.has_reports(), "Low anomaly → silent");
    }

    #[test]
    fn network_anomaly_accumulator() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Network);
        // EMA: acc = acc * 0.7 + score * 0.3
        w.process(WorkerEvent::NetworkPacket { bytes_in: 100, anomaly_score: 0.5 }, 1000);
        assert!((w.anomaly_accumulator - 0.15).abs() < 0.01, "acc = 0*0.7 + 0.5*0.3 = 0.15");
        w.process(WorkerEvent::NetworkPacket { bytes_in: 100, anomaly_score: 0.5 }, 2000);
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
        w.process(WorkerEvent::CameraFrame { motion_score: 0.9, brightness: 100.0 }, 1000);
        assert!(!w.has_reports(), "Sensor worker ignores CameraFrame");
    }

    #[test]
    fn sensor_ignores_network_event() {
        let mut w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        w.process(WorkerEvent::NetworkPacket { bytes_in: 1000, anomaly_score: 0.9 }, 1000);
        assert!(!w.has_reports(), "Sensor worker ignores NetworkPacket");
    }

    // ── DriverProbe ────────────────────────────────────────────────────────────

    #[test]
    fn probe_sensor_ready() {
        let w = Worker::new(waddr(), chief(), WorkerKind::Sensor);
        let probe = w.probe_hardware(1000);
        assert_eq!(probe.status, HardwareStatus::Ready);
        assert_eq!(probe.kind, WorkerKind::Sensor);
        assert!(!probe.capabilities.is_empty(), "Sensor phải khai báo capabilities");
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
}
