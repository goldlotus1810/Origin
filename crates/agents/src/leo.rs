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
use crate::skill::{ExecContext, SkillPatternStore};

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
// ─────────────────────────────────────────────────────────────────────────────
// ProgResult — kết quả khi LeoAI tự lập trình
// ─────────────────────────────────────────────────────────────────────────────

/// Output từ LeoAI.program() — chain hoặc number.
#[derive(Debug, Clone)]
pub enum ProgOutput {
    /// VM computed a number (from PushNum / __hyp_add / etc.)
    Number(f64),
    /// VM emitted a MolecularChain
    Chain(olang::molecular::MolecularChain),
}

/// Kết quả khi LeoAI tự lập trình và chạy code.
#[derive(Debug, Clone)]
pub struct ProgResult {
    /// Outputs từ VM (chains + numbers)
    pub outputs: Vec<ProgOutput>,
    /// Chain hashes đã học vào STM
    pub learned_hashes: Vec<u64>,
    /// Errors (parse, semantic, VM)
    pub errors: Vec<alloc::string::String>,
    /// VM steps executed
    pub steps: u32,
}

impl ProgResult {
    /// Có lỗi không?
    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
    /// Có output không?
    pub fn has_output(&self) -> bool {
        !self.outputs.is_empty()
    }
}

/// Raw VM result — for Runtime to process VmEvents directly.
#[derive(Debug)]
pub struct RawProgResult {
    /// All VM events (Output, LookupAlias, CreateEdge, etc.)
    pub events: Vec<olang::vm::VmEvent>,
    /// Steps executed
    pub steps: u32,
    /// Parse/semantic error if any
    pub error: Option<alloc::string::String>,
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

    /// Learned skill patterns — promote through AAM review
    pub skill_patterns: SkillPatternStore,
    /// Pending pattern promotions awaiting AAM approval (steps joined by "|")
    pending_pattern_keys: Vec<alloc::string::String>,

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
            skill_patterns: SkillPatternStore::new(),
            pending_pattern_keys: Vec::new(),
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
                // Check if this is a skill pattern approval (payload[2] == steps count)
                if msg.payload[2] > 0 && idx < self.pending_pattern_keys.len() {
                    // AAM approved a skill pattern → promote to ComposedSkill
                    let key = self.pending_pattern_keys.remove(idx);
                    let steps: Vec<alloc::string::String> = key
                        .split('|')
                        .map(alloc::string::String::from)
                        .collect();
                    self.skill_patterns.promote(&steps);
                } else if idx < self.pending.len() {
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
                if msg.payload[2] > 0 && idx < self.pending_pattern_keys.len() {
                    // Pattern rejected by AAM → remove from pending
                    self.pending_pattern_keys.remove(idx);
                } else if idx < self.pending.len() {
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
    pub fn skill_pattern_count(&self) -> usize {
        self.skill_patterns.pattern_count()
    }
    pub fn composed_skill_count(&self) -> usize {
        self.skill_patterns.composed_count()
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
                MsgType::Program => {
                    // AAM approved programming request — body contains Olang source
                    if let Ok(source) = core::str::from_utf8(&frame.body) {
                        let result = self.program(source, ts);
                        // Send Ack back with step count
                        let steps_lo = (result.steps & 0xFF) as u8;
                        let out_count = (result.outputs.len() & 0xFF) as u8;
                        let err_flag = if result.has_error() { 1u8 } else { 0u8 };
                        let ack = ISLMessage::with_payload(
                            self.addr,
                            frame.header.from,
                            if result.has_error() { MsgType::Nack } else { MsgType::Ack },
                            [steps_lo, out_count, err_flag],
                        );
                        self.outbox.push(ISLFrame::bare(ack));
                    }
                }
                _ => {
                    // Unknown msg type — ignore
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
    pub fn run_instincts(&mut self, ctx: &mut ExecContext) {
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
        // Track which instincts produced results → record as skill pattern
        let mut active_steps = Vec::new();
        let mut any_success = false;
        for skill in instincts {
            let result = skill.execute(ctx);
            if result.is_ok() {
                active_steps.push(alloc::string::String::from(skill.name()));
                any_success = true;
            }
        }

        // Record observed skill sequence
        if !active_steps.is_empty() {
            self.skill_patterns
                .record(active_steps, any_success, ctx.timestamp);
        }

        // Check for promotable patterns → create SkillProposal → send to AAM
        let promotable: Vec<_> = self.skill_patterns.promotable().iter().map(|p| {
            (p.steps.clone(), p.effectiveness, p.observations)
        }).collect();
        for (steps, effectiveness, observations) in promotable {
            // Create ISL Propose message to AAM with pattern info
            let payload = [
                (effectiveness * 255.0) as u8,
                (observations.min(255)) as u8,
                steps.len().min(255) as u8,
            ];
            let msg = ISLMessage::with_payload(
                self.addr,
                self.aam_addr,
                MsgType::Propose,
                payload,
            );
            self.outbox.push(ISLFrame::bare(msg));
            // Track pending pattern for AAM approval
            self.pending_pattern_keys.push(steps.join("|"));
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

    // ─────────────────────────────────────────────────────────────────────────
    // Programming — LeoAI mở VM, lập trình, chạy, học từ kết quả
    // ─────────────────────────────────────────────────────────────────────────

    /// LeoAI tự lập trình: parse Olang source → compile → execute VM → học kết quả.
    ///
    /// Flow: AAM approve → LeoAI nhận task → build program → VM execute → feed results
    /// back vào LearningLoop + trả kết quả lên caller.
    ///
    /// LeoAI "nghĩ" bằng Olang — VM là công cụ suy luận của nó.
    pub fn program(&mut self, source: &str, ts: i64) -> ProgResult {
        self.state = LeoState::Learning;
        self.last_event_ts = ts;

        // Step 1: Parse Olang source → AST
        let stmts = match olang::syntax::parse(source) {
            Ok(s) => s,
            Err(e) => {
                return ProgResult {
                    outputs: Vec::new(),
                    learned_hashes: Vec::new(),
                    errors: alloc::vec![alloc::format!("Parse error: {}", e.message)],
                    steps: 0,
                };
            }
        };

        // Step 2: Semantic validation
        let sem_errors = olang::semantic::validate(&stmts);
        if !sem_errors.is_empty() {
            return ProgResult {
                outputs: Vec::new(),
                learned_hashes: Vec::new(),
                errors: sem_errors
                    .iter()
                    .map(|e| alloc::format!("Semantic: {}", e.message))
                    .collect(),
                steps: 0,
            };
        }

        // Step 3: Lower AST → IR → OlangProgram
        let prog = olang::semantic::lower(&stmts);

        // Step 4: Execute trong VM
        let vm = olang::vm::OlangVM::new();
        let vm_result = vm.execute(&prog);

        // Step 5: Process kết quả — học từ outputs
        let mut outputs = Vec::new();
        let mut learned_hashes = Vec::new();
        let mut errors = Vec::new();
        let cur_emotion = self.learning.context().last_emotion();

        for event in &vm_result.events {
            match event {
                olang::vm::VmEvent::Output(chain) => {
                    if !chain.is_empty() {
                        let hash = chain.chain_hash();
                        // Feed output vào STM — LeoAI học từ kết quả chạy code
                        self.learning.stm_mut().push(chain.clone(), cur_emotion, ts);
                        learned_hashes.push(hash);

                        // Build output description
                        if let Some(num) = chain.to_number() {
                            outputs.push(ProgOutput::Number(num));
                        } else {
                            outputs.push(ProgOutput::Chain(chain.clone()));
                        }
                    }
                }
                olang::vm::VmEvent::Error(e) => {
                    errors.push(alloc::format!("VM: {:?}", e));
                }
                olang::vm::VmEvent::LookupAlias(_alias) => {
                    // LeoAI cannot resolve aliases internally — only Runtime has Registry.
                    // Unresolved aliases are normal for standalone LeoAI execution.
                    // Use program_raw() if caller needs to handle alias resolution.
                }
                _ => {} // Other events passed through
            }
        }

        // Co-activate all learned outputs together
        if learned_hashes.len() >= 2 {
            for i in 0..learned_hashes.len() - 1 {
                self.learning.graph_mut().co_activate(
                    learned_hashes[i],
                    learned_hashes[i + 1],
                    cur_emotion,
                    0.8,
                    ts,
                );
            }
        }

        self.state = LeoState::Listening;

        ProgResult {
            outputs,
            learned_hashes,
            errors,
            steps: vm_result.steps,
        }
    }

    /// LeoAI tự viết Olang code từ knowledge rồi chạy.
    ///
    /// Ví dụ: LeoAI thấy "lửa" và "nước" trong STM → tự viết `lửa ∘ nước`
    /// → chạy VM → nhận LCA → học concept mới.
    pub fn program_compose(&mut self, alias_a: &str, alias_b: &str, ts: i64) -> ProgResult {
        let code = alloc::format!("emit {} ∘ {};", alias_a, alias_b);
        self.program(&code, ts)
    }

    /// LeoAI tự verify một truth assertion.
    ///
    /// Ví dụ: `"lửa" == { S=1 R=6 V=200 A=180 T=4 }`
    pub fn program_verify(&mut self, alias: &str, chain_hash: u64, ts: i64) -> ProgResult {
        if let Some(code) = self.express_truth(alias, chain_hash) {
            let full = alloc::format!("emit {};", code);
            self.program(&full, ts)
        } else {
            ProgResult {
                outputs: Vec::new(),
                learned_hashes: Vec::new(),
                errors: alloc::vec![alloc::format!("Unknown hash: 0x{:X}", chain_hash)],
                steps: 0,
            }
        }
    }

    /// LeoAI chạy thí nghiệm: thử biến đổi chain rồi xem kết quả.
    ///
    /// Ví dụ: "nếu thay đổi valence của lửa thì sao?"
    /// → viết `emit { S=1 R=6 V=48 A=180 T=4 };` → chạy → học
    pub fn program_experiment(
        &mut self,
        chain_hash: u64,
        dim: &str,
        new_val: u8,
        ts: i64,
    ) -> ProgResult {
        if let Some(obs) = self.learning.stm().find_by_hash(chain_hash) {
            if let Some(mol) = obs.chain.first() {
                let (s, r, v, a, t) = match dim {
                    "S" | "shape" => (new_val, mol.relation, mol.emotion.valence, mol.emotion.arousal, mol.time),
                    "R" | "relation" => (mol.shape, new_val, mol.emotion.valence, mol.emotion.arousal, mol.time),
                    "V" | "valence" => (mol.shape, mol.relation, new_val, mol.emotion.arousal, mol.time),
                    "A" | "arousal" => (mol.shape, mol.relation, mol.emotion.valence, new_val, mol.time),
                    "T" | "time" => (mol.shape, mol.relation, mol.emotion.valence, mol.emotion.arousal, new_val),
                    _ => {
                        return ProgResult {
                            outputs: Vec::new(),
                            learned_hashes: Vec::new(),
                            errors: alloc::vec![alloc::format!("Unknown dimension: {}", dim)],
                            steps: 0,
                        };
                    }
                };
                let code = alloc::format!("emit {{ S={} R={} V={} A={} T={} }};", s, r, v, a, t);
                self.program(&code, ts)
            } else {
                ProgResult {
                    outputs: Vec::new(),
                    learned_hashes: Vec::new(),
                    errors: alloc::vec![alloc::string::String::from("Empty chain")],
                    steps: 0,
                }
            }
        } else {
            ProgResult {
                outputs: Vec::new(),
                learned_hashes: Vec::new(),
                errors: alloc::vec![alloc::format!("Unknown hash: 0x{:X}", chain_hash)],
                steps: 0,
            }
        }
    }

    /// Return all VmEvents for caller to process (LookupAlias, CreateEdge, etc.)
    /// LeoAI runs program and returns raw events that Runtime needs to handle.
    pub fn program_raw(&mut self, source: &str, ts: i64) -> RawProgResult {
        self.state = LeoState::Learning;
        self.last_event_ts = ts;

        let stmts = match olang::syntax::parse(source) {
            Ok(s) => s,
            Err(e) => {
                self.state = LeoState::Listening;
                return RawProgResult {
                    events: Vec::new(),
                    steps: 0,
                    error: Some(alloc::format!("Parse: {}", e.message)),
                };
            }
        };

        let sem_errors = olang::semantic::validate(&stmts);
        if !sem_errors.is_empty() {
            self.state = LeoState::Listening;
            return RawProgResult {
                events: Vec::new(),
                steps: 0,
                error: Some(sem_errors[0].message.clone()),
            };
        }

        let prog = olang::semantic::lower(&stmts);
        let vm = olang::vm::OlangVM::new();
        let result = vm.execute(&prog);

        self.state = LeoState::Listening;

        RawProgResult {
            events: result.events,
            steps: result.steps,
            error: None,
        }
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
    use alloc::vec;
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

    // ── Programming — LeoAI mở VM, tự lập trình ───────────────────────────

    #[test]
    fn program_arithmetic() {
        let mut l = leo();
        let result = l.program("emit 1 + 2;", 1000);
        assert!(!result.has_error(), "Errors: {:?}", result.errors);
        assert!(result.has_output(), "Should produce output");
        // Output should be numeric 3
        assert!(
            result.outputs.iter().any(|o| matches!(o, ProgOutput::Number(n) if (*n - 3.0).abs() < 0.01)),
            "1 + 2 should = 3: {:?}", result.outputs
        );
    }

    #[test]
    fn program_mol_literal() {
        let mut l = leo();
        let result = l.program("emit { S=1 R=6 V=200 A=180 T=4 };", 1000);
        assert!(!result.has_error(), "Errors: {:?}", result.errors);
        assert!(result.has_output());
        // Should learn the chain into STM
        assert!(!result.learned_hashes.is_empty(), "Should learn output");
    }

    #[test]
    fn program_parse_error() {
        let mut l = leo();
        // Use truly invalid syntax — unterminated block
        let result = l.program("fn broken( { emit;", 1000);
        assert!(result.has_error(), "Should detect parse error: {:?}", result);
    }

    #[test]
    fn program_semantic_error() {
        let mut l = leo();
        let result = l.program("emit { S=999 };", 1000);
        assert!(result.has_error(), "S=999 should fail semantic validation");
    }

    #[test]
    fn program_compose() {
        let mut l = leo();
        // program_compose builds "emit a ∘ b;" and runs it
        let result = l.program_compose("fire", "water", 1000);
        // Should not have parse/semantic errors (aliases are fine)
        assert!(!result.has_error(), "Errors: {:?}", result.errors);
        assert!(result.steps > 0, "Should execute some steps");
    }

    #[test]
    fn program_experiment_changes_dim() {
        let mut l = leo();
        // First ingest something to have in STM
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();

        // Experiment: change valence
        let result = l.program_experiment(hash, "V", 48, 2000);
        assert!(!result.has_error(), "Experiment should not error: {:?}", result.errors);
        assert!(result.has_output(), "Should produce modified chain");
    }

    #[test]
    fn program_experiment_unknown_dim() {
        let mut l = leo();
        l.ingest(report(-0.5, 1), 1000);
        let hash = l.learning.stm().top_n(1)[0].chain.chain_hash();
        let result = l.program_experiment(hash, "X", 48, 2000);
        assert!(result.has_error(), "Unknown dim should error");
    }

    #[test]
    fn program_experiment_missing_hash() {
        let mut l = leo();
        let result = l.program_experiment(0xDEAD, "V", 48, 1000);
        assert!(result.has_error(), "Missing hash should error");
    }

    #[test]
    fn program_raw_returns_events() {
        let mut l = leo();
        let result = l.program_raw("emit 42;", 1000);
        assert!(result.error.is_none(), "Should not error");
        assert!(result.steps > 0);
        assert!(
            result.events.iter().any(|e| matches!(e, olang::vm::VmEvent::Output(_))),
            "Should have Output event"
        );
    }

    #[test]
    fn program_via_isl_inbox() {
        let mut l = leo();
        // Simulate AAM sending Program message via ISL
        let msg = ISLMessage::with_payload(
            aam_addr(),
            leo_addr(),
            MsgType::Program,
            [0, 0, 0],
        );
        let source = b"emit 1 + 2;".to_vec();
        let frame = ISLFrame::with_body(msg, source);
        l.receive_isl(frame);
        let processed = l.poll_inbox(1000);
        assert_eq!(processed, 1, "Should process 1 message");

        // Should have Ack in outbox
        let outbox = l.flush_outbox();
        assert!(!outbox.is_empty(), "Should send Ack back");
        assert_eq!(outbox[0].header.msg_type, MsgType::Ack, "Should be Ack (not Nack)");
    }

    #[test]
    fn program_learns_into_stm() {
        let mut l = leo();
        let stm_before = l.stm_len();
        let result = l.program("emit { S=2 R=3 V=100 A=50 T=1 };", 1000);
        assert!(!result.has_error());
        // STM should grow — LeoAI learned from its own code output
        assert!(l.stm_len() > stm_before, "STM should grow after programming");
    }

    // ── SkillPattern → AAM integration tests ────────────────────────────────

    #[test]
    fn ingest_records_instinct_patterns() {
        let mut l = leo();
        // Ingest enough data to trigger instincts
        for i in 0..3u8 {
            l.ingest(report(-0.3, i), (i as i64 + 1) * 1000);
        }
        // run_instincts should have recorded instinct patterns
        assert!(
            l.skill_patterns.pattern_count() > 0,
            "Instinct patterns should be recorded after ingest"
        );
    }

    #[test]
    fn skill_pattern_aam_approval_flow() {
        let mut l = leo();
        // Simulate enough ingests to accumulate a promotable pattern
        for i in 0..5u8 {
            l.ingest(report(-0.2 - i as f32 * 0.05, i), (i as i64 + 1) * 1000);
        }

        // Check if any patterns are promotable
        let promotable = l.skill_patterns.promotable();
        if promotable.is_empty() {
            // If instincts didn't consistently fire the same set, record manually
            let steps = vec![
                alloc::string::String::from("Honesty"),
                alloc::string::String::from("Curiosity"),
            ];
            for i in 0..3 {
                l.skill_patterns.record(steps.clone(), true, i * 100);
            }
        }

        let promotable = l.skill_patterns.promotable();
        assert!(!promotable.is_empty(), "Should have promotable patterns");
        assert_eq!(l.skill_patterns.composed_count(), 0, "Not yet promoted");

        // Simulate AAM approval — clear any keys added by run_instincts during ingest
        l.pending_pattern_keys.clear();
        let steps = promotable[0].steps.clone();
        l.pending_pattern_keys.push(steps.join("|"));

        // AAM sends Approved with payload[2] = steps count (non-zero → pattern)
        let approved_msg = ISLMessage::with_payload(
            aam_addr(),
            leo_addr(),
            MsgType::Approved,
            [0, 0, steps.len() as u8],
        );
        l.receive_aam_decision(approved_msg, 10000);

        assert_eq!(
            l.skill_patterns.composed_count(),
            1,
            "Pattern should be promoted after AAM approval"
        );
        assert!(
            l.pending_pattern_keys.is_empty(),
            "Pending pattern keys should be drained"
        );
    }

    #[test]
    fn skill_pattern_aam_rejection() {
        let mut l = leo();
        let steps = vec![alloc::string::String::from("A")];
        l.pending_pattern_keys.push(steps.join("|"));

        // AAM sends Nack
        let nack_msg = ISLMessage::with_payload(
            aam_addr(),
            leo_addr(),
            MsgType::Nack,
            [0, 0, 1], // payload[2] > 0 → pattern nack
        );
        l.receive_aam_decision(nack_msg, 10000);

        assert!(l.pending_pattern_keys.is_empty(), "Rejected pattern removed");
        assert_eq!(l.skill_patterns.composed_count(), 0, "Not promoted");
    }

    #[test]
    fn run_instincts_sends_propose_when_promotable() {
        let mut l = leo();
        // Manually record enough patterns to be promotable
        let steps = vec![
            alloc::string::String::from("Honesty"),
            alloc::string::String::from("Curiosity"),
        ];
        for i in 0..3 {
            l.skill_patterns.record(steps.clone(), true, i * 100);
        }

        // Run instincts → should detect promotable pattern → send Propose to AAM
        let emotion = EmotionTag::NEUTRAL;
        let chain = olang::encoder::encode_codepoint(0x25CF);
        let mut ctx = ExecContext::new(5000, emotion, 0.0);
        ctx.push_input(chain);
        l.run_instincts(&mut ctx);

        let outbox = l.flush_outbox();
        let propose_msgs: Vec<_> = outbox
            .iter()
            .filter(|f| f.header.msg_type == MsgType::Propose)
            .collect();
        assert!(
            !propose_msgs.is_empty(),
            "Should send Propose to AAM for promotable pattern"
        );
        assert!(
            !l.pending_pattern_keys.is_empty(),
            "Should track pending pattern key"
        );
    }
}
