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
}

impl Worker {
    /// Tạo Worker mới.
    pub fn new(addr: ISLAddress, chief: ISLAddress, kind: WorkerKind) -> Self {
        Self {
            addr, chief, kind,
            state:  WorkerState::Sleeping,
            outbox: Vec::new(),
            events: 0,
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
            WorkerEvent::Tick { ts } => {
                // Heartbeat — chỉ gửi nếu có gì trong outbox
                // Silent = không gửi tick rỗng
                let _ = ts;
            }
            WorkerEvent::Wake => {
                // Wake event — không làm gì thêm, chờ sự kiện tiếp
            }
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
        // Nhận ActuatorCmd → thực thi → gửi ACK
        if cmd.msg_type == MsgType::ActuatorCmd {
            let ack = ISLMessage::ack(self.addr, cmd.from, MsgType::ActuatorCmd);
            let frame = ISLFrame::bare(ack);
            self.outbox.push(WorkerReport {
                frame,
                emotion: EmotionTag::NEUTRAL,
            });
        }
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
}
