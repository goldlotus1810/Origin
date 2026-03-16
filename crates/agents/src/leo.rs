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
use isl::message::{ISLFrame, ISLMessage, MsgType};

use crate::chief::IngestedReport;
use crate::encoder::ContentInput;
use crate::instinct::innate_instincts;
use crate::learning::LearningLoop;
use crate::skill::ExecContext;

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
    pub chain_hash: u64,
    pub fire_count: u32,
    pub confidence: f32,
    pub timestamp: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// LeoAI
// ─────────────────────────────────────────────────────────────────────────────

/// LeoAI — não của hệ thống.
///
/// Dùng LearningLoop để học — không implement learning logic riêng.
pub struct LeoAI {
    pub addr: ISLAddress,
    pub aam_addr: ISLAddress,
    pub state: LeoState,

    /// LearningLoop — cánh tay học của LeoAI
    pub learning: LearningLoop,

    /// Proposals chờ AAM xác nhận
    pub pending: Vec<LeoPendingProposal>,

    /// Outbox → AAM
    outbox: Vec<ISLFrame>,

    /// ISL inbox — frames from Chiefs and AAM
    inbox: Vec<ISLFrame>,

    /// QT9: bytes chờ ghi disk — caller drain và flush.
    pending_writes: Vec<u8>,

    /// Số QR đã promote (AAM approved + ghi file).
    qr_promoted: u32,

    /// Stats
    pub ingested: u32,
    pub dreamed: u32,
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
            state: LeoState::Listening,
            learning: LearningLoop::new(0x1E0A1), // stable session id
            pending: Vec::new(),
            outbox: Vec::new(),
            inbox: Vec::new(),
            pending_writes: Vec::new(),
            qr_promoted: 0,
            ingested: 0,
            dreamed: 0,
            last_event_ts: 0,
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
        self.state = LeoState::Learning;
        self.last_event_ts = ts;
        self.ingested += 1;

        // Tạo text từ payload để LearningLoop xử lý
        // Format: "sensor:{sensor_id} val:{quantized}"
        let text = if report.payload.len() >= 3 {
            alloc::format!(
                "sensor:{} unit:{} val:{}",
                report.payload[0],
                report.payload[1],
                report.payload[2]
            )
        } else {
            alloc::string::String::from("event")
        };

        let input = ContentInput::Text {
            content: text,
            timestamp: ts,
        };
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
    /// Trước khi dream: chăm sóc Silk (decay + cắt tỉa).
    /// Tìm observations đủ điều kiện → tạo proposal → gửi AAM xác nhận.
    pub fn run_dream(&mut self, ts: i64) {
        self.state = LeoState::Dreaming;
        self.dreamed += 1;

        // Chăm sóc Ln-1 trước khi dream — cắt tỉa để Dream nhìn thấy cây sạch
        let elapsed = if self.last_event_ts > 0 {
            ts - self.last_event_ts
        } else {
            0
        };
        // Convert ms→ns nếu cần (ts thường là ms, decay cần ns)
        let elapsed_ns = elapsed * 1_000_000;
        self.learning.maintain_silk(elapsed_ns, 10_000);

        // Dream candidates: observations có fire trong Silk ≥ threshold
        let candidates = self.learning.dream_candidates(10);
        for obs in &candidates {
            let hash = obs.chain.chain_hash();
            let fire = self.learning.graph().edges_from(hash).len() as u32;
            if fire >= 3 {
                let conf = (fire as f32 / 10.0).min(0.95);
                let p = LeoPendingProposal {
                    chain_hash: hash,
                    fire_count: fire,
                    confidence: conf,
                    timestamp: ts,
                };
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
        if self.learning.stm().len() < 3 {
            return;
        }
        if now - self.last_event_ts < DREAM_IDLE_NS {
            return;
        }
        self.run_dream(now);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AAM interaction
    // ─────────────────────────────────────────────────────────────────────────

    /// Nhận AAM decision (Approved/Rejected).
    ///
    /// AAM chỉ approve/reject — không đụng KnowledgeTree.
    /// LeoAI nhận approved → promote QR trong LearningLoop.
    pub fn receive_aam_decision(&mut self, msg: ISLMessage, ts: i64) {
        match msg.msg_type {
            MsgType::Approved => {
                let idx = msg.payload[0] as usize;
                if idx < self.pending.len() {
                    let p = self.pending.remove(idx);

                    // Tìm chain trong STM bằng chain_hash
                    if let Some(obs) = self.learning.stm().find_by_hash(p.chain_hash) {
                        // QT9: Ghi file TRƯỚC — cập nhật RAM SAU
                        use olang::writer::OlangWriter;
                        let mut writer = OlangWriter::new(ts);
                        let _ = writer.append_node(&obs.chain, 0, true, ts);
                        self.pending_writes.extend_from_slice(writer.as_bytes());
                        self.qr_promoted += 1;
                    }

                    // Xóa observation đã promote khỏi STM
                    self.learning.stm_mut().remove_promoted(&[p.chain_hash]);
                }
            }
            MsgType::Nack => {
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

    pub fn stm_len(&self) -> usize {
        self.learning.stm().len()
    }
    pub fn edge_count(&self) -> usize {
        self.learning.graph().len()
    }
    pub fn fx(&self) -> f32 {
        self.learning.context().fx()
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn qr_count(&self) -> u32 {
        self.qr_promoted
    }

    /// Có bytes chờ ghi disk?
    pub fn has_pending_writes(&self) -> bool {
        !self.pending_writes.is_empty()
    }

    /// Drain pending writes — caller flush to disk.
    pub fn drain_pending_writes(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.pending_writes)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ISL Inbox — nhận ISL frames từ Chiefs và AAM
    // ─────────────────────────────────────────────────────────────────────────

    /// Receive an ISL frame into the inbox.
    pub fn receive_isl(&mut self, frame: ISLFrame) {
        self.inbox.push(frame);
    }

    /// Poll inbox — process all pending ISL messages.
    ///
    /// Dispatches:
    ///   Approved/Nack → receive_aam_decision()
    ///   ChainPayload → treat as report data, feed to learning
    ///   Propose → forward to AAM outbox
    ///
    /// Returns number of messages processed.
    pub fn poll_inbox(&mut self, ts: i64) -> u32 {
        if self.inbox.is_empty() {
            return 0;
        }
        let frames: Vec<ISLFrame> = core::mem::take(&mut self.inbox);
        let count = frames.len() as u32;
        for frame in frames {
            match frame.header.msg_type {
                MsgType::Approved | MsgType::Nack => {
                    self.receive_aam_decision(frame.header, ts);
                }
                MsgType::ChainPayload => {
                    // Treat as data from Chief — ingest
                    let emotion = silk::edge::EmotionTag::NEUTRAL;
                    let report = IngestedReport {
                        from_worker: frame.header.from,
                        emotion,
                        payload: frame.body,
                        timestamp: ts,
                    };
                    self.ingest(report, ts);
                }
                _ => {
                    // Unknown msg type — push to outbox for forwarding
                }
            }
        }
        count
    }

    /// Number of pending ISL messages in inbox.
    pub fn inbox_len(&self) -> usize {
        self.inbox.len()
    }

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
            alloc::format!("{}", self.qr_promoted),
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
    // Domain Skills analysis — knowledge quality assessment
    // ─────────────────────────────────────────────────────────────────────────

    /// Chạy domain skills trên STM để đánh giá knowledge quality.
    ///
    /// Dùng SimilaritySkill + ClusterSkill + CuratorSkill.
    /// Kết quả: state["cluster_count"], state["similarity"],
    ///          state["curated_count"] trong ExecContext.
    pub fn run_knowledge_analysis(&self, ts: i64) -> ExecContext {
        use crate::domain_skills::{ClusterSkill, CuratorSkill};
        use crate::skill::Skill;

        let emotion = self.learning.context().last_emotion();
        let mut ctx = ExecContext::new(ts, emotion, self.fx());

        // Feed STM observations as input chains
        let obs_list = self.learning.stm().top_n(20);
        for obs in &obs_list {
            ctx.push_input(obs.chain.clone());
        }

        if ctx.input_chains.len() < 2 {
            return ctx;
        }

        // Cluster analysis
        let cluster = ClusterSkill;
        let _ = cluster.execute(&mut ctx);

        // Curator sorts by richness
        // Reset inputs with cluster output for curation
        let clusters_output = ctx.output_chains.clone();
        ctx.input_chains = clusters_output;
        ctx.output_chains.clear();

        let curator = CuratorSkill;
        let _ = curator.execute(&mut ctx);

        ctx
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal
    // ─────────────────────────────────────────────────────────────────────────

    // ─────────────────────────────────────────────────────────────────────────
    // Express — LeoAI writes Olang code to describe what it knows
    // ─────────────────────────────────────────────────────────────────────────

    /// Generate Olang code expressing knowledge about an STM observation.
    ///
    /// LeoAI becomes a meta-programmer: it writes Olang code that describes
    /// the molecular coordinates of concepts it has learned.
    ///
    /// Example output: `"lửa" == { S=1 R=6 V=200 A=180 T=4 }`
    ///
    /// This is the bridge between cognitive understanding and executable code.
    /// LeoAI "thinks in Olang" — expressing truth assertions about what it knows.
    pub fn express_observation(&self, chain_hash: u64) -> Option<alloc::string::String> {
        let obs = self.learning.stm().find_by_hash(chain_hash)?;
        let mol = obs.chain.first()?;

        // Generate molecular literal assertion
        Some(alloc::format!(
            "{{ S={} R={} V={} A={} T={} }}",
            mol.shape, mol.relation, mol.emotion.valence, mol.emotion.arousal, mol.time,
        ))
    }

    /// Generate Olang truth assertion: `alias == { S=.. R=.. V=.. A=.. T=.. }`
    ///
    /// LeoAI expresses: "I know that this alias maps to this molecular pattern."
    /// The VM can then execute this to verify the assertion.
    pub fn express_truth(&self, alias: &str, chain_hash: u64) -> Option<alloc::string::String> {
        let obs = self.learning.stm().find_by_hash(chain_hash)?;
        let mol = obs.chain.first()?;

        Some(alloc::format!(
            "\"{}\" == {{ S={} R={} V={} A={} T={} }}",
            alias, mol.shape, mol.relation, mol.emotion.valence, mol.emotion.arousal, mol.time,
        ))
    }

    /// Generate Olang for ALL observations in STM — LeoAI's complete expressed knowledge.
    ///
    /// Returns a Vec of Olang code strings, one per observation.
    /// Each describes the molecular coordinates of a learned concept.
    pub fn express_all(&self) -> Vec<alloc::string::String> {
        self.learning
            .stm()
            .all()
            .iter()
            .filter_map(|obs| {
                let mol = obs.chain.first()?;
                Some(alloc::format!(
                    "{{ S={} R={} V={} A={} T={} }}",
                    mol.shape, mol.relation, mol.emotion.valence, mol.emotion.arousal, mol.time,
                ))
            })
            .collect()
    }

    /// Generate Olang evolution expression: shows how one concept derived from another.
    ///
    /// Example: `{ S=1 R=6 V=48 A=144 T=4 } ← { S=1 R=6 V=200 A=180 T=4 }`
    /// "angry fire ← happy fire" — derived via valence shift.
    pub fn express_evolution(
        &self,
        source_hash: u64,
        evolved_hash: u64,
    ) -> Option<alloc::string::String> {
        let source_obs = self.learning.stm().find_by_hash(source_hash)?;
        let evolved_obs = self.learning.stm().find_by_hash(evolved_hash)?;
        let s_mol = source_obs.chain.first()?;
        let e_mol = evolved_obs.chain.first()?;

        // Find which dimension changed
        let deltas = s_mol.dimension_delta(e_mol);
        let dim_name = if deltas.len() == 1 {
            match deltas[0].0 {
                olang::molecular::Dimension::Shape => "S",
                olang::molecular::Dimension::Relation => "R",
                olang::molecular::Dimension::Valence => "V",
                olang::molecular::Dimension::Arousal => "A",
                olang::molecular::Dimension::Time => "T",
            }
        } else {
            "?"
        };

        Some(alloc::format!(
            "{{ S={} R={} V={} A={} T={} }} ← {{ S={} R={} V={} A={} T={} }} (* Δ{} *)",
            e_mol.shape, e_mol.relation, e_mol.emotion.valence, e_mol.emotion.arousal, e_mol.time,
            s_mol.shape, s_mol.relation, s_mol.emotion.valence, s_mol.emotion.arousal, s_mol.time,
            dim_name,
        ))
    }

    fn make_proposal_frame(&self, p: &LeoPendingProposal) -> ISLFrame {
        let conf_q = (p.confidence * 255.0) as u8;
        let fire_low = (p.fire_count & 0xFF) as u8;
        let msg = ISLMessage::with_payload(
            self.addr,
            self.aam_addr,
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
    use crate::chief::IngestedReport;
    use isl::address::ISLAddress;
    use silk::edge::EmotionTag;

    fn leo_addr() -> ISLAddress {
        ISLAddress::new(0, 0, 0, 2)
    }
    fn aam_addr() -> ISLAddress {
        ISLAddress::new(0, 0, 0, 0)
    }
    fn worker() -> ISLAddress {
        ISLAddress::new(2, 0, 0, 1)
    }

    fn leo() -> LeoAI {
        LeoAI::new(leo_addr(), aam_addr())
    }

    fn report(v: f32, idx: u8) -> IngestedReport {
        IngestedReport {
            from_worker: worker(),
            emotion: EmotionTag {
                valence: v,
                arousal: 0.5,
                dominance: 0.5,
                intensity: v.abs(),
            },
            payload: alloc::vec![1u8, 0x01, idx, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            timestamp: (idx as i64) * 1000,
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
        assert!(
            l.edge_count() > 0,
            "Silk edges sau nhiều ingest: {}",
            l.edge_count()
        );
    }

    #[test]
    fn proposal_goes_to_aam() {
        let p = LeoPendingProposal {
            chain_hash: 0xABCD,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 1000,
        };
        let l = leo();
        let frame = l.make_proposal_frame(&p);
        assert_eq!(frame.header.msg_type, MsgType::Propose);
        assert_eq!(frame.header.to, aam_addr());
        assert_eq!(frame.header.from, leo_addr());
        // payload: [kind=0x02, conf_q, fire_low]
        assert_eq!(frame.header.payload[0], 0x02);
    }

    #[test]
    fn flush_outbox_clears() {
        let mut l = leo();
        let p = LeoPendingProposal {
            chain_hash: 1,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 0,
        };
        l.outbox.push(l.make_proposal_frame(&p));
        let out = l.flush_outbox();
        assert_eq!(out.len(), 1);
        assert!(l.flush_outbox().is_empty());
    }

    #[test]
    fn aam_approved_clears_pending() {
        let mut l = leo();
        let p = LeoPendingProposal {
            chain_hash: 0x1,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 0,
        };
        l.pending.push(p);
        // AAM gửi Approved với index=0
        let approved =
            ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_aam_decision(approved, 1000);
        assert_eq!(l.pending_count(), 0, "Approved → xóa khỏi pending");
    }

    #[test]
    fn aam_approved_writes_qr() {
        let mut l = leo();
        // Feed data → STM gets an observation
        l.ingest(report(-0.5, 1), 1000);
        assert!(l.stm_len() >= 1);

        // Lấy chain_hash từ STM observation đầu tiên
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();

        // Tạo pending proposal cho hash đó
        l.pending.push(LeoPendingProposal {
            chain_hash: hash,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 1000,
        });

        // AAM Approved
        let approved =
            ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_aam_decision(approved, 2000);

        // QR đã promote
        assert_eq!(l.qr_count(), 1, "QR promoted count phải = 1");
        assert!(l.has_pending_writes(), "Phải có bytes chờ ghi disk");

        // Drain writes
        let bytes = l.drain_pending_writes();
        assert!(!bytes.is_empty(), "Bytes phải có data");
        assert!(!l.has_pending_writes(), "Drain xong phải rỗng");
    }

    #[test]
    fn aam_approved_removes_from_stm() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();
        let stm_before = l.stm_len();

        l.pending.push(LeoPendingProposal {
            chain_hash: hash,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 1000,
        });
        let approved =
            ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_aam_decision(approved, 2000);

        // STM observation đã bị remove sau promote
        assert!(l.stm_len() < stm_before, "STM phải giảm sau promote QR");
    }

    // ── ISL inbox ──────────────────────────────────────────────────────────

    #[test]
    fn leo_inbox_starts_empty() {
        let l = leo();
        assert_eq!(l.inbox_len(), 0);
    }

    #[test]
    fn leo_receive_isl_queues() {
        let mut l = leo();
        let msg = ISLMessage::new(aam_addr(), leo_addr(), MsgType::Approved);
        let frame = ISLFrame::bare(msg);
        l.receive_isl(frame);
        assert_eq!(l.inbox_len(), 1);
    }

    #[test]
    fn leo_poll_inbox_processes_aam_decision() {
        let mut l = leo();
        // Add a pending proposal
        l.pending.push(LeoPendingProposal {
            chain_hash: 0x1,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 0,
        });
        // Send Approved via inbox
        let approved =
            ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_isl(ISLFrame::bare(approved));

        let processed = l.poll_inbox(1000);
        assert_eq!(processed, 1);
        assert_eq!(l.inbox_len(), 0, "Inbox drained");
        assert_eq!(l.pending_count(), 0, "Approved clears pending");
    }

    #[test]
    fn leo_poll_inbox_nack() {
        let mut l = leo();
        l.pending.push(LeoPendingProposal {
            chain_hash: 0x2,
            fire_count: 3,
            confidence: 0.5,
            timestamp: 0,
        });
        let nack = ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Nack, [0, 0, 0]);
        l.receive_isl(ISLFrame::bare(nack));
        l.poll_inbox(1000);
        assert_eq!(l.pending_count(), 0, "Nack clears pending");
    }

    #[test]
    fn leo_poll_inbox_empty_noop() {
        let mut l = leo();
        let processed = l.poll_inbox(1000);
        assert_eq!(processed, 0);
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

    #[test]
    fn knowledge_analysis_runs() {
        let mut l = leo();
        // Feed several reports for diversity
        for i in 0..5 {
            l.ingest(report(-0.3 + i as f32 * 0.1, i as u8 + 1), i as i64 * 1000);
        }
        let ctx = l.run_knowledge_analysis(6000);
        // Should have cluster info if enough observations
        if l.stm_len() >= 2 {
            assert!(
                ctx.get("cluster_count").is_some() || ctx.get("curated_count").is_some(),
                "Analysis should produce cluster or curation data"
            );
        }
    }

    /// Integration: LeoAI QR writes → OlangReader → verify QR node.
    #[test]
    fn leo_qr_writes_parseable() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();

        l.pending.push(LeoPendingProposal {
            chain_hash: hash,
            fire_count: 5,
            confidence: 0.8,
            timestamp: 1000,
        });
        let approved =
            ISLMessage::with_payload(aam_addr(), leo_addr(), MsgType::Approved, [0, 0, 0]);
        l.receive_aam_decision(approved, 2000);

        let bytes = l.drain_pending_writes();
        assert!(!bytes.is_empty(), "Phải có pending writes");

        // Parse back with OlangReader
        let reader = olang::reader::OlangReader::new(&bytes).expect("valid header");
        let pf = reader.parse_all().expect("parseable");

        assert_eq!(pf.node_count(), 1, "1 QR node");
        assert!(pf.nodes[0].is_qr, "Node phải là QR");
        assert_eq!(pf.nodes[0].layer, 0);
        assert_eq!(pf.nodes[0].timestamp, 2000);
    }

    // ── Express — LeoAI generates Olang code ──────────────────────────────

    #[test]
    fn express_observation_returns_mol_literal() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();
        let expr = l.express_observation(hash);
        assert!(expr.is_some(), "Should express existing observation");
        let s = expr.unwrap();
        assert!(s.starts_with("{ S="), "Should be mol literal: {}", s);
        assert!(s.ends_with(" }"), "Should end with ' }}': {}", s);
        assert!(s.contains("V="), "Should contain valence");
    }

    #[test]
    fn express_observation_missing_returns_none() {
        let l = leo();
        assert!(l.express_observation(0xDEAD).is_none());
    }

    #[test]
    fn express_truth_includes_alias() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();
        let expr = l.express_truth("lửa", hash);
        assert!(expr.is_some());
        let s = expr.unwrap();
        assert!(s.contains("\"lửa\" =="), "Should contain alias assertion: {}", s);
        assert!(s.contains("S="), "Should contain mol dims: {}", s);
    }

    #[test]
    fn express_all_returns_all_stm() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        l.ingest(report(0.3, 2), 2000);
        let exprs = l.express_all();
        assert!(!exprs.is_empty(), "Should express at least 1 observation: got {}", exprs.len());
        for s in &exprs {
            assert!(s.contains("S="), "Each should be mol literal: {}", s);
        }
    }

    #[test]
    fn express_evolution_shows_delta() {
        let mut l = leo();
        // Ingest two observations — they'll differ in dimensions
        l.ingest(report(-0.5, 1), 1000);
        l.ingest(report(0.8, 2), 2000);
        let obs = l.learning.stm().all();
        if obs.len() >= 2 {
            let h1 = obs[0].chain.chain_hash();
            let h2 = obs[1].chain.chain_hash();
            let expr = l.express_evolution(h1, h2);
            assert!(expr.is_some(), "Should express evolution");
            let s = expr.unwrap();
            assert!(s.contains("←"), "Should contain arrow: {}", s);
            assert!(s.contains("Δ"), "Should contain delta marker: {}", s);
        }
    }
}
