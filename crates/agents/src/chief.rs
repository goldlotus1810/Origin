//! # chief — Chief Agent
//!
//! Chief = tủy sống — xử lý và tổng hợp từ Workers.
//!
//! Phân cấp:
//!   AAM → Chief → Worker  (chain of command)
//!   ✅ AAM ↔ Chief · ✅ Chief ↔ Chief · ✅ Chief ↔ Worker
//!   ❌ AAM ↔ Worker · ❌ Worker ↔ Worker
//!
//! Vòng đời: chờ ISL → wake → xử lý → tổng hợp → báo LeoAI → sleep
//!
//! Chief types (từ spec):
//!   HomeChief    — quản lý Worker thiết bị nhà
//!   VisionChief  — quản lý Worker camera/sensor
//!   NetworkChief — quản lý Worker network/security

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use isl::address::ISLAddress;
use isl::message::{ISLMessage, ISLFrame, MsgType};
use isl::queue::ISLQueue;
use silk::edge::EmotionTag;
use crate::worker::WorkerKind;

// ─────────────────────────────────────────────────────────────────────────────
// ChiefKind
// ─────────────────────────────────────────────────────────────────────────────

/// Loại Chief — mỗi loại quản lý domain riêng.
///
/// Phân cấp từ spec:
///   HomeChief    — quản lý Worker thiết bị nhà (light, door, sensor)
///   VisionChief  — quản lý Worker camera/sensor hình ảnh
///   NetworkChief — quản lý Worker network/security
///   General      — fallback cho Worker không thuộc domain nào
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiefKind {
    Home,    // quản lý đèn, nhiệt, cửa...
    Vision,  // camera, sensor hình ảnh
    Network, // network, bảo mật
    General, // generic
}

impl ChiefKind {
    /// Kiểm tra WorkerKind có thuộc domain của Chief này không.
    ///
    /// Domain routing:
    ///   Home    → Sensor, Actuator
    ///   Vision  → Camera
    ///   Network → Network
    ///   General → tất cả
    pub fn accepts(&self, worker: WorkerKind) -> bool {
        match self {
            ChiefKind::Home    => matches!(worker, WorkerKind::Sensor | WorkerKind::Actuator),
            ChiefKind::Vision  => matches!(worker, WorkerKind::Camera),
            ChiefKind::Network => matches!(worker, WorkerKind::Network),
            ChiefKind::General => true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ChiefState
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiefState {
    Sleeping,
    Processing,
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkerInfo — thông tin Worker đã đăng ký
// ─────────────────────────────────────────────────────────────────────────────

/// Thông tin một Worker đang quản lý.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub addr:       ISLAddress,
    pub kind_byte:  u8,    // WorkerKind as u8
    pub last_seen:  i64,   // timestamp
    pub alive:      bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// IngestedReport — report đã xử lý từ Worker
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả Chief xử lý report từ Worker — chuẩn bị gửi LeoAI.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct IngestedReport {
    pub from_worker: ISLAddress,
    pub emotion:     EmotionTag,
    pub payload:     Vec<u8>,   // body đã decode
    pub timestamp:   i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Chief
// ─────────────────────────────────────────────────────────────────────────────

/// Chief Agent — quản lý Workers, tổng hợp và chuyển lên LeoAI.
///
/// Mỗi ChiefKind có domain-specific behavior:
///   Home    — automation rules (temp threshold → command actuator)
///   Vision  — motion event aggregation
///   Network — security alert level, immediate escalation
///   General — pass-through, no domain logic
#[allow(missing_docs)]
pub struct Chief {
    pub addr:    ISLAddress,
    pub aam:     ISLAddress,  // AAM address
    pub leo:     ISLAddress,  // LeoAI address
    pub kind:    ChiefKind,
    pub state:   ChiefState,
    /// Workers đã đăng ký
    pub workers: BTreeMap<u32, WorkerInfo>, // key = addr.to_u32()
    /// Inbox từ Workers
    inbox:       ISLQueue,
    /// Outbox → LeoAI
    pub outbox:  Vec<IngestedReport>,
    /// Outbox → Workers (commands)
    cmd_queue:   Vec<ISLFrame>,
    pub events:  u32,
    // ── Domain-specific state ────────────────────────────────────────────────
    /// NetworkChief: security alert level (0=normal, 1=elevated, 2=critical).
    pub security_level: u8,
    /// VisionChief: motion event count trong window hiện tại.
    pub motion_events: u32,
    /// HomeChief: last temperature reading (for automation).
    pub last_temp: Option<f32>,
}

impl Chief {
    pub fn new(addr: ISLAddress, aam: ISLAddress, leo: ISLAddress, kind: ChiefKind) -> Self {
        Self {
            addr, aam, leo, kind,
            state:    ChiefState::Sleeping,
            workers:  BTreeMap::new(),
            inbox:    ISLQueue::new(),
            outbox:   Vec::new(),
            cmd_queue: Vec::new(),
            events:   0,
            security_level: 0,
            motion_events:  0,
            last_temp:      None,
        }
    }

    /// Đăng ký Worker mới.
    ///
    /// Kiểm tra domain routing: WorkerKind phải thuộc ChiefKind.
    /// Trả về `false` nếu Worker không thuộc domain này.
    pub fn register_worker(&mut self, addr: ISLAddress, kind_byte: u8, ts: i64) -> bool {
        let worker_kind = worker_kind_from_byte(kind_byte);
        if !self.kind.accepts(worker_kind) {
            return false;
        }
        self.workers.insert(addr.to_u32(), WorkerInfo {
            addr, kind_byte, last_seen: ts, alive: true,
        });
        true
    }

    /// Nhận ISLFrame từ Worker vào inbox.
    pub fn receive(&mut self, frame: ISLFrame) -> bool {
        self.inbox.push(frame.header)
        // body sẽ xử lý trong process_inbox
        // Đơn giản hóa: push header, body xử lý riêng
    }

    /// Nhận ISLFrame đầy đủ (header + body).
    pub fn receive_frame(&mut self, frame: ISLFrame, ts: i64) {
        self.state  = ChiefState::Processing;
        self.events += 1;

        // Cập nhật last_seen cho worker
        let key = frame.header.from.to_u32();
        if let Some(w) = self.workers.get_mut(&key) {
            w.last_seen = ts;
            w.alive     = true;
        }

        // Xử lý theo msg_type
        match frame.header.msg_type {
            MsgType::ChainPayload => {
                let emotion = decode_emotion_from_body(&frame.body);
                self.outbox.push(IngestedReport {
                    from_worker: frame.header.from,
                    emotion,
                    payload:     frame.body.clone(),
                    timestamp:   ts,
                });
                // Domain-specific post-processing
                self.domain_process(&frame.body, ts);
            }
            MsgType::Ack => {
                // Worker ack command → OK
            }
            MsgType::Emergency => {
                // Emergency → forward lên AAM
                let alert = ISLFrame::bare(
                    ISLMessage::emergency(self.addr, 0xEE)
                );
                self.cmd_queue.push(alert);
                // NetworkChief: escalate security level ngay
                if self.kind == ChiefKind::Network {
                    self.security_level = 2; // critical
                }
            }
            _ => {}
        }

        self.state = ChiefState::Sleeping;
    }

    /// Gửi command từ AAM xuống Worker.
    ///
    /// AAM → Chief → Worker (không được đi tắt AAM → Worker)
    pub fn forward_command(&mut self, target: ISLAddress, cmd: u8, value: u8) {
        let msg   = ISLMessage::actuator(self.addr, target, cmd, value);
        let frame = ISLFrame::bare(msg);
        self.cmd_queue.push(frame);
    }

    /// Lấy commands chờ gửi xuống Workers.
    pub fn drain_commands(&mut self) -> Vec<ISLFrame> {
        let mut out = Vec::new();
        core::mem::swap(&mut self.cmd_queue, &mut out);
        out
    }

    /// Lấy reports chờ gửi lên LeoAI.
    pub fn drain_reports(&mut self) -> Vec<IngestedReport> {
        let mut out = Vec::new();
        core::mem::swap(&mut self.outbox, &mut out);
        out
    }

    /// Số Workers đang alive.
    pub fn alive_count(&self) -> usize {
        self.workers.values().filter(|w| w.alive).count()
    }

    /// Heartbeat check — mark workers không báo cáo trong timeout là dead.
    pub fn heartbeat_check(&mut self, now: i64, timeout_ns: i64) {
        for w in self.workers.values_mut() {
            if now - w.last_seen > timeout_ns {
                w.alive = false;
            }
        }
    }

    // ── Domain-specific processing ───────────────────────────────────────────

    /// Xử lý domain-specific sau khi ingest ChainPayload.
    fn domain_process(&mut self, body: &[u8], _ts: i64) {
        match self.kind {
            ChiefKind::Home => self.home_process(body),
            ChiefKind::Vision => self.vision_process(body),
            ChiefKind::Network => self.network_process(body),
            ChiefKind::General => {} // pass-through
        }
    }

    /// HomeChief: track temperature, trigger automation.
    ///
    /// Automation rules:
    ///   temp > 35°C → command tất cả Actuator workers: ON cooling
    ///   temp < 10°C → command tất cả Actuator workers: ON heating
    fn home_process(&mut self, body: &[u8]) {
        if body.len() < 6 { return; }
        let unit_byte = body[1];
        if unit_byte != 0x01 { return; } // only Temperature
        let val = f32::from_be_bytes([body[2], body[3], body[4], body[5]]);
        self.last_temp = Some(val);

        // Automation: threshold → command actuators
        if val > 35.0 {
            self.command_all_actuators(0x01, 0xFF); // ON cooling
        } else if val < 10.0 {
            self.command_all_actuators(0x02, 0xFF); // ON heating
        }
    }

    /// VisionChief: count motion events.
    fn vision_process(&mut self, body: &[u8]) {
        if body.len() < 6 { return; }
        let unit_byte = body[1];
        // Motion sensor (0x04) hoặc camera frame with motion
        if unit_byte == 0x04 {
            let val = f32::from_be_bytes([body[2], body[3], body[4], body[5]]);
            if val > 0.5 {
                self.motion_events += 1;
            }
        }
    }

    /// NetworkChief: assess security from network events.
    fn network_process(&mut self, body: &[u8]) {
        if body.len() < 6 { return; }
        // Network anomaly detection heuristic:
        // High arousal + negative valence from network worker → elevate security
        let emotion = decode_emotion_from_body(body);
        if emotion.arousal > 0.7 && emotion.valence < -0.3 {
            if self.security_level < 2 {
                self.security_level += 1;
            }
        }
    }

    /// Command tất cả Actuator workers đã đăng ký.
    fn command_all_actuators(&mut self, cmd: u8, value: u8) {
        let actuator_addrs: Vec<ISLAddress> = self.workers.values()
            .filter(|w| w.alive && w.kind_byte == WorkerKind::Actuator as u8)
            .map(|w| w.addr)
            .collect();
        for addr in actuator_addrs {
            self.forward_command(addr, cmd, value);
        }
    }

    /// Reset security level (dùng sau khi AAM xác nhận an toàn).
    pub fn reset_security(&mut self) {
        self.security_level = 0;
    }

    /// Reset motion counter (dùng sau mỗi aggregation window).
    pub fn reset_motion(&mut self) {
        self.motion_events = 0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Map byte → WorkerKind.
fn worker_kind_from_byte(b: u8) -> WorkerKind {
    match b {
        0x01 => WorkerKind::Sensor,
        0x02 => WorkerKind::Actuator,
        0x03 => WorkerKind::Camera,
        0x04 => WorkerKind::Network,
        _    => WorkerKind::Generic,
    }
}

/// Decode EmotionTag từ body bytes (từ Worker encode).
/// Body format: [sensor_id(1), unit(1), f32(4), i64(8)] = 14 bytes
fn decode_emotion_from_body(body: &[u8]) -> EmotionTag {
    if body.len() < 6 { return EmotionTag::NEUTRAL; }
    let unit_byte = body[1];
    let val_bytes = [body[2], body[3], body[4], body[5]];
    let val = f32::from_be_bytes(val_bytes);

    // Map unit + value → EmotionTag (mirror của worker::sensor_emotion)
    match unit_byte {
        0x01 => { // Temperature
            if val > 35.0 {
                EmotionTag { valence: -0.30, arousal: 0.70, dominance: 0.30, intensity: 0.55 }
            } else if val < 10.0 {
                EmotionTag { valence: -0.20, arousal: 0.60, dominance: 0.30, intensity: 0.45 }
            } else {
                EmotionTag { valence: 0.10, arousal: 0.30, dominance: 0.60, intensity: 0.15 }
            }
        }
        0x04 => { // Motion
            if val > 0.5 {
                EmotionTag { valence: 0.0, arousal: 0.75, dominance: 0.50, intensity: 0.60 }
            } else { EmotionTag::NEUTRAL }
        }
        0x05 => { // Sound
            if val > 80.0 {
                EmotionTag { valence: -0.40, arousal: 0.85, dominance: 0.25, intensity: 0.70 }
            } else { EmotionTag::NEUTRAL }
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

    fn aam()   -> ISLAddress { ISLAddress::new(0, 0, 0, 0) }
    fn leo()   -> ISLAddress { ISLAddress::new(0, 0, 0, 1) }
    fn chief() -> ISLAddress { ISLAddress::new(1, 0, 0, 0) }
    fn worker_addr(i: u8) -> ISLAddress { ISLAddress::new(2, 0, 0, i) }

    fn make_chief() -> Chief {
        Chief::new(chief(), aam(), leo(), ChiefKind::Home)
    }

    fn sensor_frame(from: ISLAddress, to: ISLAddress, temp: f32, ts: i64) -> ISLFrame {
        let msg  = ISLMessage::with_payload(from, to, MsgType::ChainPayload, [1, 1, 0]);
        // Body: sensor_id(1) + unit(1=temp) + f32(4) + i64(8)
        let mut body = alloc::vec![1u8, 0x01]; // sensor_id=1, unit=Temperature
        body.extend_from_slice(&temp.to_be_bytes());
        body.extend_from_slice(&ts.to_be_bytes());
        ISLFrame::with_body(msg, body)
    }

    #[test]
    fn chief_starts_sleeping() {
        let c = make_chief();
        assert_eq!(c.state, ChiefState::Sleeping);
        assert_eq!(c.alive_count(), 0);
    }

    #[test]
    fn register_worker() {
        let mut c = make_chief();
        assert!(c.register_worker(worker_addr(1), 0x01, 1000)); // Sensor → Home OK
        assert!(c.register_worker(worker_addr(2), 0x02, 1000)); // Actuator → Home OK
        assert_eq!(c.alive_count(), 2);
    }

    #[test]
    fn domain_routing_rejects_wrong_kind() {
        let mut home = Chief::new(chief(), aam(), leo(), ChiefKind::Home);
        assert!(!home.register_worker(worker_addr(1), 0x03, 1000), "Camera → Home = rejected");
        assert!(!home.register_worker(worker_addr(2), 0x04, 1000), "Network → Home = rejected");
        assert_eq!(home.alive_count(), 0);

        let mut vision = Chief::new(chief(), aam(), leo(), ChiefKind::Vision);
        assert!(vision.register_worker(worker_addr(1), 0x03, 1000), "Camera → Vision = OK");
        assert!(!vision.register_worker(worker_addr(2), 0x01, 1000), "Sensor → Vision = rejected");

        let mut net = Chief::new(chief(), aam(), leo(), ChiefKind::Network);
        assert!(net.register_worker(worker_addr(1), 0x04, 1000), "Network → Network = OK");
        assert!(!net.register_worker(worker_addr(2), 0x01, 1000), "Sensor → Network = rejected");
    }

    #[test]
    fn general_chief_accepts_all() {
        let mut gen = Chief::new(chief(), aam(), leo(), ChiefKind::General);
        assert!(gen.register_worker(worker_addr(1), 0x01, 1000));
        assert!(gen.register_worker(worker_addr(2), 0x03, 1000));
        assert!(gen.register_worker(worker_addr(3), 0x04, 1000));
        assert_eq!(gen.alive_count(), 3);
    }

    #[test]
    fn receive_sensor_creates_report() {
        let mut c = make_chief();
        let w1 = worker_addr(1);
        assert!(c.register_worker(w1, 0x01, 1000));

        let frame = sensor_frame(w1, chief(), 22.0, 1000);
        c.receive_frame(frame, 1000);

        let reports = c.drain_reports();
        assert_eq!(reports.len(), 1, "Sensor frame → 1 report cho LeoAI");
        assert_eq!(reports[0].from_worker, w1);
    }

    #[test]
    fn chief_sleeps_after_processing() {
        let mut c = make_chief();
        let frame = sensor_frame(worker_addr(1), chief(), 30.0, 1000);
        c.receive_frame(frame, 1000);
        assert_eq!(c.state, ChiefState::Sleeping, "Sleep ngay sau xử lý");
    }

    #[test]
    fn forward_command_to_worker() {
        let mut c  = make_chief();
        let target = worker_addr(3);
        c.forward_command(target, 0x01, 0xFF); // ON command

        let cmds = c.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].header.to,       target);
        assert_eq!(cmds[0].header.from,     chief());
        assert_eq!(cmds[0].header.msg_type, MsgType::ActuatorCmd);
    }

    #[test]
    fn aam_to_worker_via_chief() {
        // AAM không được ra lệnh trực tiếp cho Worker
        // AAM → Chief → Worker
        let mut c  = make_chief();
        let target = worker_addr(1);
        // AAM gửi command đến Chief (simulate)
        let aam_cmd = ISLMessage::actuator(aam(), chief(), 0x01, 0x00);
        let aam_frame = ISLFrame::bare(aam_cmd);

        // Chief nhận rồi forward xuống Worker
        // (trong thực tế Chief parse cmd và gọi forward_command)
        c.forward_command(target, 0x01, 0x00);
        let cmds = c.drain_commands();

        assert_eq!(cmds[0].header.from, chief(), "Command phải từ Chief, không phải AAM");
        assert_eq!(cmds[0].header.to,   target);
    }

    #[test]
    fn emergency_forwarded_to_aam() {
        let mut c = make_chief();
        let emerg = ISLMessage::emergency(worker_addr(1), 0xFF);
        let frame = ISLFrame::bare(emerg);
        c.receive_frame(frame, 1000);

        let cmds = c.drain_commands();
        // Emergency → Chief tạo alert gửi lên AAM
        assert!(!cmds.is_empty(), "Emergency phải tạo alert");
        assert_eq!(cmds[0].header.msg_type, MsgType::Emergency);
    }

    #[test]
    fn heartbeat_marks_dead() {
        let mut c = make_chief();
        assert!(c.register_worker(worker_addr(1), 0x01, 0));
        // 10 giây trôi qua, timeout = 5 giây
        c.heartbeat_check(10_000_000_000, 5_000_000_000);
        assert_eq!(c.alive_count(), 0, "Worker không báo cáo → dead");
    }

    #[test]
    fn hot_temp_emotion_detected() {
        let mut c = make_chief();
        let frame = sensor_frame(worker_addr(1), chief(), 40.0, 1000); // nóng
        c.receive_frame(frame, 1000);
        let reports = c.drain_reports();
        assert!(reports[0].emotion.arousal > 0.5, "Nóng → arousal cao");
        assert!(reports[0].emotion.valence < 0.0, "Nóng → valence âm");
    }

    // ── Domain-specific tests ────────────────────────────────────────────────

    #[test]
    fn home_chief_automation_hot() {
        let mut c = make_chief(); // Home kind
        // Đăng ký actuator worker
        assert!(c.register_worker(worker_addr(5), 0x02, 1000)); // Actuator

        // Gửi temp > 35°C → phải tạo command cho actuators
        let frame = sensor_frame(worker_addr(1), chief(), 40.0, 2000);
        c.receive_frame(frame, 2000);

        let cmds = c.drain_commands();
        assert!(!cmds.is_empty(), "Nóng > 35°C → HomeChief phải command actuator");
        assert_eq!(cmds[0].header.msg_type, MsgType::ActuatorCmd);
        assert_eq!(cmds[0].header.to, worker_addr(5));
    }

    #[test]
    fn home_chief_automation_cold() {
        let mut c = make_chief();
        assert!(c.register_worker(worker_addr(5), 0x02, 1000)); // Actuator

        let frame = sensor_frame(worker_addr(1), chief(), 5.0, 2000); // lạnh
        c.receive_frame(frame, 2000);

        let cmds = c.drain_commands();
        assert!(!cmds.is_empty(), "Lạnh < 10°C → command actuator");
    }

    #[test]
    fn home_chief_tracks_temp() {
        let mut c = make_chief();
        assert!(c.last_temp.is_none());

        let frame = sensor_frame(worker_addr(1), chief(), 25.5, 1000);
        c.receive_frame(frame, 1000);

        assert!((c.last_temp.unwrap() - 25.5).abs() < 0.01);
    }

    #[test]
    fn home_chief_no_automation_normal_temp() {
        let mut c = make_chief();
        assert!(c.register_worker(worker_addr(5), 0x02, 1000)); // Actuator

        let frame = sensor_frame(worker_addr(1), chief(), 22.0, 2000); // bình thường
        c.receive_frame(frame, 2000);

        let cmds = c.drain_commands();
        assert!(cmds.is_empty(), "22°C → không cần automation");
    }

    #[test]
    fn network_chief_escalates_on_emergency() {
        let mut c = Chief::new(chief(), aam(), leo(), ChiefKind::Network);
        assert_eq!(c.security_level, 0);

        let emerg = ISLMessage::emergency(worker_addr(1), 0xFF);
        let frame = ISLFrame::bare(emerg);
        c.receive_frame(frame, 1000);

        assert_eq!(c.security_level, 2, "Emergency → critical security level");
    }

    #[test]
    fn network_chief_reset_security() {
        let mut c = Chief::new(chief(), aam(), leo(), ChiefKind::Network);
        c.security_level = 2;
        c.reset_security();
        assert_eq!(c.security_level, 0);
    }

    #[test]
    fn vision_chief_counts_motion() {
        let mut c = Chief::new(chief(), aam(), leo(), ChiefKind::Vision);
        assert_eq!(c.motion_events, 0);

        // Gửi motion sensor data (unit=0x04, value > 0.5)
        let msg = ISLMessage::with_payload(worker_addr(1), chief(), MsgType::ChainPayload, [2, 4, 0]);
        let motion_val: f32 = 1.0;
        let mut body = alloc::vec![2u8, 0x04]; // sensor_id=2, unit=Motion
        body.extend_from_slice(&motion_val.to_be_bytes());
        body.extend_from_slice(&1000i64.to_be_bytes());
        let frame = ISLFrame::with_body(msg, body);
        c.receive_frame(frame, 1000);

        assert_eq!(c.motion_events, 1, "Motion event detected");
    }

    #[test]
    fn vision_chief_reset_motion() {
        let mut c = Chief::new(chief(), aam(), leo(), ChiefKind::Vision);
        c.motion_events = 5;
        c.reset_motion();
        assert_eq!(c.motion_events, 0);
    }
}
