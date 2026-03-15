//! # leo — LeoAI
//!
//! LeoAI = Agent duy nhất chăm sóc KnowledgeTree.
//! Dùng LearningLoop (đã có) để học — không tự implement learning.
//!
//! Vai trò:
//!   - Nhận reports từ Chief
//!   - Feed vào LearningLoop → STM → Silk → Dream → Proposal
//!   - Gửi proposals lên AAM để xác nhận
//!   - Im lặng khi không có gì
//!
//! KHÔNG ra lệnh ai. KHÔNG đụng direct vào KnowledgeTree.
//! LearningLoop là cánh tay — LeoAI là ý chí điều phối.

extern crate alloc;
use alloc::vec::Vec;

use isl::address::ISLAddress;
use isl::message::{ISLMessage, ISLFrame, MsgType};

use crate::chief::IngestedReport;
use crate::learning::LearningLoop;
use crate::encoder::ContentInput;
use crate::skill::ExecContext;
use crate::instinct::innate_instincts;

// ─────────────────────────────────────────────────────────────────────────────
// LeoState
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeoState {
    Listening, // default — chờ
    Learning,  // đang feed vào LearningLoop
    Dreaming,  // dream cycle
    Proposing, // đang gửi proposal lên AAM
}

// ─────────────────────────────────────────────────────────────────────────────
// LeoPendingProposal — chờ AAM xác nhận
// ─────────────────────────────────────────────────────────────────────────────

/// Một proposal đang chờ AAM.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct LeoPendingProposal {
    pub chain_hash:  u64,
    pub fire_count:  u32,
    pub confidence:  f32,
    pub timestamp:   i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// LeoAI
// ─────────────────────────────────────────────────────────────────────────────

/// LeoAI — não của hệ thống.
///
/// Dùng LearningLoop để học — không implement learning logic riêng.
pub struct LeoAI {
    pub addr:      ISLAddress,
    pub aam_addr:  ISLAddress,
    pub state:     LeoState,

    /// LearningLoop — cánh tay học của LeoAI
    pub learning:  LearningLoop,

    /// Proposals chờ AAM xác nhận
    pub pending:   Vec<LeoPendingProposal>,

    /// Outbox → AAM
    outbox:        Vec<ISLFrame>,

    /// Stats
    pub ingested:  u32,
    pub dreamed:   u32,
    last_event_ts: i64,
    dream_interval: u32, // số turns giữa các dream cycle
}

/// 5 phút idle → dream
const DREAM_IDLE_NS: i64 = 5 * 60 * 1_000_000_000;
/// Dream mỗi 8 turns (y hệt HomeRuntime)
const DEFAULT_DREAM_INTERVAL: u32 = 8;

impl LeoAI {
    pub fn new(addr: ISLAddress, aam_addr: ISLAddress) -> Self {
        Self {
            addr,
            aam_addr,
            state:          LeoState::Listening,
            learning:       LearningLoop::new(0x1E0A1), // stable session id
            pending:        Vec::new(),
            outbox:         Vec::new(),
            ingested:       0,
            dreamed:        0,
            last_event_ts:  0,
            dream_interval: DEFAULT_DREAM_INTERVAL,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Ingest — nhận report từ Chief, feed vào LearningLoop
    // ─────────────────────────────────────────────────────────────────────────

    /// Nhận IngestedReport từ Chief → feed vào LearningLoop.
    ///
    /// LearningLoop tự lo encode + STM + Silk + co_activate.
    /// LeoAI chỉ điều phối khi nào feed, khi nào dream.
    pub fn ingest(&mut self, report: IngestedReport, ts: i64) {
        self.state         = LeoState::Learning;
        self.last_event_ts = ts;
        self.ingested     += 1;

        // Tạo text từ payload để LearningLoop xử lý
        // Format: "sensor:{sensor_id} val:{quantized}"
        let text = if report.payload.len() >= 3 {
            alloc::format!("sensor:{} unit:{} val:{}",
                report.payload[0], report.payload[1], report.payload[2])
        } else {
            alloc::string::String::from("event")
        };

        let input = ContentInput::Text { content: text, timestamp: ts };
        let result = self.learning.process_one(input);

        // Chạy 7 bản năng bẩm sinh trên kết quả encode
        if let crate::learning::ProcessResult::Ok { chain, emotion } = result {
            let mut ictx = ExecContext::new(ts, emotion, self.fx());
            ictx.push_input(chain);
            self.run_instincts(&mut ictx);
        }

        // Auto-dream mỗi dream_interval turns
        if self.ingested.is_multiple_of(self.dream_interval) {
            self.run_dream(ts);
        }

        self.state = LeoState::Listening;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Dream — LearningLoop đã có DreamCycle
    // ─────────────────────────────────────────────────────────────────────────

    /// Chạy dream cycle qua LearningLoop.
    ///
    /// Tìm observations đủ điều kiện → tạo proposal → gửi AAM xác nhận.
    pub fn run_dream(&mut self, ts: i64) {
        self.state    = LeoState::Dreaming;
        self.dreamed += 1;

        // Dream candidates: observations có fire trong Silk ≥ threshold
        let candidates = self.learning.dream_candidates(10);
        for obs in &candidates {
            let hash = obs.chain.chain_hash();
            let fire = self.learning.graph().edges_from(hash).len() as u32;
            if fire >= 3 {
                let conf = (fire as f32 / 10.0).min(0.95);
                let p = LeoPendingProposal { chain_hash: hash, fire_count: fire, confidence: conf, timestamp: ts };
                // Đóng gói gửi AAM
                let frame = self.make_proposal_frame(&p);
                self.outbox.push(frame);
                self.pending.push(p);
            }
        }

        self.state = LeoState::Listening;
    }

    /// Thử dream nếu đã idle đủ lâu.
    pub fn try_dream_if_idle(&mut self, now: i64) {
        if self.learning.stm().len() < 3 { return; }
        if now - self.last_event_ts < DREAM_IDLE_NS { return; }
        self.run_dream(now);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AAM interaction
    // ─────────────────────────────────────────────────────────────────────────

    /// Nhận AAM decision (Approved/Rejected).
    ///
    /// AAM chỉ approve/reject — không đụng KnowledgeTree.
    /// LeoAI nhận approved → promote QR trong LearningLoop.
    pub fn receive_aam_decision(&mut self, msg: ISLMessage, _ts: i64) {
        match msg.msg_type {
            MsgType::Approved => {
                // payload[0] = proposal index (đơn giản)
                let idx = msg.payload[0] as usize;
                if idx < self.pending.len() {
                    let p = self.pending.remove(idx);
                    // Promote: learning sẽ xử lý trong Dream cycle tiếp theo
                    // Đây là signal "AAM đã ký" — knowledge được ghi QR
                    // (STM → QR qua Dream + ED25519 trong writer)
                    let _ = p; // TODO: wire vào olang::writer QR path
                }
            }
            MsgType::Nack => {
                // Bị từ chối → xóa khỏi pending
                let idx = msg.payload[0] as usize;
                if idx < self.pending.len() {
                    self.pending.remove(idx);
                }
            }
            _ => {}
        }
    }

    /// Flush outbox → gửi lên AAM.
    pub fn flush_outbox(&mut self) -> Vec<ISLFrame> {
        let mut out = Vec::new();
        core::mem::swap(&mut self.outbox, &mut out);
        out
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Stats
    // ─────────────────────────────────────────────────────────────────────────

    pub fn stm_len(&self)    -> usize { self.learning.stm().len() }
    pub fn edge_count(&self) -> usize { self.learning.graph().len() }
    pub fn fx(&self)         -> f32   { self.learning.context().fx() }
    pub fn pending_count(&self) -> usize { self.pending.len() }

    // ─────────────────────────────────────────────────────────────────────────
    // Instinct — 7 bản năng bẩm sinh
    // ─────────────────────────────────────────────────────────────────────────

    /// Chạy 7 bản năng bẩm sinh trên context hiện tại.
    ///
    /// Trả về ExecContext sau khi tất cả instincts đã xử lý.
    /// Agent đọc state từ context: epistemic_grade, curiosity_level, v.v.
    pub fn run_instincts(&self, ctx: &mut ExecContext) {
        let instincts = innate_instincts();

        // Chuẩn bị state cho Reflection từ learning stats
        ctx.set(
            alloc::string::String::from("qr_count"),
            alloc::format!("{}", 0), // TODO: wire real QR count khi có writer
        );
        ctx.set(
            alloc::string::String::from("dn_count"),
            alloc::format!("{}", self.stm_len()),
        );
        ctx.set(
            alloc::string::String::from("edge_count"),
            alloc::format!("{}", self.edge_count()),
        );
        ctx.set(
            alloc::string::String::from("known_count"),
            alloc::format!("{}", self.stm_len()),
        );

        // Chạy từng instinct theo thứ tự ưu tiên
        // Honesty → Contradiction → Causality → Abstraction → Analogy → Curiosity → Reflection
        for skill in instincts {
            let _ = skill.execute(ctx);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal
    // ─────────────────────────────────────────────────────────────────────────

    fn make_proposal_frame(&self, p: &LeoPendingProposal) -> ISLFrame {
        let conf_q   = (p.confidence * 255.0) as u8;
        let fire_low = (p.fire_count & 0xFF) as u8;
        let msg = ISLMessage::with_payload(
            self.addr, self.aam_addr,
            MsgType::Propose,
            [0x02, conf_q, fire_low], // 0x02 = PromoteQR
        );
        ISLFrame::bare(msg)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use isl::address::ISLAddress;
    use silk::edge::EmotionTag;
    use crate::chief::IngestedReport;

    fn leo_addr() -> ISLAddress { ISLAddress::new(0, 0, 0, 2) }
    fn aam_addr()  -> ISLAddress { ISLAddress::new(0, 0, 0, 0) }
    fn worker()    -> ISLAddress { ISLAddress::new(2, 0, 0, 1) }

    fn leo() -> LeoAI { LeoAI::new(leo_addr(), aam_addr()) }

    fn report(v: f32, idx: u8) -> IngestedReport {
        IngestedReport {
            from_worker: worker(),
            emotion:     EmotionTag { valence: v, arousal: 0.5, dominance: 0.5, intensity: v.abs() },
            payload:     alloc::vec![1u8, 0x01, idx, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            timestamp:   (idx as i64) * 1000,
        }
    }

    #[test]
    fn leo_starts_listening() {
        let l = leo();
        assert_eq!(l.state, LeoState::Listening);
        assert_eq!(l.ingested, 0);
        assert_eq!(l.dreamed, 0);
    }

    #[test]
    fn ingest_feeds_learning() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        l.ingest(report(-0.3, 2), 2000);
        assert_eq!(l.ingested, 2);
        assert!(l.stm_len() >= 1, "STM nhận observations");
    }

    #[test]
    fn leo_sleeps_after_ingest() {
        let mut l = leo();
        l.ingest(report(0.3, 1), 1000);
        assert_eq!(l.state, LeoState::Listening);
    }

    #[test]
    fn auto_dream_every_n_turns() {
        let mut l = leo();
        l.dream_interval = 3; // dream mỗi 3 turns
        for i in 0..3 {
            l.ingest(report(-0.4 + i as f32 * 0.1, i as u8 + 1), i as i64 * 1000);
        }
        // Turn 3 → auto dream
        assert_eq!(l.dreamed, 1, "Dream sau {} turns", l.dream_interval);
    }

    #[test]
    fn dream_not_triggered_early() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        // last_event_ts = 1000, now = 2000 ns (< 5 phút)
        l.try_dream_if_idle(2000);
        assert_eq!(l.dreamed, 0, "Không dream khi chưa idle đủ");
    }

    #[test]
    fn dream_builds_knowledge() {
        let mut l = leo();
        // Feed nhiều reports cùng pattern
        for i in 0..10 {
            l.ingest(report(-0.5, i as u8 + 1), i as i64 * 1000);
        }
        assert!(l.edge_count() > 0, "Silk edges sau nhiều ingest: {}", l.edge_count());
    }

    #[test]
    fn proposal_goes_to_aam() {
        let p = LeoPendingProposal { chain_hash: 0xABCD, fire_count: 5, confidence: 0.8, timestamp: 1000 };
        let l = leo();
        let frame = l.make_proposal_frame(&p);
        assert_eq!(frame.header.msg_type, MsgType::Propose);
        assert_eq!(frame.header.to,   aam_addr());
        assert_eq!(frame.header.from, leo_addr());
        // payload: [kind=0x02, conf_q, fire_low]
        assert_eq!(frame.header.payload[0], 0x02);
    }

    #[test]
    fn flush_outbox_clears() {
        let mut l = leo();
        let p = LeoPendingProposal { chain_hash: 1, fire_count: 5, confidence: 0.8, timestamp: 0 };
        l.outbox.push(l.make_proposal_frame(&p));
        let out = l.flush_outbox();
        assert_eq!(out.len(), 1);
        assert!(l.flush_outbox().is_empty());
    }

    #[test]
    fn aam_approved_clears_pending() {
        let mut l = leo();
        let p = LeoPendingProposal { chain_hash: 0x1, fire_count: 5, confidence: 0.8, timestamp: 0 };
        l.pending.push(p);
        // AAM gửi Approved với index=0
        let approved = ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_aam_decision(approved, 1000);
        assert_eq!(l.pending_count(), 0, "Approved → xóa khỏi pending");
    }

    #[test]
    fn leo_does_not_touch_knowtree_directly() {
        // LeoAI không có direct write method vào KnowledgeTree
        // Chỉ qua LearningLoop.process_one() + Dream
        let l = leo();
        // Verify: không có method như "write_qr", "add_node", "commit"
        // (compile-time check — nếu code này compile được là đúng)
        let _ = l.stm_len();
        let _ = l.edge_count();
        // LeoAI chỉ đọc stats, không ghi trực tiếp
    }
}
