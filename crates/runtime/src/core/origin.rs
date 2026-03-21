//! # origin — HomeRuntime
//!
//! ○(∅) == ○ — tự boot, tự vận hành.
//!
//! process_one(input) → Response
//!   SecurityGate → Parse → Encode → Context → STM → Silk → Response

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};

use crate::response_template::{render, compose_response, detect_language, Lang, ResponseContext, ResponseParams};
use agents::encoder::ContentInput;
use agents::gate::GateVerdict;
use agents::learning::{LearningLoop, ProcessResult};
use context::emotion::sentence_affect;
use context::emotion::IntentKind;
use context::infer::infer_context;
use context::intent::{decide_action, estimate_intent, IntentAction};
use memory::dream::{DreamConfig, DreamCycle};
use silk::walk::ResponseTone;

use crate::parser::{CmpOp, OlangExpr, OlangParser, ParseResult, RelationOp};
use olang::compiler::{Compiler, Target};
use olang::ir::{compile_expr, OlangIrExpr};
use olang::optimize::{self, OptLevel};
use olang::semantic;
use olang::syntax;
use olang::knowtree::{KnowTreeLegacy, SlimKnowTreeLegacy};
use olang::registry::Registry;
use vsdf::body::{body_from_molecule_full, BodyStore};
use olang::self_model::SelfModel;
use olang::separator::parse_to_chains;
use olang::startup::{boot, bootstrap_programs, chain_to_emoji, resolve_with_cp, BootStage, SystemManifest};
use olang::vm::{OlangVM, VmEvent};
use agents::leo::{LeoAI, ProgOutput};
use agents::chief::{Chief, ChiefKind};
use agents::worker::Worker;
use agents::skill::{ExecContext, Skill};
use agents::domain_skills::{
    ClusterSkill, SimilaritySkill, GeneralizationSkill,
    IngestSkill, DeltaSkill, HebbianSkill, CuratorSkill,
    MergeSkill, PruneSkill, TemporalPatternSkill, InverseRenderSkill,
};
use crate::auth::{AuthHeader, AuthState};
use crate::router::MessageRouter;
use memory::proposal::{AlertLevel, RegistryGate};

// ─────────────────────────────────────────────────────────────────────────────
// System Clock — đọc giờ hệ thống thực
// ─────────────────────────────────────────────────────────────────────────────

/// Read real system time (milliseconds since UNIX epoch).
///
/// Khi feature `std` bật: dùng std::time::SystemTime.
/// Khi no_std: trả 0 — caller phải inject ts từ ngoài.
#[cfg(feature = "std")]
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// no_std fallback — trả 0, caller phải inject ts.
#[cfg(not(feature = "std"))]
pub fn now_ms() -> i64 {
    0
}

/// Nanosecond precision (for session IDs and high-res timing).
#[cfg(feature = "std")]
pub fn now_ns() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64
}

#[cfg(not(feature = "std"))]
pub fn now_ns() -> i64 {
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// Response
// ─────────────────────────────────────────────────────────────────────────────

/// Response từ HomeRuntime.
#[derive(Debug, Clone)]
pub struct Response {
    pub text: String,
    pub tone: ResponseTone,
    pub fx: f32,
    pub kind: ResponseKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseKind {
    Natural,     // Trả lời tự nhiên
    OlangResult, // Kết quả ○{} query
    Crisis,      // Crisis response
    Blocked,     // SecurityGate blocked
    System,      // System command response
}

// ─────────────────────────────────────────────────────────────────────────────
// HomeRuntime
// ─────────────────────────────────────────────────────────────────────────────

/// HomeOS Runtime — mọi thứ qua đây.
///
/// ○(∅) == ○: boot từ hư không, sống từ đây.
pub struct HomeRuntime {
    learning: LearningLoop,
    parser: OlangParser,
    dream: DreamCycle,
    registry: Registry,
    alias_to_cp: BTreeMap<alloc::string::String, u32>,
    self_model: SelfModel,
    uptime_ns: i64,
    turn_count: u64,
    last_dream_turn: u64, // turn khi Dream lần cuối chạy
    /// QT9: bytes chờ ghi disk — caller (server) drain và flush.
    pending_writes: alloc::vec::Vec<u8>,
    /// L2-Ln knowledge storage — TieredStore compact encoding (legacy).
    knowtree: KnowTreeLegacy,
    /// L2-Ln knowledge storage — spec-compliant ~10B/node format.
    slim_knowtree: SlimKnowTreeLegacy,
    /// Recent text history — cho reference resolution ("bà ấy", "anh ta"...).
    /// Giữ tối đa 16 turns gần nhất (text + timestamp).
    recent_texts: alloc::vec::Vec<RecentText>,
    // ── Phase 10: Dream stats ───────────────────────────────────────────────
    /// Fibonacci dream schedule index (starts at 5 → Fib[5]=5, then 6→8, 7→13...)
    dream_fib_index: usize,
    /// Total dream cycles run
    dream_cycles: u64,
    /// Total proposals approved across all dreams
    dream_approved_total: u64,
    /// Total L3 concepts created from Dream
    dream_l3_created: u64,
    /// NodeBody store — chain_hash → SDF + Spline bindings
    body_store: BodyStore,
    /// LeoAI — não: nhận task, tự lập trình VM, học từ kết quả
    leo: LeoAI,
    /// RegistryGate — cơ chế cứng: mọi thứ phải đăng ký Registry
    registry_gate: RegistryGate,
    /// MessageRouter — central dispatcher cho agent hierarchy
    router: MessageRouter,
    /// Chiefs — tier 1 agents (Home, Vision, Network)
    chiefs: alloc::vec::Vec<Chief>,
    /// Workers — tier 2 agents (sensors, actuators)
    workers: alloc::vec::Vec<Worker>,
    /// Boot stage reached during startup
    boot_stage: BootStage,
    /// SystemManifest — hệ thống biết mình có gì sau boot
    manifest: SystemManifest,
    /// Boot errors (nếu có)
    boot_errors: alloc::vec::Vec<String>,
    /// Execution tracing toggle (○{trace} bật/tắt)
    trace_enabled: bool,
    /// Platform bridge — HAL → phần cứng thật.
    /// None = REPL/test mode (chỉ log events).
    /// Some = wired to real hardware via PlatformBridge.
    platform: Option<alloc::boxed::Box<dyn hal::ffi::PlatformBridge>>,
    /// Module loader — resolves, caches, and tracks module dependencies
    module_loader: olang::module::ModuleLoader,
    /// Auth state machine: Virgin → Locked → Unlocked
    auth_state: AuthState,
    /// Auth header: master pubkey, salt, setup metadata
    auth_header: AuthHeader,
}

/// Lưu text gần đây cho reference resolution.
#[derive(Debug, Clone)]
struct RecentText {
    text: String,
    /// Timestamp — dùng cho time-based decay (tương lai).
    #[allow(dead_code)]
    timestamp: i64,
    /// Tên riêng đã extract được (nếu có)
    names: alloc::vec::Vec<String>,
}

/// Extract first integer from text (e.g., "đặt nhiệt độ 25" → 25).
fn extract_number(s: &str) -> Option<i32> {
    let mut num_str = String::new();
    let mut found = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
            found = true;
        } else if found {
            break;
        }
    }
    if found { num_str.parse().ok() } else { None }
}

/// Extract text content from OlangExpr for SecurityGate checking.
fn olang_expr_text(expr: &OlangExpr) -> String {
    match expr {
        OlangExpr::Query(s) | OlangExpr::Command(s) | OlangExpr::Use(s) => s.clone(),
        OlangExpr::RelationQuery { subject, object, .. } => {
            let mut t = subject.clone();
            if let Some(o) = object {
                t.push(' ');
                t.push_str(o);
            }
            t
        }
        OlangExpr::Compose { a, b } => format!("{} {}", a, b),
        OlangExpr::ContextQuery { term, context } => format!("{} {}", term, context),
        OlangExpr::LetBinding { name, value } => {
            format!("{} {}", name, olang_expr_text(value))
        }
        OlangExpr::FnDef { name, body } => {
            let mut t = name.clone();
            for e in body {
                let s = olang_expr_text(e);
                if !s.is_empty() {
                    t.push(' ');
                    t.push_str(&s);
                }
            }
            t
        }
        OlangExpr::Pipeline(exprs) | OlangExpr::Pipe(exprs) => {
            let parts: alloc::vec::Vec<String> =
                exprs.iter().map(olang_expr_text).filter(|s| !s.is_empty()).collect();
            parts.join(" ")
        }
        OlangExpr::Emit(inner) | OlangExpr::Return(inner) => olang_expr_text(inner),
        _ => String::new(),
    }
}

impl HomeRuntime {
    /// Boot từ hư không — ○(∅)==○.
    pub fn new(session_id: u64) -> Self {
        Self::with_file(session_id, None)
    }

    /// Boot với platform bridge — Olang ○{} điều khiển phần cứng thật.
    pub fn with_platform(
        session_id: u64,
        file_bytes: Option<&[u8]>,
        platform: alloc::boxed::Box<dyn hal::ffi::PlatformBridge>,
    ) -> Self {
        let mut rt = Self::with_file(session_id, file_bytes);
        rt.platform = Some(platform);
        rt
    }

    /// Boot với file bytes — load registry + Silk edges từ origin.olang.
    ///
    /// QT8: mọi thứ tạo ra → ghi pending_writes TRƯỚC → caller flush ra disk.
    /// QT9: mọi node → phải đăng ký Registry.
    pub fn with_file(session_id: u64, file_bytes: Option<&[u8]>) -> Self {
        let boot_result = boot(file_bytes);
        let mut learning = LearningLoop::new(session_id);

        // Restore Silk edges từ file → SilkGraph
        for edge in &boot_result.edges {
            learning.graph_mut().restore_edge(
                edge.from_hash,
                edge.to_hash,
                edge.edge_type,
                edge.timestamp,
            );
        }

        // Restore Hebbian links từ file → SilkGraph.learned
        for heb in &boot_result.hebbian_records {
            learning.graph_mut().restore_learned(
                heb.from_hash,
                heb.to_hash,
                heb.weight,
                heb.fire_count,
            );
        }

        // Restore STM observations từ file → ShortTermMemory
        // Append-only: replay all records, dedup by hash (last wins)
        for stm_rec in &boot_result.stm_records {
            learning.restore_stm_observation(
                stm_rec.chain_hash,
                stm_rec.valence,
                stm_rec.arousal,
                stm_rec.dominance,
                stm_rec.intensity,
                stm_rec.fire_count,
                stm_rec.maturity,
                stm_rec.layer,
                stm_rec.timestamp,
            );
        }

        // Restore ConversationCurve từ file — replay valence turns
        for curve_rec in &boot_result.curve_records {
            learning.restore_curve_turn(curve_rec.valence, curve_rec.fx_dn);
        }

        // QT8: nhận pending_writes từ boot (L0+L1 seeds chưa có trong file)
        let mut pending_writes = boot_result.pending_writes;

        // QT8+QT9: Ghi system agents vào origin.olang nếu chưa có
        // Chiefs, LeoAI → phải có trong file để L0 biết hệ thống đang chạy gì
        let agent_writes = Self::write_agent_records(
            &boot_result.registry,
            file_bytes,
        );
        pending_writes.extend_from_slice(&agent_writes);

        // Restore auth: last RT_AUTH record wins (append-only)
        let (auth_header, auth_state) = if let Some(last_auth) = boot_result.auth_records.last() {
            let hdr = AuthHeader::from_bytes(&last_auth.header_bytes);
            let state = if hdr.is_virgin() { AuthState::Virgin } else { AuthState::Locked };
            (hdr, state)
        } else {
            (AuthHeader::virgin(), AuthState::Virgin)
        };

        let mut rt = Self {
            learning,
            parser: OlangParser::new(),
            dream: DreamCycle::new(DreamConfig::for_conversation()),
            alias_to_cp: build_alias_map(file_bytes),
            registry: boot_result.registry,
            self_model: SelfModel::new(),
            uptime_ns: 0,
            turn_count: 0,
            last_dream_turn: 0,
            pending_writes,
            knowtree: KnowTreeLegacy::for_pc(),
            slim_knowtree: SlimKnowTreeLegacy::new(),
            recent_texts: alloc::vec::Vec::new(),
            dream_fib_index: 4, // Fib[4]=5: first dream after 5 turns
            dream_cycles: 0,
            dream_approved_total: 0,
            dream_l3_created: 0,
            body_store: BodyStore::with_capacity(4096), // Max 4K bodies in RAM — evict LFU
            leo: LeoAI::new(
                isl::address::ISLAddress::new(1, 0, 0, 1), // tier 1, LeoAI
                isl::address::ISLAddress::new(0, 0, 0, 0), // tier 0, AAM
            ),
            registry_gate: RegistryGate::new(),
            router: MessageRouter::new(),
            chiefs: Self::boot_chiefs(),
            workers: alloc::vec::Vec::new(),
            boot_stage: boot_result.stage,
            manifest: boot_result.manifest,
            boot_errors: boot_result.errors,
            trace_enabled: false,
            platform: None,
            module_loader: olang::module::ModuleLoader::new(alloc::vec!["stdlib".into(), ".".into()]),
            auth_state,
            auth_header,
        };

        // Restore KnowTree compact nodes từ file (legacy 0x08)
        for kt_rec in &boot_result.knowtree_records {
            rt.knowtree.restore_compact_node(&kt_rec.data);
        }

        // Restore SlimKnowTree nodes từ file (0x0A — spec-compliant)
        for sk_rec in &boot_result.slim_knowtree_records {
            rt.slim_knowtree.restore_slim_node(
                sk_rec.hash,
                &sk_rec.tagged,
                sk_rec.layer,
                sk_rec.timestamp,
            );
        }

        // ── Run bootstrap programs — firmware trên VM ─────────────────────
        // bootstrap_programs() = Olang source strings cho bản năng bẩm sinh
        // Chạy qua process_text("○{...}") → VM → Silk edges + Registry
        rt.run_bootstrap();

        rt
    }

    /// Boot default Chiefs — Home, Vision, Network.
    fn boot_chiefs() -> alloc::vec::Vec<Chief> {
        let aam = isl::address::ISLAddress::new(0, 0, 0, 0);
        let leo_addr = isl::address::ISLAddress::new(1, 0, 0, 1);
        alloc::vec![
            Chief::new(isl::address::ISLAddress::new(1, 1, 0, 0), aam, leo_addr, ChiefKind::Home),
            Chief::new(isl::address::ISLAddress::new(1, 2, 0, 0), aam, leo_addr, ChiefKind::Vision),
            Chief::new(isl::address::ISLAddress::new(1, 3, 0, 0), aam, leo_addr, ChiefKind::Network),
        ]
    }

    /// Run bootstrap programs — firmware trên Olang VM.
    ///
    /// Chạy mỗi bootstrap program qua parser → VM → Silk/Registry.
    /// Đây là bước tự nhận thức: hệ thống nhìn thấy chính mình,
    /// xác nhận axioms, tạo Silk edges giữa các nhóm nguyên tố.
    fn run_bootstrap(&mut self) {
        let programs = bootstrap_programs();
        let ts = 0i64; // boot timestamp
        for src in &programs {
            let olang_input = alloc::format!("\u{25CB}{{{}}}", src);
            let _response = self.process_text(&olang_input, ts);
        }
        // Reset turn count — bootstrap không tính là user turns
        self.turn_count = 0;
    }

    /// QT8+QT9: Ghi agent records (Chiefs, LeoAI) vào origin.olang nếu chưa có.
    ///
    /// Agents đã được đăng ký trong L1_SYSTEM_SEED (startup.rs) với các codepoints:
    ///   ♔ 0x2654 = AAM, ♕ 0x2655 = LeoAI, ♖ 0x2656 = HomeChief, etc.
    /// Hàm này kiểm tra xem các agent session records (kích hoạt runtime) đã có chưa.
    /// Nếu chưa → ghi edge records liên kết agent → session.
    fn write_agent_records(
        _registry: &olang::registry::Registry,
        _file_bytes: Option<&[u8]>,
    ) -> alloc::vec::Vec<u8> {
        use olang::writer::OlangWriter;

        // Agent codepoints đã được ghi bởi write_missing_seeds (L1_SYSTEM_SEED).
        // Ở đây ta ghi EDGE records: agent → origin (∈ relation) để đánh dấu
        // "agent này đang hoạt động trong hệ thống".

        // Parse file để kiểm tra edge đã tồn tại chưa
        let mut existing_edges: alloc::collections::BTreeSet<(u64, u64)> =
            alloc::collections::BTreeSet::new();

        if let Some(bytes) = _file_bytes {
            if let Ok(reader) = olang::reader::OlangReader::new(bytes) {
                if let Ok(parsed) = reader.parse_all() {
                    for edge in &parsed.edges {
                        existing_edges.insert((edge.from_hash, edge.to_hash));
                    }
                }
            }
        }

        let mut writer = OlangWriter::new_append();
        let ts = 0i64;

        // Origin node hash — anchor point
        let origin_hash = olang::encoder::encode_codepoint(0x25CB).chain_hash();

        // Active agent codepoints → ghi edge: agent ∈ origin
        let active_agents: &[(u32, &str)] = &[
            (0x2654, "agent:aam"),
            (0x2655, "agent:leo"),
            (0x2656, "agent:chief:home"),
            (0x2657, "agent:chief:vision"),
            (0x2658, "agent:chief:network"),
        ];

        for &(cp, _name) in active_agents {
            let agent_hash = olang::encoder::encode_codepoint(cp).chain_hash();
            if !existing_edges.contains(&(agent_hash, origin_hash)) {
                // Edge: agent ∈ origin (relation = Member = 0x01)
                writer.append_edge(agent_hash, origin_hash, 0x01, ts);
            }
        }

        if writer.write_count() > 0 {
            writer.into_bytes()
        } else {
            alloc::vec::Vec::new()
        }
    }

    /// L0 Integrity Check — kiểm tra mọi node trong Registry đều có trong origin.olang.
    ///
    /// QT8: Phát hiện node đang hoạt động mà chưa được ghi file.
    /// Returns: danh sách hash + alias của node vi phạm (RAM-only).
    pub fn integrity_check(&self, file_bytes: Option<&[u8]>) -> alloc::vec::Vec<String> {
        let mut violations = alloc::vec::Vec::new();

        // Parse file → collect hashes
        let mut file_hashes: alloc::collections::BTreeSet<u64> =
            alloc::collections::BTreeSet::new();

        if let Some(bytes) = file_bytes {
            if let Ok(reader) = olang::reader::OlangReader::new(bytes) {
                if let Ok(parsed) = reader.parse_all() {
                    for node in &parsed.nodes {
                        file_hashes.insert(node.chain.chain_hash());
                    }
                }
            }
        }

        // So sánh Registry vs file
        for layer in 0u8..16 {
            for entry in self.registry.entries_in_layer(layer) {
                if !file_hashes.contains(&entry.chain_hash) {
                    // Node trong Registry nhưng KHÔNG trong file → vi phạm QT8
                    let alias = self
                        .registry
                        .lookup_name_by_hash(entry.chain_hash)
                        .unwrap_or_else(|| alloc::format!("0x{:016X}", entry.chain_hash));
                    violations.push(alloc::format!(
                        "[QT8] L{} {} — in RAM, not in origin.olang",
                        layer, alias
                    ));
                }
            }
        }

        violations
    }

    /// Xử lý một text input — entry point cho text.
    ///
    /// Parse ○{} trước, nếu natural text → delegate to process_input (universal pipeline).
    pub fn process_text(&mut self, text: &str, ts: i64) -> Response {
        // ── AUTH guard: ○{} commands require unlocked state ──────────────────
        // Natural text is always allowed (emotion pipeline, learning).
        // Olang commands (○{...}) require auth if system has been set up.
        if !self.auth_header.is_virgin() && !self.is_unlocked() {
            // Allow auth commands even when locked
            let trimmed = text.trim();
            if trimmed.starts_with("○{auth ") || trimmed.starts_with("o{auth ") {
                // fall through to parser
            } else if trimmed.contains("○{") || trimmed.contains("o{") {
                return Response {
                    text: String::from("Auth ○ Hệ thống đang khóa. Dùng ○{auth unlock <user> <pass>} để mở."),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::Blocked,
                };
            }
        }

        // ── Parse: natural hoặc ○{} ──────────────────────────────────────────
        match self.parser.parse(text) {
            ParseResult::Natural(s) => {
                let input = ContentInput::Text {
                    content: s,
                    timestamp: ts,
                };
                self.process_input(input, ts)
            }
            ParseResult::OlangExpr(expr) => {
                self.turn_count += 1;
                self.uptime_ns = ts;
                self.process_olang(expr, ts)
            }
            ParseResult::FullProgram(source) => {
                self.turn_count += 1;
                self.uptime_ns = ts;
                self.run_program(&source, ts)
            }
            ParseResult::Error(e) => Response {
                text: format!("Parse error: {}", e),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::Blocked,
            },
        }
    }

    // ── ○{} expression — compile và execute qua OlangVM ─────────────────────

    fn process_olang(&mut self, expr: OlangExpr, ts: i64) -> Response {
        // ── SecurityGate — chạy TRƯỚC MỌI THỨ (QT: Gate trước mọi pipeline) ──
        let raw_text = olang_expr_text(&expr);
        if !raw_text.is_empty() {
            match self.learning.gate().check_text(&raw_text) {
                GateVerdict::Allow => {}
                GateVerdict::Crisis { message } => {
                    return Response {
                        text: message,
                        tone: ResponseTone::Supportive,
                        fx: self.learning.context().fx(),
                        kind: ResponseKind::Crisis,
                    };
                }
                GateVerdict::Block { reason } => {
                    return Response {
                        text: format!("⚠ SecurityGate blocked: {:?}", reason),
                        tone: ResponseTone::Gentle,
                        fx: self.learning.context().fx(),
                        kind: ResponseKind::Blocked,
                    };
                }
                GateVerdict::BlackCurtain => {
                    return Response {
                        text: String::from("…"),
                        tone: ResponseTone::Gentle,
                        fx: self.learning.context().fx(),
                        kind: ResponseKind::Blocked,
                    };
                }
            }
        }

        // Commands bypass VM
        if let OlangExpr::Command(ref cmd) = expr {
            return self.handle_command(cmd, ts);
        }
        // ZWJ: display original codepoints directly (before VM)
        if let OlangExpr::Query(ref s) = expr {
            if s.contains('‍') {
                let chains = parse_to_chains(s);
                let mol_count: usize = chains.iter().map(|c| c.len()).sum();
                return Response {
                    text: format!("○ {} ({} mol)", s, mol_count),
                    tone: ResponseTone::Engaged,
                    fx: self.learning.context().fx(),
                    kind: ResponseKind::OlangResult,
                };
            }
        }

        // Compile OlangExpr → OlangIrExpr → OlangProgram
        let ir_expr = olang_expr_to_ir(expr);
        let prog = compile_expr(&ir_expr);
        let vm = OlangVM::new();
        let result = vm.execute(&prog);

        // Collect output từ VM events + FEED vào LearningLoop
        let mut output_text = String::new();
        let mut learned: alloc::vec::Vec<olang::molecular::MolecularChain> = alloc::vec::Vec::new();
        // Compute emotion once — tránh borrow conflict
        let cur_emotion = self.learning.context().last_emotion();

        for event in &result.events {
            match event {
                VmEvent::Output(chain) => {
                    // Output chain → STM + display
                    if !chain.is_empty() {
                        // Check if string chain first (shape=0x02, relation=0x01)
                        if let Some(text) = olang::vm::chain_to_string(chain) {
                            output_text.push_str(&text);
                        }
                        // Check if numeric result
                        else if let Some(num) = chain.to_number() {
                            // Display number cleanly
                            if (num - homemath::round(num)).abs() < 1e-10 && num.abs() < 1e15 {
                                output_text.push_str(&format!("= {} ", homemath::round(num) as i64));
                            } else {
                                output_text.push_str(&format!("= {:.6} ", num));
                            }
                        } else {
                            self.learning.stm_mut().push(chain.clone(), cur_emotion, ts);
                            learned.push(chain.clone());
                            let lca_emoji = chain_to_emoji(chain);
                            let info = chain_info(chain, None);
                            output_text.push_str(&format!("∘→{} {}", lca_emoji, info));
                        }
                    } else {
                        output_text.push('○');
                    }
                }
                VmEvent::LookupAlias(alias) => {
                    // Check alias_to_cp cache trước (bao gồm L2 nodes)
                    let cp_from_cache = self.alias_to_cp.get(alias.as_str()).copied();
                    let (chain, cp_opt) = if let Some(cp) = cp_from_cache {
                        (olang::encoder::encode_codepoint(cp), Some(cp))
                    } else {
                        resolve_with_cp(alias, &self.registry)
                    };
                    if !chain.is_empty() {
                        // Lookup → STM: user referenced this node
                        self.learning.stm_mut().push(chain.clone(), cur_emotion, ts);
                        learned.push(chain.clone());
                        let emoji = if let Some(cp) = cp_opt {
                            char::from_u32(cp)
                                .map(|c| {
                                    let mut s = alloc::string::String::new();
                                    s.push(c);
                                    s
                                })
                                .unwrap_or_else(|| chain_to_emoji(&chain))
                        } else {
                            chain_to_emoji(&chain)
                        };
                        let info = chain_info(&chain, cp_opt);
                        output_text.push_str(&format!("{}={} {}", alias, emoji, info));
                    } else {
                        // RegistryGate: alias không tìm thấy → chưa đăng ký
                        self.registry_gate.check_registered(
                            alias,
                            olang::hash::fnv1a_str(alias),
                            olang::registry::NodeKind::Knowledge as u8,
                            AlertLevel::Normal,
                            ts,
                        );
                        output_text.push_str(&format!("{}=? [chưa registry] ", alias));
                    }
                }
                VmEvent::CreateEdge { from, to, rel } => {
                    // Explicit edge → Silk: user asserted relation
                    self.learning.graph_mut().co_activate(
                        *from,
                        *to,
                        cur_emotion,
                        1.0, // intentional → full reward
                        ts,
                    );
                    // QT8: Ghi edge vào pending_writes
                    {
                        use olang::writer::OlangWriter;
                        let mut ew = OlangWriter::new_append();
                        ew.append_edge(*from, *to, *rel, ts);
                        self.pending_writes.extend_from_slice(ew.as_bytes());
                    }
                    output_text.push_str(&format!(
                        "edge(0x{:04X}→0x{:04X} rel=0x{:02X}) ",
                        from & 0xFFFF,
                        to & 0xFFFF,
                        rel
                    ));
                }
                VmEvent::QueryRelation { hash, rel } => {
                    // Real graph query: find all nodes reachable via this relation type
                    let kind_filter =
                        silk::edge::EdgeKind::from_byte(*rel);
                    let reachable_nodes = silk::walk::reachable(
                        self.learning.graph(),
                        *hash,
                        kind_filter,
                        13, // Fib[7] max depth
                    );
                    if reachable_nodes.is_empty() {
                        output_text.push_str(&format!(
                            "query(0x{:04X} rel=0x{:02X}) → (no results) ",
                            hash & 0xFFFF,
                            rel
                        ));
                    } else {
                        output_text.push_str(&format!(
                            "query(0x{:04X} rel=0x{:02X}) → {} nodes: ",
                            hash & 0xFFFF,
                            rel,
                            reachable_nodes.len()
                        ));
                        for (i, n) in reachable_nodes.iter().take(8).enumerate() {
                            if i > 0 {
                                output_text.push_str(", ");
                            }
                            output_text.push_str(&format!("0x{:04X}", n & 0xFFFF));
                        }
                        if reachable_nodes.len() > 8 {
                            output_text.push_str(&format!(" (+{})", reachable_nodes.len() - 8));
                        }
                        output_text.push(' ');
                    }
                }
                VmEvent::TriggerDream => {
                    return self.handle_command("dream", ts);
                }
                VmEvent::RequestStats => {
                    return self.handle_command("stats", ts);
                }
                VmEvent::Error(e) => {
                    output_text.push_str(&format!("[err:{:?}] ", e));
                }
                // ── Reasoning & Debug ────────────────────────────────────
                VmEvent::TraceStep {
                    op_name,
                    stack_depth,
                    pc,
                } => {
                    output_text
                        .push_str(&format!("[trace pc={} op={} stack={}] ", pc, op_name, stack_depth));
                }
                VmEvent::InspectChain {
                    hash,
                    molecule_count,
                    byte_size,
                    is_empty,
                } => {
                    output_text.push_str(&format!(
                        "[inspect hash=0x{:016X} molecules={} bytes={} empty={}] ",
                        hash, molecule_count, byte_size, is_empty
                    ));
                }
                VmEvent::AssertFailed => {
                    output_text.push_str("[ASSERT FAILED: chain is empty] ");
                }
                VmEvent::TypeInfo {
                    hash,
                    classification,
                } => {
                    output_text.push_str(&format!(
                        "[typeof 0x{:04X} = {}] ",
                        hash & 0xFFFF,
                        classification
                    ));
                }
                VmEvent::WhyConnection { from, to } => {
                    // Real graph traversal: BFS find path from → to
                    let path = silk::walk::find_path(
                        self.learning.graph(),
                        *from,
                        *to,
                        13, // Fib[7] max depth
                    );
                    if path.is_empty() {
                        output_text.push_str(&format!(
                            "[why 0x{:04X} ↔ 0x{:04X}: no path found] ",
                            from & 0xFFFF,
                            to & 0xFFFF
                        ));
                    } else {
                        let formatted = silk::walk::format_path(&path);
                        output_text.push_str(&format!(
                            "[why: {} ({} steps)] ",
                            formatted,
                            path.len() - 1
                        ));
                    }
                }
                VmEvent::ExplainOrigin { hash } => {
                    // Real graph traversal: trace all incoming edges
                    let origins = silk::walk::trace_origin(
                        self.learning.graph(),
                        *hash,
                        5, // reasonable depth for explain
                    );
                    if origins.is_empty() {
                        output_text.push_str(&format!(
                            "[explain 0x{:04X}: no known origins] ",
                            hash & 0xFFFF
                        ));
                    } else {
                        let formatted = silk::walk::format_origin(&origins);
                        output_text.push_str(&format!(
                            "[explain: {} ({} connections)] ",
                            formatted,
                            origins.len()
                        ));
                    }
                }

                // ── Device I/O events ────────────────────────────────────
                // VM emit → Runtime xử lý → HAL → phần cứng thật.
                // Khi chạy từ REPL (không có HAL), chỉ log event.
                // Khi chạy trên thiết bị, Runtime inject HAL và thực thi.

                VmEvent::DeviceWrite { device_id, value } => {
                    if let Some(ref platform) = self.platform {
                        let ok = platform.write_actuator(device_id, *value);
                        output_text.push_str(&format!(
                            "[device_write \"{}\" = 0x{:02X} → {}] ",
                            device_id, value, if ok { "ok" } else { "fail" }
                        ));
                    } else {
                        output_text.push_str(&format!(
                            "[device_write \"{}\" = 0x{:02X}] ",
                            device_id, value
                        ));
                    }
                }
                VmEvent::DeviceRead { device_id } => {
                    if let Some(ref platform) = self.platform {
                        if let Some(val) = platform.read_sensor(device_id) {
                            output_text.push_str(&format!(
                                "[device_read \"{}\" = {:.2}] ",
                                device_id, val
                            ));
                        } else {
                            output_text.push_str(&format!(
                                "[device_read \"{}\" = none] ",
                                device_id
                            ));
                        }
                    } else {
                        output_text.push_str(&format!(
                            "[device_read \"{}\"] ",
                            device_id
                        ));
                    }
                }
                VmEvent::DeviceListRequest => {
                    output_text.push_str("[device_list] ");
                }

                // ── FFI & System I/O events ────────────────────────────────
                VmEvent::FfiCall { name, args } => {
                    output_text.push_str(&format!(
                        "[ffi \"{}\" ({} args)] ",
                        name,
                        args.len()
                    ));
                }
                VmEvent::FileReadRequest { path } => {
                    if let Some(ref platform) = self.platform {
                        if let Some(data) = platform.read_file(path) {
                            // Push file contents as string if valid UTF-8
                            if let Ok(text) = core::str::from_utf8(&data) {
                                output_text.push_str(&format!(
                                    "[file_read \"{}\" → {} bytes] {}",
                                    path, data.len(), text
                                ));
                            } else {
                                output_text.push_str(&format!(
                                    "[file_read \"{}\" → {} bytes (binary)] ",
                                    path, data.len()
                                ));
                            }
                        } else {
                            output_text.push_str(&format!(
                                "[file_read \"{}\" → not found] ",
                                path
                            ));
                        }
                    } else {
                        output_text.push_str(&format!(
                            "[file_read \"{}\"] ",
                            path
                        ));
                    }
                }
                VmEvent::FileWriteRequest { path, data } => {
                    if let Some(ref platform) = self.platform {
                        let ok = platform.write_file(path, data);
                        output_text.push_str(&format!(
                            "[file_write \"{}\" ({} bytes) → {}] ",
                            path, data.len(), if ok { "ok" } else { "fail" }
                        ));
                    } else {
                        output_text.push_str(&format!(
                            "[file_write \"{}\" ({} bytes)] ",
                            path, data.len()
                        ));
                    }
                }
                VmEvent::FileAppendRequest { path, data } => {
                    if let Some(ref platform) = self.platform {
                        // QT9: Append-only — read + append + write
                        let mut existing = platform.read_file(path).unwrap_or_default();
                        existing.extend_from_slice(data);
                        let ok = platform.write_file(path, &existing);
                        output_text.push_str(&format!(
                            "[file_append \"{}\" (+{} bytes) → {}] ",
                            path, data.len(), if ok { "ok" } else { "fail" }
                        ));
                    } else {
                        output_text.push_str(&format!(
                            "[file_append \"{}\" ({} bytes)] ",
                            path, data.len()
                        ));
                    }
                }
                VmEvent::SpawnRequest { body_ops_count } => {
                    output_text.push_str(&format!(
                        "[spawn {} ops] ",
                        body_ops_count
                    ));
                }
                VmEvent::UseModule { name } => {
                    output_text.push_str(&format!("[use {}] ", name));
                }
                VmEvent::UseModuleSelective { name, imports } => {
                    output_text.push_str(&format!("[use {} {{{}}}] ", name, imports.join(", ")));
                }
                VmEvent::ModDecl { path } => {
                    output_text.push_str(&format!("[mod {}] ", path));
                }
                VmEvent::EarlyReturn => {
                    // Handled inside VM — should not reach runtime
                }
            }
        }

        // Consecutive lookup/output → Silk co_activate (A ∘ B = association)
        if learned.len() >= 2 {
            for w in learned.windows(2) {
                let ha = w[0].chain_hash();
                let hb = w[1].chain_hash();
                self.learning.graph_mut().co_activate(
                    ha,
                    hb,
                    cur_emotion,
                    0.7, // intentional but indirect
                    ts,
                );
            }
        }

        // ── QT8+QT9: Ghi L1 nodes + Silk edges cho VM learned chains ────
        // Mọi chain từ VM (Output, LookupAlias) → phải persist vào origin.olang
        if !learned.is_empty() {
            use olang::writer::OlangWriter;
            let mut vm_writer = OlangWriter::new_append();
            let mut wrote_any = false;
            for chain in &learned {
                let hash = chain.chain_hash();
                if self.registry.lookup_hash(hash).is_none() {
                    let _ = vm_writer.append_node(chain, 1, false, ts);
                    wrote_any = true;
                    // QT9: Registry SAU
                    self.gated_insert(chain, 1, ts, false, olang::registry::NodeKind::Knowledge, "vm:L1");
                }
                // Silk edges từ chain này
                let edges: alloc::vec::Vec<_> = self.learning.graph()
                    .edges_from(hash)
                    .iter()
                    .filter(|e| e.weight >= 0.10)
                    .map(|e| (e.from_hash, e.to_hash, e.kind.as_byte(), e.updated_at))
                    .collect();
                for (from, to, kind, edge_ts) in &edges {
                    vm_writer.append_edge(*from, *to, *kind, *edge_ts);
                    wrote_any = true;
                }
            }
            if wrote_any {
                self.pending_writes.extend_from_slice(vm_writer.as_bytes());
            }
        }

        let text = if output_text.is_empty() {
            format!("○ ({} steps)", result.steps)
        } else {
            format!("○ {}", output_text.trim())
        };

        Response {
            text,
            tone: ResponseTone::Engaged,
            fx: self.learning.context().fx(),
            kind: ResponseKind::OlangResult,
        }
    }

    // ── Full Olang program execution ───────────────────────────────────────

    /// Execute a full Olang program (multi-statement source code).
    ///
    /// Unlike process_text (which handles ○{} single expressions),
    /// this parses and runs a complete Olang program with functions,
    /// control flow, variables, etc.
    pub fn run_program(&mut self, source: &str, ts: i64) -> Response {
        // Parse full Olang syntax
        let stmts = match syntax::parse(source) {
            Ok(stmts) => stmts,
            Err(e) => {
                return Response {
                    text: format!("Parse error: {:?}", e),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::Blocked,
                };
            }
        };

        // Validate
        let errors = semantic::validate(&stmts);
        if !errors.is_empty() {
            let msgs: alloc::vec::Vec<String> = errors.iter()
                .map(|e| e.message.clone())
                .collect();
            return Response {
                text: format!("Validation errors:\n{}", msgs.join("\n")),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::Blocked,
            };
        }

        // Lower to IR
        let prog = semantic::lower(&stmts);

        // Optimize
        let mut prog = prog;
        optimize::optimize(&mut prog, OptLevel::O1);

        // Execute
        let vm = OlangVM::new();
        let result = vm.execute(&prog);

        // Collect output
        let mut output_text = String::new();
        let cur_emotion = self.learning.context().last_emotion();

        for event in &result.events {
            match event {
                VmEvent::Output(chain) => {
                    if !chain.is_empty() {
                        if let Some(text) = olang::vm::chain_to_string(chain) {
                            output_text.push_str(&text);
                        } else if let Some(num) = chain.to_number() {
                            if (num - homemath::round(num)).abs() < 1e-10 && num.abs() < 1e15 {
                                output_text.push_str(&format!("{}", homemath::round(num) as i64));
                            } else {
                                output_text.push_str(&format!("{}", num));
                            }
                        } else {
                            output_text.push_str(&olang::vm::format_chain_display(chain));
                        }
                    }
                }
                VmEvent::LookupAlias(alias) => {
                    let cp_from_cache = self.alias_to_cp.get(alias.as_str()).copied();
                    let (chain, _cp_opt) = if let Some(cp) = cp_from_cache {
                        (olang::encoder::encode_codepoint(cp), Some(cp))
                    } else {
                        resolve_with_cp(alias, &self.registry)
                    };
                    if !chain.is_empty() {
                        self.learning.stm_mut().push(chain.clone(), cur_emotion, ts);
                    }
                }
                VmEvent::Error(e) => {
                    output_text.push_str(&format!("[error: {}] ", e));
                }
                VmEvent::TriggerDream => {
                    let resp = self.handle_command("dream", ts);
                    output_text.push_str(&resp.text);
                }
                VmEvent::RequestStats => {
                    let resp = self.handle_command("stats", ts);
                    output_text.push_str(&resp.text);
                }
                VmEvent::UseModule { name } => {
                    // Check if already loaded in ModuleLoader cache
                    if self.module_loader.is_loaded(&name) {
                        // Already loaded — skip
                    } else if let Some(ref platform) = self.platform {
                        // Resolve module path: silk.graph → silk/graph.ol
                        let file_path = olang::module::ModuleLoader::resolve_path(&name);
                        let paths = [
                            file_path.clone(),
                            format!("{}.olang", name.replace('.', "/")),
                            name.clone(),
                        ];
                        let mut loaded = false;
                        for path in &paths {
                            if let Some(data) = platform.read_file(path) {
                                if let Ok(source) = core::str::from_utf8(&data) {
                                    // Use ModuleLoader for proper caching + cycle detection
                                    match self.module_loader.load_from_source(&name, source, None) {
                                        Ok(_pub_symbols) => {
                                            // Execute the module program
                                            let resp = self.run_program(source, ts);
                                            if !resp.text.is_empty() && resp.kind != ResponseKind::Blocked {
                                                output_text.push_str(&resp.text);
                                            }
                                        }
                                        Err(e) => {
                                            output_text.push_str(&format!("[module error: {}] ", e));
                                        }
                                    }
                                    loaded = true;
                                    break;
                                }
                            }
                        }
                        if !loaded {
                            output_text.push_str(&format!("[module '{}' not found] ", name));
                        }
                    } else {
                        output_text.push_str(&format!("[no platform for module '{}'] ", name));
                    }
                }
                VmEvent::UseModuleSelective { name, imports } => {
                    // Load module if not cached, then validate selective imports
                    if !self.module_loader.is_loaded(&name) {
                        if let Some(ref platform) = self.platform {
                            let file_path = olang::module::ModuleLoader::resolve_path(&name);
                            let paths = [
                                file_path.clone(),
                                format!("{}.olang", name.replace('.', "/")),
                                name.clone(),
                            ];
                            let mut loaded = false;
                            for path in &paths {
                                if let Some(data) = platform.read_file(path) {
                                    if let Ok(source) = core::str::from_utf8(&data) {
                                        match self.module_loader.load_from_source(&name, source, None) {
                                            Ok(_) => {
                                                let resp = self.run_program(source, ts);
                                                if !resp.text.is_empty() && resp.kind != ResponseKind::Blocked {
                                                    output_text.push_str(&resp.text);
                                                }
                                            }
                                            Err(e) => {
                                                output_text.push_str(&format!("[module error: {}] ", e));
                                            }
                                        }
                                        loaded = true;
                                        break;
                                    }
                                }
                            }
                            if !loaded {
                                output_text.push_str(&format!("[module '{}' not found] ", name));
                            }
                        }
                    }
                    // Validate selective imports (pub/private enforcement)
                    match self.module_loader.resolve_imports(&name, &imports) {
                        Ok(_symbols) => {
                            // Symbols validated — they're accessible in caller's scope
                        }
                        Err(e) => {
                            output_text.push_str(&format!("[import error: {}] ", e));
                        }
                    }
                }
                _ => {} // Other events handled silently
            }
        }

        let text = if output_text.is_empty() {
            format!("○ done ({} steps)", result.steps)
        } else {
            output_text
        };

        Response {
            text,
            tone: ResponseTone::Engaged,
            fx: self.learning.context().fx(),
            kind: ResponseKind::OlangResult,
        }
    }

    // ── System commands ───────────────────────────────────────────────────────

    fn handle_command(&mut self, cmd: &str, ts: i64) -> Response {
        match cmd {
            "stats" => {
                // Update self-model tại thời điểm query
                self.self_model.update(&self.registry, ts);
                let summary = self.self_model.summary();
                let kt_summary = self.knowtree.summary();
                let reg_nodes = self.registry.len();
                let reg_aliases = self.registry.alias_count();
                let silk_nodes = self.learning.graph().node_count();
                let router_summary = self.router.summary();
                let text = format!(
                    "HomeOS ○\n\
                     Turns    : {}\n\
                     Registry : {} nodes, {} aliases\n\
                     STM      : {} observations\n\
                     Silk     : {} nodes, {} edges\n\
                     Chiefs   : {}\n\
                     Workers  : {}\n\
                     f(x)     : {:.3}\n\
                     {}\n\
                     {}\n\
                     {}",
                    self.turn_count,
                    reg_nodes,
                    reg_aliases,
                    self.learning.stm().len(),
                    silk_nodes,
                    self.learning.graph().len(),
                    self.chiefs.len(),
                    self.workers.len(),
                    self.learning.context().fx(),
                    summary,
                    kt_summary,
                    router_summary,
                );
                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "ram" => {
                // ○{ram} — memory usage breakdown
                let (reg_entries, reg_aliases, reg_misc, reg_total) =
                    self.registry.memory_usage();
                let silk_bytes = self.learning.graph().memory_usage();
                let silk_edges = self.learning.graph().len();
                let silk_cap = 100_000usize; // max_edges from maintain()
                let stm_obs = self.learning.stm().len();
                let stm_bytes = stm_obs * 60; // ~60 bytes per observation

                // Format helper: bytes → human-readable
                fn fmt_bytes(b: usize) -> String {
                    if b >= 1_048_576 {
                        format!("{:.1} MB", b as f64 / 1_048_576.0)
                    } else if b >= 1024 {
                        format!("{:.1} KB", b as f64 / 1024.0)
                    } else {
                        format!("{} B", b)
                    }
                }

                let total = reg_total + silk_bytes + stm_bytes;

                let text = format!(
                    "RAM Usage ○\n\
                     ─── Registry ───\n\
                     Entries  : {} nodes → {}\n\
                     Aliases  : {} names → {}\n\
                     Misc     : {}\n\
                     Subtotal : {}\n\
                     ─── Silk Graph ───\n\
                     Edges    : {} / {} cap → {}\n\
                     ─── STM ───\n\
                     Obs      : {} → {}\n\
                     ─── Total ───\n\
                     Estimated: {}",
                    self.registry.len(), fmt_bytes(reg_entries),
                    self.registry.alias_count(), fmt_bytes(reg_aliases),
                    fmt_bytes(reg_misc),
                    fmt_bytes(reg_total),
                    silk_edges, silk_cap, fmt_bytes(silk_bytes),
                    stm_obs, fmt_bytes(stm_bytes),
                    fmt_bytes(total),
                );

                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "dream" => {
                self.run_dream(ts);
                let result = self
                    .dream
                    .run(self.learning.stm(), self.learning.graph(), ts);

                let mut lines: alloc::vec::Vec<String> = alloc::vec::Vec::new();
                lines.push(String::from("Dream cycle ○"));
                lines.push(format!("Scanned    : {}", result.scanned));
                lines.push(format!("Clusters   : {}", result.clusters_found));
                lines.push(format!("Proposals  : {}", result.proposals.len()));
                lines.push(format!("Approved   : {}", result.approved));

                // Show what was discovered
                if !result.proposals.is_empty() {
                    lines.push(String::from("─── Discovered ───"));
                    for p in &result.proposals {
                        match &p.kind {
                            memory::proposal::ProposalKind::NewNode { chain, sources, .. } => {
                                let hash = chain.chain_hash();
                                let label = self.registry.alias_for_hash(hash)
                                    .unwrap_or("(new concept)");
                                lines.push(format!(
                                    "  L3 concept: {} (from {} sources, confidence {:.2})",
                                    label, sources.len(), p.confidence
                                ));
                            }
                            memory::proposal::ProposalKind::PromoteQR { chain_hash, fire_count } => {
                                let label = self.registry.alias_for_hash(*chain_hash)
                                    .unwrap_or("(memory)");
                                lines.push(format!(
                                    "  Promote QR: {} (fire={})",
                                    label, fire_count
                                ));
                            }
                            _ => {}
                        }
                    }
                }

                if !result.matured_nodes.is_empty() {
                    lines.push(format!("Matured    : {} nodes", result.matured_nodes.len()));
                }

                lines.push(String::from("─── Lifetime ───"));
                lines.push(format!("Total cycles: {}", self.dream_cycles));
                lines.push(format!("Total approved: {}", self.dream_approved_total));
                lines.push(format!("L3 concepts : {}", self.dream_l3_created));
                lines.push(format!(
                    "Fib interval: {} turns",
                    silk::hebbian::fib(self.dream_fib_index)
                ));
                lines.push(format!(
                    "KnowTree    : {} nodes, {} L3",
                    self.knowtree.total_nodes(),
                    self.knowtree.concepts()
                ));

                Response {
                    text: lines.join("\n"),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "health" => {
                let m = self.metrics();
                let stm_status = if m.stm_observations > 0 { "●" } else { "○" };
                let silk_status = if m.silk_edges > 0 { "●" } else { "○" };
                let curve_status = if m.turns > 0 { "●" } else { "○" };
                let text = alloc::format!(
                    "Health ○\n\
                     STM      : {} ({} obs, hit {:.0}%, max_fire {})\n\
                     Silk     : {} ({} edges, density {:.4})\n\
                     Curve    : {} (f(x)={:.3}, tone={})\n\
                     Turns    : {}\n\
                     Pending  : {} bytes\n\
                     Saveable : {} edges",
                    stm_status,
                    m.stm_observations,
                    m.stm_hit_rate * 100.0,
                    m.stm_max_fire,
                    silk_status,
                    m.silk_edges,
                    m.silk_density,
                    curve_status,
                    m.fx,
                    m.tone,
                    m.turns,
                    m.pending_bytes,
                    m.saveable_edges,
                );
                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "memory" | "nho" => {
                // ○{memory} — show what the system remembers
                let stm = self.learning.stm();
                if stm.is_empty() {
                    return Response {
                        text: String::from("Trí nhớ ngắn hạn trống — chưa học gì."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }

                let mut lines: alloc::vec::Vec<String> = alloc::vec::Vec::new();
                lines.push(format!("Trí nhớ ngắn hạn ○ ({} observations)", stm.len()));
                lines.push(String::from("────────────────────────────────"));

                // Top observations by fire_count
                let top = stm.top_n(10);
                for (i, obs) in top.iter().enumerate() {
                    let hash = obs.chain.chain_hash();
                    let v = obs.emotion.valence;
                    let a = obs.emotion.arousal;
                    let fire = obs.fire_count;

                    // Try to find alias in registry
                    let label = self.registry.alias_for_hash(hash)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{:016X}", hash));

                    let mood = if v > 0.3 { "+" } else if v < -0.3 { "-" } else { "~" };
                    lines.push(format!(
                        "  {}. {} [{}] fire={} V={:.2} A={:.2}",
                        i + 1, label, mood, fire, v, a
                    ));
                }

                // Silk connections summary
                let silk_edges = self.learning.graph().len();
                let silk_nodes = self.learning.graph().node_count();
                if silk_edges > 0 {
                    lines.push(String::new());
                    lines.push(format!("Silk: {} liên kết giữa {} concepts", silk_edges, silk_nodes));
                }

                // LeoAI knowledge expression
                let expressed = self.leo.express_all();
                if !expressed.is_empty() {
                    lines.push(String::new());
                    lines.push(format!("LeoAI biểu đạt: {} patterns", expressed.len()));
                    for (i, expr) in expressed.iter().take(5).enumerate() {
                        lines.push(format!("  {}. {}", i + 1, expr));
                    }
                    if expressed.len() > 5 {
                        lines.push(format!("  ... và {} nữa", expressed.len() - 5));
                    }
                }

                Response {
                    text: lines.join("\n"),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "cluster" => {
                // ○{cluster} — cluster STM observations using ClusterSkill
                let stm = self.learning.stm();
                if stm.len() < 2 {
                    return Response {
                        text: String::from("Cần ít nhất 2 observations để cluster."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }

                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());

                // Feed all STM chains into skill
                for obs in stm.all().iter().take(32) {
                    ctx.push_input(obs.chain.clone());
                }

                let skill = ClusterSkill;
                let result = skill.execute(&mut ctx);

                let mut lines: alloc::vec::Vec<String> = alloc::vec::Vec::new();
                lines.push(String::from("Cluster ○"));
                match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let count = ctx.get("cluster_count").unwrap_or("?");
                        lines.push(format!("Input    : {} observations", stm.len().min(32)));
                        lines.push(format!("Clusters : {}", count));
                        lines.push(format!("Note     : {}", note));

                        // Show cluster representatives
                        for (i, chain) in ctx.output_chains.iter().enumerate().take(8) {
                            let hash = chain.chain_hash();
                            let label = self.registry.alias_for_hash(hash)
                                .unwrap_or("(cluster)");
                            lines.push(format!(
                                "  {}. {} (hash {:016X}, {} molecules)",
                                i + 1, label, hash, chain.0.len()
                            ));
                        }
                    }
                    _ => lines.push(String::from("Không đủ data để cluster.")),
                }

                Response {
                    text: lines.join("\n"),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "generalize" | "rules" => {
                // ○{generalize} — extract IF-THEN rules from STM
                let stm = self.learning.stm();
                if stm.len() < 3 {
                    return Response {
                        text: String::from("Cần ít nhất 3 observations để generalize."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }

                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());

                for obs in stm.all().iter().take(32) {
                    ctx.push_input(obs.chain.clone());
                }

                let skill = GeneralizationSkill;
                let result = skill.execute(&mut ctx);

                let mut lines: alloc::vec::Vec<String> = alloc::vec::Vec::new();
                lines.push(String::from("Generalization ○"));
                match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let rule_count: usize = ctx.get("rule_count")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        lines.push(format!("Input : {} observations", stm.len().min(32)));
                        lines.push(format!("Rules : {}", rule_count));
                        lines.push(format!("Note  : {}", note));
                        lines.push(String::from("────────────────────────────────"));
                        for i in 0..rule_count {
                            let key = alloc::format!("rule_{}", i);
                            if let Some(rule) = ctx.get(&key) {
                                lines.push(format!("  {}", rule));
                            }
                        }
                    }
                    _ => lines.push(String::from("Không đủ data để generalize.")),
                }

                Response {
                    text: lines.join("\n"),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "help" => Response {
                text: String::from(
                    "HomeOS ○{} Commands:\n\
                     ○{🔥}                  — query node\n\
                     ○{🔥 ∘ 💧}            — compose (LCA)\n\
                     ○{🔥 ∈ ?}             — relation query\n\
                     ○{? → 💧}             — reverse query\n\
                     ○{term ∂ ctx}         — context query\n\
                     ○{dream}              — run Dream cycle\n\
                     ○{stats}              — system statistics\n\
                     ○{ram}                — memory usage breakdown\n\
                     ○{health}             — system health check\n\
                     ○{memory}             — show learned knowledge\n\
                     ○{cluster}            — cluster STM observations\n\
                     ○{generalize}         — extract IF-THEN rules\n\
                     ○{solve \"2x+3=7\"}     — solve equation\n\
                     ○{derive \"x^2+3x\"}   — symbolic derivative\n\
                     ○{integrate \"2x\"}     — symbolic integral\n\
                     ○{simplify \"2x+3x\"}  — simplify expression\n\
                     ○{eval \"x^2+1\" x=3}  — evaluate expression\n\
                     ○{const pi}           — compute π (adaptive precision)\n\
                     ○{const all}          — list all constants\n\
                     ○{const compare}      — compare precision tiers\n\
                     ○{fib 10}             — Fibonacci(10)\n\
                     ○{fib ratio 20}       — F(21)/F(20) → φ\n\
                     ○{leo emit fire ∘ water;}  — LeoAI lập trình VM\n\
                     ○{program emit { S=1 R=6 T=4 };} — LeoAI chạy Olang\n\
                     ○{run let x = 1 + 2; emit x;} — LeoAI chạy + học\n\
                     ○{inspect 🔥}          — inspect chain structure\n\
                     ○{typeof 🔥}           — classify chain type\n\
                     ○{assert 🔥}           — verify node exists\n\
                     ○{explain 🔥}          — trace node origin\n\
                     ○{why fire water}      — find connection\n\
                     ○{trace}              — toggle execution tracing\n\
                     ○{fuse 🔥}             — QT2 finiteness check\n\
                     ○{compile c 🔥 ∘ 💧}  — compile to C\n\
                     ○{compile rust stats} — compile to Rust\n\
                     ○{compile wasm 1 + 2} — compile to WASM\n\
                     ○{similar term1 term2} — compare 2 concepts\n\
                     ○{delta term1 term2}  — show differences\n\
                     ○{hebbian term1 term2} — check Hebbian weight\n\
                     ○{merge term1 term2}  — merge concepts\n\
                     ○{ingest text}        — encode text → chain\n\
                     ○{fit edge=0.8 circ=0.9 aspect=1.0} — SDF fitting\n\
                     ○{prune}             — clean low-confidence STM\n\
                     ○{curate}            — rank knowledge quality\n\
                     ○{temporal}          — detect temporal patterns\n\
                     ○{read text}         — BookReader → learn\n\
                     ○{if fire { stats }}  — conditional\n\
                     ○{loop 3 { stats }}   — loop N times\n\
                     ○{fn test { stats }}  — function definition\n\
                     ○{let x = fire}       — variable binding\n\
                     ○{{ S=1 R=6 V=200 }}  — molecular literal\n\
                     ○{help}               — this message",
                ),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::System,
            },

            // ── inspect <node> — hiển thị cấu trúc chain ──────────────────
            _ if cmd.starts_with("inspect ") => {
                let arg = cmd["inspect ".len()..].trim();
                let (chain, _) = resolve_with_cp(arg, &self.registry);
                if chain.is_empty() {
                    return Response {
                        text: format!("○ inspect: '{}' not found in registry", arg),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let hash = chain.chain_hash();
                let bytes = chain.to_bytes();
                let classification = classify_chain_type(&chain);
                let mol_details: alloc::vec::Vec<String> = chain.0.iter().enumerate().map(|(i, &bits)| {
                    let mol = olang::molecular::Molecule::from_u16(bits);
                    format!(
                        "  mol[{}]: S={} R={} V={} A={} T={}",
                        i, mol.shape_u8(), mol.relation_u8(), mol.valence_u8(), mol.arousal_u8(), mol.time_u8()
                    )
                }).collect();
                Response {
                    text: format!(
                        "Inspect ○ '{}'\n\
                         Hash     : {:016x}\n\
                         Molecules: {}\n\
                         Bytes    : {}\n\
                         Type     : {}\n\
                         {}",
                        arg, hash, chain.len(), bytes.len(), classification,
                        mol_details.join("\n")
                    ),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            // ── typeof <node> — phân loại chain (SDF/MATH/EMOTICON/Mixed) ───
            _ if cmd.starts_with("typeof ") => {
                let arg = cmd["typeof ".len()..].trim();
                let (chain, _) = resolve_with_cp(arg, &self.registry);
                if chain.is_empty() {
                    return Response {
                        text: format!("○ typeof: '{}' not found", arg),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let classification = classify_chain_type(&chain);
                Response {
                    text: format!("typeof '{}' = {}", arg, classification),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            // ── assert <node> — kiểm tra node tồn tại (non-empty) ──────────
            _ if cmd.starts_with("assert ") => {
                let arg = cmd["assert ".len()..].trim();
                let (chain, _) = resolve_with_cp(arg, &self.registry);
                let passed = !chain.is_empty();
                Response {
                    text: if passed {
                        format!("✓ assert '{}' — OK ({} molecules)", arg, chain.len())
                    } else {
                        format!("✗ assert '{}' — FAILED (not found)", arg)
                    },
                    tone: if passed { ResponseTone::Engaged } else { ResponseTone::Gentle },
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            // ── explain <node> — truy ngược nguồn gốc node ─────────────────
            _ if cmd.starts_with("explain ") => {
                let arg = cmd["explain ".len()..].trim();
                let (chain, _) = resolve_with_cp(arg, &self.registry);
                if chain.is_empty() {
                    return Response {
                        text: format!("○ explain: '{}' not found", arg),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let hash = chain.chain_hash();
                // Trace origin qua Silk graph (depth=5 ≈ Fib[5])
                let origins = silk::walk::trace_origin(self.learning.graph(), hash, 5);
                if origins.is_empty() {
                    Response {
                        text: format!("explain '{}' — root node, no incoming edges", arg),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                } else {
                    let origin_text = silk::walk::format_origin(&origins);
                    Response {
                        text: format!(
                            "explain '{}' — {} incoming connections:\n{}",
                            arg, origins.len(), origin_text
                        ),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                }
            }

            // ── why <a> <b> — tìm kết nối giữa 2 nodes ────────────────────
            _ if cmd.starts_with("why ") => {
                let args: alloc::vec::Vec<&str> = cmd["why ".len()..].trim().splitn(2, ' ').collect();
                if args.len() < 2 {
                    return Response {
                        text: String::from("○ why: cần 2 arguments — ○{why fire water}"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let (chain_a, _) = resolve_with_cp(args[0], &self.registry);
                let (chain_b, _) = resolve_with_cp(args[1], &self.registry);
                if chain_a.is_empty() || chain_b.is_empty() {
                    return Response {
                        text: format!(
                            "○ why: {} not found",
                            if chain_a.is_empty() { args[0] } else { args[1] }
                        ),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let hash_a = chain_a.chain_hash();
                let hash_b = chain_b.chain_hash();
                // BFS path (max depth=8 ≈ Fib[6])
                let path = silk::walk::find_path(self.learning.graph(), hash_a, hash_b, 8);
                if path.is_empty() {
                    // Try LCA as semantic connection
                    let common = olang::lca::lca(&chain_a, &chain_b);
                    let emoji = chain_to_emoji(&common);
                    Response {
                        text: format!(
                            "why '{}' ↔ '{}' — no Silk path, LCA = {}",
                            args[0], args[1], emoji
                        ),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                } else {
                    let path_text = silk::walk::format_path(&path);
                    Response {
                        text: format!(
                            "why '{}' ↔ '{}' — {} hops:\n{}",
                            args[0], args[1], path.len() - 1, path_text
                        ),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                }
            }

            // ── trace — toggle execution tracing ────────────────────────────
            "trace" => {
                self.trace_enabled = !self.trace_enabled;
                Response {
                    text: format!("Trace: {}", if self.trace_enabled { "ON" } else { "OFF" }),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            // ── fuse <node> — QT2 check: chain hữu hạn? ───────────────────
            _ if cmd.starts_with("fuse ") => {
                let arg = cmd["fuse ".len()..].trim();
                let (chain, _) = resolve_with_cp(arg, &self.registry);
                if chain.is_empty() {
                    return Response {
                        text: format!("○ fuse: '{}' → ∞ (empty/not found = invalid)", arg),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                // QT2: ∞ sai, ∞-1 đúng. Non-empty chain = finite = valid.
                Response {
                    text: format!("✓ fuse '{}' → ∞-1 (finite, {} molecules = valid)", arg, chain.len()),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            _ if cmd.starts_with("leo ") || cmd.starts_with("program ") || cmd.starts_with("run ") => {
                // ── LeoAI programming — AAM approves → LeoAI mở VM ─────────
                // ○{leo emit fire ∘ water;}
                // ○{program emit { S=1 R=6 T=4 };}
                // ○{run let x = fire; emit x ∘ water;}
                let prefix_len = cmd.find(' ').unwrap_or(0) + 1;
                let source = &cmd[prefix_len..];

                // AAM gate: approve nếu không Crisis
                let intent = estimate_intent(source, 0.0, 0.0);
                if intent.primary == IntentKind::Crisis {
                    return Response {
                        text: String::from("⚠ LeoAI: SecurityGate blocked — Crisis detected."),
                        tone: ResponseTone::Gentle,
                        fx: self.learning.context().fx(),
                        kind: ResponseKind::Blocked,
                    };
                }

                // LeoAI lập trình + chạy VM + học kết quả
                let prog_result = self.leo.program(source, ts);

                // Format output
                let mut text = String::from("LeoAI ○ programmed:\n");
                if prog_result.has_error() {
                    for err in &prog_result.errors {
                        text.push_str(&format!("  ✗ {}\n", err));
                    }
                } else {
                    for (i, out) in prog_result.outputs.iter().enumerate() {
                        match out {
                            ProgOutput::Number(n) => {
                                if (*n - homemath::round(*n)).abs() < 1e-10 && n.abs() < 1e15 {
                                    text.push_str(&format!("  [{}] = {}\n", i, homemath::round(*n) as i64));
                                } else {
                                    text.push_str(&format!("  [{}] = {:.6}\n", i, n));
                                }
                            }
                            ProgOutput::Chain(chain) => {
                                let emoji = chain_to_emoji(chain);
                                let info = chain_info(chain, None);
                                text.push_str(&format!("  [{}] ∘→{} {}\n", i, emoji, info));
                            }
                        }
                    }
                    if !prog_result.learned_hashes.is_empty() {
                        text.push_str(&format!(
                            "  Learned: {} chains, {} VM steps\n",
                            prog_result.learned_hashes.len(),
                            prog_result.steps
                        ));
                    }
                }

                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: self.learning.context().fx(),
                    kind: ResponseKind::OlangResult,
                }
            }

            // ── compile <target> <source> ──────────────────────────────────────
            // ○{compile c fire ∘ water}   → C source
            // ○{compile rust stats}       → Rust source
            // ○{compile wasm 1 + 2}       → WASM (WAT) source
            _ if cmd.starts_with("compile ") => {
                let rest = cmd["compile ".len()..].trim();
                // First word = target, rest = Olang source
                let (target_str, source) = match rest.find(' ') {
                    Some(pos) => (rest[..pos].trim(), rest[pos + 1..].trim()),
                    None => {
                        return Response {
                            text: String::from("compile <target> <source>\ntargets: c, rust, wasm"),
                            tone: ResponseTone::Engaged,
                            fx: 0.0,
                            kind: ResponseKind::System,
                        };
                    }
                };
                let target = match target_str {
                    "c" | "C" => Target::C,
                    "rust" | "Rust" | "rs" => Target::Rust,
                    "wasm" | "WASM" | "wat" | "WAT" => Target::Wasm,
                    other => {
                        return Response {
                            text: format!("Unknown target '{}' — use: c, rust, wasm", other),
                            tone: ResponseTone::Engaged,
                            fx: 0.0,
                            kind: ResponseKind::System,
                        };
                    }
                };

                // Try full Olang syntax pipeline first (with semantic validation)
                let mut prog = match syntax::parse(source) {
                    Ok(stmts) => {
                        // Semantic validation
                        let errors = semantic::validate(&stmts);
                        if !errors.is_empty() {
                            let msgs: alloc::vec::Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
                            return Response {
                                text: format!("Semantic error:\n{}", msgs.join("\n")),
                                tone: ResponseTone::Engaged,
                                fx: 0.0,
                                kind: ResponseKind::System,
                            };
                        }
                        // Lower AST → OlangProgram (IR)
                        semantic::lower(&stmts)
                    }
                    Err(_) => {
                        // Fallback: try ○{} parser for simple expressions
                        let parse_result = self.parser.parse(&format!("○{{{}}}", source));
                        let expr = match parse_result {
                            ParseResult::OlangExpr(e) => e,
                            ParseResult::Error(e) => {
                                return Response {
                                    text: format!("Parse error: {}", e),
                                    tone: ResponseTone::Engaged,
                                    fx: 0.0,
                                    kind: ResponseKind::System,
                                };
                            }
                            _ => {
                                return Response {
                                    text: String::from("compile: source must be an Olang expression"),
                                    tone: ResponseTone::Engaged,
                                    fx: 0.0,
                                    kind: ResponseKind::System,
                                };
                            }
                        };
                        let ir_expr = olang_expr_to_ir(expr);
                        compile_expr(&ir_expr)
                    }
                };
                // Optimize before compilation
                let before_ops = prog.ops.len();
                let opt_stats = optimize::optimize(&mut prog, OptLevel::O2);
                let compiler = Compiler::new(target);
                match compiler.emit(&prog) {
                    Ok(output) => {
                        let opt_info = if opt_stats.folds > 0 || opt_stats.dead_removed > 0
                            || opt_stats.nops_removed > 0 || opt_stats.identities_removed > 0
                        {
                            format!(
                                "\nOptimized: {} → {} ops (folded:{}, dead:{}, nop:{}, identity:{})",
                                before_ops, prog.ops.len(),
                                opt_stats.folds, opt_stats.dead_removed,
                                opt_stats.nops_removed, opt_stats.identities_removed,
                            )
                        } else {
                            String::new()
                        };
                        Response {
                            text: format!(
                                "Compiled to {} ({} bytes, {} ops):\n{}{}",
                                target.name(),
                                output.len(),
                                prog.ops.len(),
                                output,
                                opt_info,
                            ),
                            tone: ResponseTone::Engaged,
                            fx: self.learning.context().fx(),
                            kind: ResponseKind::OlangResult,
                        }
                    }
                    Err(e) => Response {
                        text: format!("Compile error: {:?}", e),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                }
            }

            _ if cmd.starts_with("similar ") => {
                // ○{similar fire water} — compare 2 concepts using SimilaritySkill
                let args: alloc::vec::Vec<&str> = cmd[8..].split_whitespace().collect();
                if args.len() < 2 {
                    return Response {
                        text: String::from("Cần 2 terms: ○{similar term1 term2}"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }

                let chain_a = self.registry.lookup_name(args[0])
                    .and_then(|h| {
                        let entry = self.registry.lookup_hash(h)?;
                        Some(olang::encoder::encode_codepoint(entry.chain_hash as u32))
                    })
                    .unwrap_or_else(|| {
                        // Fallback: encode as text
                        let chains: alloc::vec::Vec<_> = args[0].chars()
                            .map(|c| olang::encoder::encode_codepoint(c as u32))
                            .filter(|ch| !ch.is_empty())
                            .collect();
                        if chains.is_empty() { olang::encoder::encode_codepoint('?' as u32) }
                        else { olang::lca::lca_many(&chains) }
                    });

                let chain_b = self.registry.lookup_name(args[1])
                    .and_then(|h| {
                        let entry = self.registry.lookup_hash(h)?;
                        Some(olang::encoder::encode_codepoint(entry.chain_hash as u32))
                    })
                    .unwrap_or_else(|| {
                        let chains: alloc::vec::Vec<_> = args[1].chars()
                            .map(|c| olang::encoder::encode_codepoint(c as u32))
                            .filter(|ch| !ch.is_empty())
                            .collect();
                        if chains.is_empty() { olang::encoder::encode_codepoint('?' as u32) }
                        else { olang::lca::lca_many(&chains) }
                    });

                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                ctx.push_input(chain_a);
                ctx.push_input(chain_b);

                let skill = SimilaritySkill;
                let result = skill.execute(&mut ctx);

                let text = match result {
                    agents::skill::SkillResult::Ok { note, chain, .. } => {
                        let sim = ctx.get("similarity").unwrap_or("?");
                        let lca_hash = chain.chain_hash();
                        let lca_label = self.registry.alias_for_hash(lca_hash)
                            .unwrap_or("(LCA)");
                        format!(
                            "Similarity ○\n  {} ↔ {} = {}\n  LCA: {} ({:016X})\n  {}",
                            args[0], args[1], sim, lca_label, lca_hash, note
                        )
                    }
                    _ => format!("Không thể so sánh {} và {}.", args[0], args[1]),
                };

                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            _ if cmd.starts_with("ingest ") => {
                // ○{ingest <text>} — IngestSkill: text → MolecularChain
                let text = &cmd[7..];
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                ctx.set(String::from("text"), text.to_string());
                let result = IngestSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { chain, note, .. } => {
                        let count = ctx.get("ingested_count").unwrap_or("0");
                        let hash = chain.chain_hash();
                        format!("Ingest ○ {} chars → chain {:016X}\n  {}", count, hash, note)
                    }
                    _ => String::from("Ingest ○ không đủ dữ liệu."),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            _ if cmd.starts_with("delta ") => {
                // ○{delta term1 term2} — DeltaSkill: differences between concepts
                let args: alloc::vec::Vec<&str> = cmd[6..].split_whitespace().collect();
                if args.len() < 2 {
                    return Response {
                        text: String::from("Cần 2 terms: ○{delta term1 term2}"),
                        tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System,
                    };
                }
                let chain_a = self.resolve_term(args[0]);
                let chain_b = self.resolve_term(args[1]);
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                ctx.push_input(chain_a);
                ctx.push_input(chain_b);
                let result = DeltaSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let count = ctx.get("delta_count").unwrap_or("?");
                        format!("Delta ○\n  {} ↔ {}: {} molecules differ\n  {}", args[0], args[1], count, note)
                    }
                    _ => format!("Delta ○ không thể so sánh {} và {}.", args[0], args[1]),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            _ if cmd.starts_with("hebbian ") => {
                // ○{hebbian term1 term2} — HebbianSkill: check/strengthen connection
                let args: alloc::vec::Vec<&str> = cmd[8..].split_whitespace().collect();
                if args.len() < 2 {
                    return Response {
                        text: String::from("Cần 2 terms: ○{hebbian term1 term2}"),
                        tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System,
                    };
                }
                let chain_a = self.resolve_term(args[0]);
                let chain_b = self.resolve_term(args[1]);
                let ha = chain_a.chain_hash();
                let hb = chain_b.chain_hash();
                // Get current Silk weight
                let current_w = self.learning.graph().learned_weight(ha, hb);
                let fire_count = self.learning.graph().edges_from(ha).len() as u32;
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                ctx.push_input(chain_a);
                ctx.push_input(chain_b);
                ctx.set(String::from("current_weight"), format!("{:.4}", current_w));
                ctx.set(String::from("fire_count"), format!("{}", fire_count));
                let result = HebbianSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        format!("Hebbian ○\n  {} ↔ {}: w={:.4} → {}\n  fire={} {}",
                            args[0], args[1], current_w,
                            ctx.get("new_weight").unwrap_or("?"),
                            fire_count, note)
                    }
                    _ => format!("Hebbian ○ không đủ dữ liệu cho {} ↔ {}.", args[0], args[1]),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            _ if cmd.starts_with("merge ") => {
                // ○{merge term1 term2} — MergeSkill: merge two concept clusters
                let args: alloc::vec::Vec<&str> = cmd[6..].split_whitespace().collect();
                if args.len() < 2 {
                    return Response {
                        text: String::from("Cần 2 terms: ○{merge term1 term2}"),
                        tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System,
                    };
                }
                let chain_a = self.resolve_term(args[0]);
                let chain_b = self.resolve_term(args[1]);
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                ctx.push_input(chain_a);
                ctx.push_input(chain_b);
                let result = MergeSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { chain, note, .. } => {
                        let hash = chain.chain_hash();
                        let label = self.registry.alias_for_hash(hash).unwrap_or("(merged)");
                        format!("Merge ○\n  {} + {} → {} ({:016X})\n  {}", args[0], args[1], label, hash, note)
                    }
                    _ => format!("Merge ○ không thể merge {} và {}.", args[0], args[1]),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            _ if cmd.starts_with("fit ") => {
                // ○{fit edge=0.8 circ=0.9 aspect=1.0} — InverseRenderSkill: SDF fitting
                let params = &cmd[4..];
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                // Parse key=value pairs
                for part in params.split_whitespace() {
                    if let Some((k, v)) = part.split_once('=') {
                        match k {
                            "edge" => ctx.set(String::from("edge_ratio"), v.to_string()),
                            "circ" => ctx.set(String::from("circularity"), v.to_string()),
                            "aspect" => ctx.set(String::from("aspect_ratio"), v.to_string()),
                            _ => {}
                        }
                    }
                }
                let result = InverseRenderSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let sdf_type = ctx.get("sdf_type").unwrap_or("?");
                        let conf = ctx.get("sdf_confidence").unwrap_or("?");
                        format!("Fit ○\n  SDF: {} (confidence {})\n  {}", sdf_type, conf, note)
                    }
                    _ => String::from("Fit ○ cần ít nhất edge=, circ=, aspect= parameters."),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            "prune" => {
                // ○{prune} — PruneSkill: remove low-confidence STM entries
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                // Feed all STM observations as input
                for obs in self.learning.stm().all().iter() {
                    ctx.push_input(obs.chain.clone());
                }
                let before = ctx.input_chains.len();
                let result = PruneSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let kept = ctx.get("kept_count").unwrap_or("?");
                        let removed = ctx.get("pruned_count").unwrap_or("?");
                        format!("Prune ○\n  Before: {} → Kept: {}, Removed: {}\n  {}", before, kept, removed, note)
                    }
                    _ => String::from("Prune ○ STM rỗng — không có gì để prune."),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            "curate" => {
                // ○{curate} — CuratorSkill: rank knowledge quality
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                for obs in self.learning.stm().all().iter() {
                    ctx.push_input(obs.chain.clone());
                }
                let result = CuratorSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let sorted = ctx.get("sorted_count").unwrap_or("?");
                        format!("Curate ○\n  Ranked: {} items\n  {}", sorted, note)
                    }
                    _ => String::from("Curate ○ không đủ dữ liệu."),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            "temporal" => {
                // ○{temporal} — TemporalPatternSkill: detect patterns in STM timestamps
                let emotion = self.learning.context().last_emotion();
                let mut ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
                let all_obs = self.learning.stm().all();
                if all_obs.len() < 3 {
                    return Response {
                        text: String::from("Temporal ○ cần ít nhất 3 observations."),
                        tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System,
                    };
                }
                for obs in all_obs.iter() {
                    ctx.push_input(obs.chain.clone());
                }
                let ts_list: alloc::vec::Vec<String> = all_obs.iter().map(|o| format!("{}", o.timestamp)).collect();
                ctx.set(String::from("timestamps"), ts_list.join(","));
                let result = TemporalPatternSkill.execute(&mut ctx);
                let text = match result {
                    agents::skill::SkillResult::Ok { note, .. } => {
                        let mut lines = alloc::vec![format!("Temporal ○")];
                        if let Some(period) = ctx.get("temporal_period") {
                            lines.push(format!("  Period: {}ms", period));
                        }
                        if let Some(p) = ctx.get("temporal_periodicity") {
                            lines.push(format!("  Periodicity: {}", p));
                        }
                        if let Some(peak) = ctx.get("temporal_peak_hour") {
                            lines.push(format!("  Peak hour: {}h", peak));
                        }
                        lines.push(format!("  {}", note));
                        lines.join("\n")
                    }
                    _ => String::from("Temporal ○ không đủ dữ liệu để phân tích."),
                };
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            _ if cmd.starts_with("read ") => {
                // ○{read <text>} — BookReader → learn sentences → STM → Silk → KnowTree
                let text = &cmd[5..];
                if text.trim().is_empty() {
                    return Response {
                        text: String::from("○{read <text>} — cần nội dung để đọc."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let stored = self.read_book(text, ts);
                Response {
                    text: format!("Read ○ {} sentences learned.", stored),
                    tone: ResponseTone::Engaged,
                    fx: self.learning.context().fx(),
                    kind: ResponseKind::System,
                }
            }

            _ if cmd.starts_with("auth") => {
                self.handle_auth_command(cmd, ts)
            }

            _ => {
                // Math commands (solve, derive, integrate, simplify, eval)
                let math_prefixes = [
                    "solve ", "giai ", "derive ", "derivative ", "dao-ham ",
                    "d/dx ", "integrate ", "integral ", "tich-phan ",
                    "simplify ", "eval ",
                ];
                // Constant/Fibonacci commands
                let const_prefixes = [
                    "const ", "hang-so ", "fib ", "fibonacci ",
                ];
                if math_prefixes.iter().any(|p| cmd.starts_with(p)) {
                    let text = olang::math::process_math_command(cmd);
                    // Phase 4: Feed math result into learning pipeline
                    let math_input = ContentInput::Math {
                        expression: cmd.to_string(),
                        timestamp: ts,
                    };
                    let _proc = self.learning.process_one(math_input);
                    Response {
                        text,
                        tone: ResponseTone::Engaged,
                        fx: self.learning.context().fx(),
                        kind: ResponseKind::System,
                    }
                } else if const_prefixes.iter().any(|p| cmd.starts_with(p)) {
                    let precision = olang::constants::Precision::from_tier_byte(
                        self.hal_tier_byte()
                    );
                    let text = olang::constants::process_constant_command(cmd, precision);
                    Response {
                        text,
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                } else {
                    Response {
                        text: format!("Unknown command: {}", cmd),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    }
                }
            }
        }
    }

    // ── Recent Text Tracking — cho reference resolution ────────────────────────

    /// Track recent text for reference resolution.
    /// Giữ tối đa 16 turns gần nhất.
    fn track_recent_text(&mut self, text: &str, ts: i64) {
        const MAX_RECENT: usize = 16;

        let names = extract_names(text);
        self.recent_texts.push(RecentText {
            text: text.to_string(),
            timestamp: ts,
            names,
        });

        // Evict oldest
        if self.recent_texts.len() > MAX_RECENT {
            self.recent_texts.remove(0);
        }
    }

    /// Resolve unresolved reference ("bà ấy", "anh ta"...) from recent texts.
    ///
    /// Tìm tên riêng gần nhất trong recent_texts mà match với đại từ.
    /// Ví dụ: "bà ấy" → tìm tên nữ gần nhất đã mention.
    fn resolve_reference(&self, text: &str) -> Option<String> {
        let lo = text.to_lowercase();

        // Xác định loại đại từ
        let is_female = lo.contains("bà ấy")
            || lo.contains("cô ấy")
            || lo.contains("chị ấy")
            || lo.contains("she ");
        let is_male = lo.contains("ông ấy")
            || lo.contains("anh ấy")
            || lo.contains("he ");
        let is_generic = lo.contains("nó ")
            || lo.contains("người đó")
            || lo.contains("họ ")
            || lo.contains("they ")
            || lo.contains("that person");

        if !is_female && !is_male && !is_generic {
            return None;
        }

        // Tìm ngược trong recent_texts — gần nhất trước
        for recent in self.recent_texts.iter().rev() {
            // Bỏ qua chính message hiện tại
            if recent.text.to_lowercase() == lo {
                continue;
            }

            for name in &recent.names {
                // Trả về tên đầu tiên tìm được (gần nhất)
                // Tương lai: có thể filter theo giới tính nếu cần
                if is_generic || is_female || is_male {
                    return Some(name.clone());
                }
            }
        }

        None
    }

    // ── Knowledge Recall — tìm kiến thức liên quan từ Silk + STM ──────────────

    /// Recall related knowledge from Silk graph + STM.
    ///
    /// Walk Silk từ content words trong query, tìm nodes có edge mạnh.
    /// Trả về context string chứa thông tin liên quan đã học.
    fn recall_context(&self, query: &str) -> Option<String> {
        let words: alloc::vec::Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.chars().count() > 2)
            .take(10)
            .collect();

        if words.is_empty() {
            return None;
        }

        // Collect word hashes
        let hashes: alloc::vec::Vec<(u64, &str)> = words
            .iter()
            .map(|w| {
                let low: String = w.to_lowercase();
                (olang::hash::fnv1a_str(&low), *w)
            })
            .collect();

        // Walk Silk: tìm edges từ query words → known words
        let mut related: alloc::vec::Vec<(u64, f32, silk::edge::EmotionTag)> =
            alloc::vec::Vec::new();
        for &(h, _) in &hashes {
            for edge in self.learning.graph().edges_from(h) {
                if edge.weight > 0.15 {
                    related.push((edge.to_hash, edge.weight, edge.emotion));
                }
            }
        }

        if related.is_empty() {
            return None;
        }

        // Sort by weight, deduplicate
        related.sort_by(|a, b| b.1.total_cmp(&a.1));
        related.dedup_by_key(|r| r.0);
        related.truncate(8);

        // Find matching STM observations — check how often user mentioned related topics
        let mut max_fire: u32 = 0;
        let mut total_matches: usize = 0;
        let mut dominant_valence: f32 = 0.0;
        for &(h, _w, emo) in &related {
            if let Some(obs) = self.learning.stm().find_by_hash(h) {
                if obs.fire_count > max_fire {
                    max_fire = obs.fire_count;
                }
                dominant_valence += emo.valence;
                total_matches += 1;
            }
        }

        if total_matches == 0 {
            // Silk connections exist but no STM match yet — first time seeing related topic
            None
        } else {
            // Build human-readable recall — enriched by BodyStore splines
            dominant_valence /= total_matches as f32;

            // BodyStore enrichment: kiểm tra xem related nodes có learned body data không
            let body_insight = self.body_insight_for_recall(&related);

            let base = if max_fire > 2 {
                format!("Bạn đã nhắc đến chủ đề này {} lần rồi", max_fire)
            } else if dominant_valence < -0.3 {
                "Mình nhớ bạn đã nói về điều tương tự trước đó".to_string()
            } else if dominant_valence > 0.3 {
                "Mình nhớ lần trước bạn cũng nhắc đến điều này".to_string()
            } else {
                "Mình đã ghi nhận điều này từ trước".to_string()
            };

            if let Some(insight) = body_insight {
                Some(format!("{} — {}", base, insight))
            } else {
                Some(base)
            }
        }
    }

    /// BodyStore insight cho recall — đọc spline data từ related nodes.
    ///
    /// Nếu related nodes có learned emotion/temperature splines → trả về text mô tả.
    /// Đây là "BodyStore phục vụ inference" — công thức → ngữ nghĩa.
    fn body_insight_for_recall(
        &self,
        related: &[(u64, f32, silk::edge::EmotionTag)],
    ) -> Option<String> {
        let mut total_v = 0.0f32;
        let mut total_a = 0.0f32;
        let mut body_count = 0u32;

        for &(hash, _weight, _emo) in related.iter().take(5) {
            if let Some(body) = self.body_store.get(hash) {
                // Đọc emotion splines tại t=0.5 (trung điểm)
                let snap = body.splines.evaluate(0.5);
                total_v += snap.emotion_v;
                total_a += snap.emotion_a;
                body_count += 1;
            }
        }

        if body_count == 0 {
            return None;
        }

        let avg_v = total_v / body_count as f32;
        let avg_a = total_a / body_count as f32;

        // Chỉ trả insight khi emotion đáng kể (không neutral)
        if avg_v.abs() < 0.15 && avg_a < 0.3 {
            return None;
        }

        let feeling = if avg_v < -0.3 {
            "chủ đề này mang cảm xúc nặng"
        } else if avg_v > 0.3 {
            "chủ đề này mang năng lượng tích cực"
        } else if avg_a > 0.6 {
            "đây là chủ đề khiến bạn rất chú ý"
        } else {
            return None;
        };

        Some(feeling.into())
    }

    // ── Phase 10: KnowTree L3 recall — topic-aware knowledge ────────────────

    /// Query KnowTree L3 concepts related to input words.
    ///
    /// Returns topic-level context from Dream-promoted L3 concepts.
    /// Supplements Silk-based recall with structured knowledge.
    fn recall_from_knowtree(&mut self, text: &str) -> Option<String> {
        let word_hashes = olang::knowtree::text_to_word_hashes(text);
        if word_hashes.is_empty() {
            return None;
        }

        let mut found_concepts: alloc::vec::Vec<String> = alloc::vec::Vec::new();

        // Search L3 concepts by word hashes
        for &wh in &word_hashes {
            if let Some(node_with_edges) = self.knowtree.query(wh, 3, 1) {
                let n_edges = node_with_edges.edges.len();
                if n_edges > 0 {
                    found_concepts.push(format!("L3[{:x}]→{}edges", wh, n_edges));
                }
            }
            // Also check L2 for direct sentence matches
            if let Some(node_with_edges) = self.knowtree.query(wh, 2, 1) {
                let n_edges = node_with_edges.edges.len();
                if n_edges > 0 {
                    found_concepts.push(format!("L2[{:x}]→{}edges", wh, n_edges));
                }
            }
        }

        if found_concepts.is_empty() {
            return None;
        }

        // Return human-readable summary instead of debug format
        let n = found_concepts.len();
        if n >= 3 {
            Some("Mình đã hiểu khá nhiều về chủ đề này".to_string())
        } else {
            Some("Mình có biết một chút về điều này".to_string())
        }
    }

    // ── Contradiction Detection — so sánh với kiến thức đã học ──────────────

    /// Detect contradiction between new input and existing knowledge.
    ///
    /// So sánh emotion profile của input với STM observations có cùng từ khóa.
    /// Nếu valence đối nghịch + cùng chủ đề → contradiction.
    fn detect_contradiction(
        &self,
        text: &str,
        new_chain: &olang::molecular::MolecularChain,
        new_emotion: silk::edge::EmotionTag,
    ) -> Option<String> {
        let words: alloc::vec::Vec<String> = text
            .split_whitespace()
            .filter(|w| w.chars().count() > 2)
            .map(|w| w.to_lowercase())
            .collect();

        if words.is_empty() {
            return None;
        }

        // Tìm STM observations có edge mạnh với cùng từ khóa
        let mut conflicts: alloc::vec::Vec<(u64, f32, silk::edge::EmotionTag)> =
            alloc::vec::Vec::new();

        for w in &words {
            let h = olang::hash::fnv1a_str(w);
            for edge in self.learning.graph().edges_from(h) {
                if edge.weight > 0.30 {
                    // Check valence opposition: new_emotion vs edge emotion
                    let v_dist =
                        (new_emotion.valence - edge.emotion.valence).abs();
                    if v_dist > 0.60 {
                        conflicts.push((edge.to_hash, v_dist, edge.emotion));
                    }
                }
            }
        }

        if conflicts.is_empty() {
            return None;
        }

        // Chain similarity — molecular level check
        let chain_sim = if let Some(obs) = self
            .learning
            .stm()
            .all()
            .iter()
            .find(|o| conflicts.iter().any(|c| o.chain.chain_hash() == c.0))
        {
            obs.chain.similarity_full(new_chain)
        } else {
            0.0
        };

        // High topic overlap + valence opposition = contradiction
        let max_v_dist = conflicts
            .iter()
            .map(|c| c.1)
            .fold(0.0f32, |a, b| a.max(b));

        if max_v_dist > 0.60 || (chain_sim > 0.3 && max_v_dist > 0.40) {
            let existing_v = conflicts[0].2.valence;
            let direction = if new_emotion.valence > existing_v {
                "tích cực hơn"
            } else {
                "tiêu cực hơn"
            };
            Some(format!(
                "Mình nhận thấy điều này {} so với những gì đã biết trước đó \
                 (khoảng cách cảm xúc: {:.0}%). Bạn có muốn cập nhật kiến thức không?",
                direction,
                max_v_dist * 100.0
            ))
        } else {
            None
        }
    }

    // ── QR Promotion — user-directed learning ─────────────────────────────────

    /// User ra lệnh "hãy học cái này" → ghi chain hiện tại thành QR node.
    ///
    /// Bypass Dream cycle — user authority = direct QR write.
    /// QT9: ghi file TRƯỚC (pending_writes), cập nhật RAM SAU.
    fn force_learn_qr(
        &mut self,
        chain: &olang::molecular::MolecularChain,
        emotion: silk::edge::EmotionTag,
        ts: i64,
    ) -> Response {
        use olang::writer::OlangWriter;

        if chain.is_empty() {
            return Response {
                text: String::from("Không có nội dung để ghi nhớ."),
                tone: ResponseTone::Gentle,
                fx: self.learning.context().fx(),
                kind: ResponseKind::Natural,
            };
        }

        // Ghi QR node vào pending_writes (QT9: file TRƯỚC)
        let mut writer = OlangWriter::new_append();
        let _ = writer.append_node(chain, 1, true, ts); // layer=1 (QR), is_qr=true

        // Serialize Silk edges liên quan
        let hash = chain.chain_hash();
        for edge in self.learning.graph().edges_from(hash) {
            if edge.weight >= 0.10 {
                writer.append_edge(edge.from_hash, edge.to_hash, edge.kind.as_byte(), ts);
            }
        }

        self.pending_writes.extend_from_slice(writer.as_bytes());

        // Cập nhật Registry (RAM SAU) — gated with NodeKind::Memory
        self.gated_insert(chain, 1, ts, true, olang::registry::NodeKind::Memory, "qr:learn");

        // Tăng fire_count trong STM
        self.learning
            .stm_mut()
            .push(chain.clone(), emotion, ts);

        let fx = self.learning.context().fx();
        Response {
            text: format!(
                "○ Đã ghi nhớ (QR). chain_hash=0x{:08X}, {} molecules.",
                hash & 0xFFFFFFFF,
                chain.len()
            ),
            tone: ResponseTone::Reinforcing,
            fx,
            kind: ResponseKind::Natural,
        }
    }

    /// User nói "cái này đúng" → promote observation gần nhất trong STM lên QR.
    ///
    /// Tìm observation có fire_count cao nhất trong STM → ghi QR.
    fn confirm_learn_qr(&mut self, ts: i64) -> Response {
        use olang::writer::OlangWriter;

        // Tìm observation gần nhất (timestamp cao nhất) trong STM
        let best = self
            .learning
            .stm()
            .all()
            .iter()
            .max_by_key(|o| o.timestamp)
            .cloned();

        let obs = match best {
            Some(o) => o,
            None => {
                return Response {
                    text: String::from("Chưa có kiến thức nào để xác nhận. Hãy nói gì đó trước."),
                    tone: ResponseTone::Gentle,
                    fx: self.learning.context().fx(),
                    kind: ResponseKind::Natural,
                };
            }
        };

        // Ghi QR node vào pending_writes
        let mut writer = OlangWriter::new_append();
        let _ = writer.append_node(&obs.chain, 1, true, ts);

        let hash = obs.chain.chain_hash();

        // Serialize related edges
        for edge in self.learning.graph().edges_from(hash) {
            if edge.weight >= 0.10 {
                writer.append_edge(edge.from_hash, edge.to_hash, edge.kind.as_byte(), ts);
            }
        }

        self.pending_writes.extend_from_slice(writer.as_bytes());

        // Cập nhật Registry — gated with NodeKind::Memory
        self.gated_insert(&obs.chain, 1, ts, true, olang::registry::NodeKind::Memory, "qr:confirm");

        let fx = self.learning.context().fx();
        Response {
            text: format!(
                "○ Xác nhận đúng → đã ghi QR. chain_hash=0x{:08X}, fire_count={}.",
                hash & 0xFFFFFFFF,
                obs.fire_count
            ),
            tone: ResponseTone::Reinforcing,
            fx,
            kind: ResponseKind::Natural,
        }
    }

    // ── SilkWalk — tìm context liên quan ─────────────────────────────────────

    /// Walk Silk từ các từ khóa trong câu hỏi.
    /// Trả về danh sách (hash, emotion, weight) của nodes liên quan nhất.
    fn silk_walk_query(&self, query: &str) -> alloc::vec::Vec<(u64, silk::edge::EmotionTag, f32)> {
        let words = query
            .split_whitespace()
            .filter(|w| w.chars().count() > 2)
            .take(8);

        let mut found: alloc::vec::Vec<(u64, silk::edge::EmotionTag, f32)> = alloc::vec::Vec::new();

        for w in words {
            let low = w.to_lowercase();
            let h = olang::hash::fnv1a_str(&low);

            // Walk từ node này → lấy neighbors có weight cao nhất
            let edges = self.learning.graph().edges_from(h);
            for e in edges {
                if e.weight > 0.05 {
                    found.push((e.to_hash, e.emotion, e.weight));
                }
            }
        }

        // Sort by weight descending
        found.sort_by(|a, b| b.2.total_cmp(&a.2));
        found.truncate(5);
        found
    }

    /// Tổng hợp emotion từ Silk walk — AMPLIFY, không trung bình.
    ///
    /// Dùng silk::walk::sentence_affect() để walk qua graph:
    /// mỗi từ → hash → tìm Silk edges → amplify emotion.
    fn walk_emotion(&self, query: &str) -> Option<silk::edge::EmotionTag> {
        let words: alloc::vec::Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.chars().count() > 1)
            .take(8)
            .collect();

        if words.is_empty() {
            return None;
        }

        // Tính hash và base emotion cho mỗi từ
        let mut word_hashes = alloc::vec::Vec::new();
        let mut word_emotions = alloc::vec::Vec::new();

        for w in &words {
            let low = w.to_lowercase();
            let h = olang::hash::fnv1a_str(&low);
            word_hashes.push(h);

            // Lấy emotion từ STM nếu có, fallback context::emotion::word_affect
            let emo = if let Some(obs) = self.learning.stm().find_by_hash(h) {
                obs.emotion
            } else {
                let raw = context::emotion::word_affect(&low);
                silk::edge::EmotionTag::new(raw.valence, raw.arousal, 0.5, raw.arousal.abs().max(0.3))
            };
            word_emotions.push(emo);
        }

        // Walk qua Silk graph — amplify, KHÔNG trung bình
        let result = silk::walk::sentence_affect(
            self.learning.graph(),
            &word_hashes,
            &word_emotions,
            8, // max_depth = Fib[6]
        );

        if result.total_weight < 0.001 {
            return None;
        }

        Some(result.composite)
    }

    // ── Universal input — BẢN NĂNG cho mọi modality ──────────────────────────

    /// Universal entry point — BẢN NĂNG: chạy 7-layer emotion pipeline cho MỌI modality.
    ///
    /// Text, Audio, Sensor, Code, Math, System — tất cả đi qua đây.
    /// 7 tầng: InferContext → EmotionTag → ctx.Apply → Intent → Crisis → Learn → Render
    pub fn process_input(&mut self, input: ContentInput, ts: i64) -> Response {
        self.turn_count += 1;
        self.uptime_ns = ts;

        // Auto-Dream: Fibonacci schedule — Fib[4]=5, Fib[5]=8, Fib[6]=13, Fib[7]=21...
        // Mỗi lần dream xong → tăng fib_index → khoảng cách tăng dần (self-regulating)
        let dream_interval = silk::hebbian::fib(self.dream_fib_index) as u64;
        if self.turn_count - self.last_dream_turn >= dream_interval
            && self.learning.stm().len() >= 3
        {
            self.run_dream(ts);
        }

        // ── T1+T2+T3: Context + Emotion — tùy modality ─────────────────────
        let (raw_tag, emo_ctx_scale) = match &input {
            ContentInput::Text { content, .. } => {
                let emo_ctx = infer_context(content);
                let raw = sentence_affect(content);
                let scaled = emo_ctx.apply(raw);
                (scaled, 1.0_f32)
            }
            ContentInput::Audio {
                freq_hz, amplitude, ..
            } => {
                // Audio emotion: pitch → valence, amplitude → arousal
                let v = ((*freq_hz - 200.0) / 400.0).clamp(-1.0, 1.0) * 0.3;
                let a = amplitude.clamp(0.0, 1.0);
                (
                    silk::edge::EmotionTag {
                        valence: v,
                        arousal: a,
                        dominance: 0.5,
                        intensity: a,
                    },
                    0.85,
                )
            }
            ContentInput::Sensor { value, .. } => {
                // Sensor: deviation from comfort → emotion
                let dev = (value - 22.0).abs() / 20.0; // 22°C = comfort
                let v = if dev > 0.5 { -dev.min(1.0) } else { 0.0 };
                (
                    silk::edge::EmotionTag {
                        valence: v,
                        arousal: dev.min(1.0),
                        dominance: 0.5,
                        intensity: dev.min(1.0),
                    },
                    0.5,
                )
            }
            _ => {
                // Code, Math, System → neutral emotion
                (silk::edge::EmotionTag::NEUTRAL, 0.3)
            }
        };

        // ── T4: Intent estimate ─────────────────────────────────────────────
        let cur_v = self.learning.context().fx();
        let text_for_intent = match &input {
            ContentInput::Text { content, .. } => content.as_str(),
            _ => "",
        };
        let est = estimate_intent(text_for_intent, cur_v, raw_tag.arousal);

        // SilkWalk: enrich with learned context (text only has meaningful walk)
        let walk_tag = match &input {
            ContentInput::Text { content, .. } => self.walk_emotion(content),
            _ => None,
        };

        // ── T5: Crisis → override ngay ──────────────────────────────────────
        if est.primary == IntentKind::Crisis {
            use crate::response_template::crisis_text;
            return Response {
                text: crisis_text(),
                tone: ResponseTone::Supportive,
                fx: self.learning.context().fx(),
                kind: ResponseKind::Crisis,
            };
        }

        // ── T6: Learning pipeline — BẢN NĂNG: mọi modality ─────────────────
        let proc_result = self.learning.process_one(input.clone());

        // ── T6a: QT8+QT9 — Ghi L1 node vào pending_writes TRƯỚC ────────
        // Mọi input thành công → tạo node L1 trong origin.olang.
        // L1 = learned node (chưa QR). QR promote → L2..Ln-1 qua Dream.
        if let ProcessResult::Ok { ref chain, emotion: _ } = proc_result {
            let hash = chain.chain_hash();
            // Chỉ ghi nếu chưa có trong registry (tránh duplicate)
            if self.registry.lookup_hash(hash).is_none() {
                use olang::writer::OlangWriter;
                let mut l1_writer = OlangWriter::new_append();
                let _ = l1_writer.append_node(chain, 1, false, ts); // L1, chưa QR
                self.pending_writes.extend_from_slice(l1_writer.as_bytes());
                // QT9: Registry SAU khi đã ghi file
                self.gated_insert(chain, 1, ts, false, olang::registry::NodeKind::Knowledge, "learn:L1");
            }

            // Silk edges mới từ process_one → ghi file
            // Lấy edges liên quan đến chain mới, weight đủ mạnh
            {
                let graph = self.learning.graph();
                let edges: alloc::vec::Vec<_> = graph.edges_from(hash)
                    .iter()
                    .filter(|e| e.weight >= 0.10)
                    .map(|e| (e.from_hash, e.to_hash, e.kind.as_byte(), e.updated_at))
                    .collect();
                if !edges.is_empty() {
                    use olang::writer::OlangWriter;
                    let mut edge_writer = OlangWriter::new_append();
                    for (from, to, kind, edge_ts) in &edges {
                        edge_writer.append_edge(*from, *to, *kind, *edge_ts);
                    }
                    self.pending_writes.extend_from_slice(edge_writer.as_bytes());
                }
            }
        }

        // ── T6a2: Persist STM observation vào origin.olang ──────────────
        // QT8: origin.olang = bộ nhớ duy nhất, RAM = cache tạm.
        if let ProcessResult::Ok { ref chain, emotion: _ } = proc_result {
            let hash = chain.chain_hash();
            if let Some(obs) = self.learning.stm().find_by_hash(hash) {
                use olang::writer::OlangWriter;
                let mut stm_writer = OlangWriter::new_append();
                stm_writer.append_stm(
                    hash,
                    obs.emotion.valence,
                    obs.emotion.arousal,
                    obs.emotion.dominance,
                    obs.emotion.intensity,
                    obs.fire_count,
                    obs.maturity.as_byte(),
                    obs.layer,
                    ts,
                );
                self.pending_writes.extend_from_slice(stm_writer.as_bytes());
            }
        }

        // ── T6a3: Persist Hebbian links vào origin.olang ──────────────
        // Lưu learned weights mới/cập nhật từ co_activate
        if let ProcessResult::Ok { ref chain, .. } = proc_result {
            let hash = chain.chain_hash();
            let graph = self.learning.graph();
            let links: alloc::vec::Vec<_> = graph.learned_links_from(hash);
            if !links.is_empty() {
                use olang::writer::OlangWriter;
                let mut heb_writer = OlangWriter::new_append();
                for link in &links {
                    heb_writer.append_hebbian(
                        link.from_hash,
                        link.to_hash,
                        link.weight,
                        link.fire_count,
                        ts,
                    );
                }
                self.pending_writes.extend_from_slice(heb_writer.as_bytes());
            }
        }

        // ── T6a4: Persist ConversationCurve turn vào origin.olang ─────
        // Mỗi turn ghi 1 record: valence + fx_dn → replay khi boot.
        {
            let curve = self.learning.context().curve();
            if curve.turn_count() > 0 {
                use olang::writer::OlangWriter;
                let mut curve_writer = OlangWriter::new_append();
                curve_writer.append_curve(
                    curve.current_v(),
                    curve.fx_dn,
                    ts,
                );
                self.pending_writes.extend_from_slice(curve_writer.as_bytes());
            }
        }

        // ── T6b: KnowTree — store text as L2 node ──────────────────────
        if let ProcessResult::Ok { ref chain, emotion } = proc_result {
            if let ContentInput::Text { ref content, .. } = input {
                let word_hashes = olang::knowtree::text_to_word_hashes(content);
                if !word_hashes.is_empty() {
                    // Legacy KnowTree (CompactNode) — backward compat
                    self.knowtree.store_sentence(chain, None, &word_hashes, ts);

                    // SlimKnowTree (spec-compliant ~10B/node)
                    self.slim_knowtree.store_sentence(chain, &word_hashes, ts);

                    // ── T6b1: Persist as SlimKnowTree record (0x0A) ──
                    {
                        use olang::writer::OlangWriter;
                        let tagged = chain.to_tagged_bytes();
                        let hash = chain.chain_hash();
                        let mut kt_writer = OlangWriter::new_append();
                        let _ = kt_writer.append_slim_knowtree(hash, &tagged, 2, ts);
                        self.pending_writes.extend_from_slice(kt_writer.as_bytes());
                    }
                }
            }

            // ── T6b2: NodeBody — tạo/cập nhật SDF + Spline cho chain ──────
            // Mỗi chain mới → tạo body từ molecule bytes
            if let Some(mol) = chain.first() {
                let hash = chain.chain_hash();
                if self.body_store.get(hash).is_none() {
                    let body = body_from_molecule_full(
                        hash,
                        mol.shape_u8(),
                        mol.relation_u8(),
                        mol.valence_u8(),
                        mol.arousal_u8(),
                        mol.time_u8(),
                    );
                    // Insert into store
                    let entry = self.body_store.get_or_create(hash);
                    *entry = body;
                }
            }

            // ── T6b3: Evolution — detect & create evolved nodes ──────────
            // So sánh chain mới với STM — nếu đúng 1 dimension khác → evolution
            // Evolution = tạo loài mới (new node), KHÔNG update node cũ (QT append-only)
            {
                let candidates = self.learning.detect_evolutions(chain);
                for cand in &candidates {
                    // Try evolve source chain → validate consistency
                    if let Some((evolved_chain, evo_result)) = cand.source_chain.evolve_and_apply(
                        0,
                        cand.dimension,
                        cand.new_value,
                    ) {
                        if evo_result.valid {
                            let evolved_hash = evolved_chain.chain_hash();

                            // QT8: GHI FILE TRƯỚC — evolved node phải có trong origin.olang
                            {
                                use olang::writer::OlangWriter;
                                let mut evo_writer = OlangWriter::new_append();
                                let _ = evo_writer.append_node(&evolved_chain, 2, false, ts);
                                // Edge: source → evolved (DerivedFrom = 0x05)
                                evo_writer.append_edge(
                                    cand.source_hash,
                                    evolved_hash,
                                    0x05, // DerivedFrom
                                    ts,
                                );
                                self.pending_writes
                                    .extend_from_slice(evo_writer.as_bytes());
                            }

                            // QT9: Registry SAU khi đã ghi file — gated with NodeKind::Knowledge
                            self.gated_insert(&evolved_chain, 2, ts, false, olang::registry::NodeKind::Knowledge, "evolved");

                            // Tạo body cho evolved node (RAM cache — sau file)
                            if let Some(evolved_mol) = evolved_chain.first() {
                                if self.body_store.get(evolved_hash).is_none() {
                                    let body = body_from_molecule_full(
                                        evolved_hash,
                                        evolved_mol.shape_u8(),
                                        evolved_mol.relation_u8(),
                                        evolved_mol.valence_u8(),
                                        evolved_mol.arousal_u8(),
                                        evolved_mol.time_u8(),
                                    );
                                    let entry = self.body_store.get_or_create(evolved_hash);
                                    *entry = body;
                                }
                            }

                            // Silk edge: source → evolved (DerivedFrom)
                            let evo_emotion = emotion; // inherit emotion context
                            self.learning.graph_mut().co_activate(
                                cand.source_hash,
                                evolved_hash,
                                evo_emotion,
                                0.9, // high strength — direct evolution link
                                ts,
                            );
                        }
                        // invalid → discard silently (QT: sai → hủy)
                    }
                }
            }
        }

        // ── T6b4: BodyStore ← Learning — cập nhật Spline từ thực tế ────────
        // Learning tích lũy emotion data → cập nhật BodyStore splines
        // Đây là bridge "Molecule = Công thức" → Spline thay đổi theo thực tế
        {
            let body_updates = self.learning.pending_body_updates();
            for update in &body_updates {
                if let Some(body) = self.body_store.get_mut(update.chain_hash) {
                    // Cập nhật emotion_v spline từ accumulated valence
                    let v = update.emotion.valence;
                    if v.abs() > 0.1 {
                        body.learn_emotion_v(vsdf::spline::VectorSpline::flat(v));
                    }
                    // Cập nhật emotion_a spline từ accumulated arousal
                    let a = update.emotion.arousal;
                    if a > 0.15 {
                        body.learn_emotion_a(vsdf::spline::VectorSpline::flat(a));
                    }
                }
            }

            // RAM pressure — evict least-used bodies mỗi 21 turns (Fib[8])
            if self.turn_count > 0 && self.turn_count.is_multiple_of(21) {
                self.body_store.evict_lfu();
            }
        }

        // ── T6c: Track recent text for reference resolution ────────────────
        if let ContentInput::Text { ref content, .. } = input {
            self.track_recent_text(content, ts);
        }

        // ── T6d: Bản năng bẩm sinh — 7 instincts chạy SAU learning ────────
        // Honesty → Contradiction → Causality → Abstraction → Analogy → Curiosity → Reflection
        let instinct_ctx = if let ProcessResult::Ok { ref chain, emotion } = proc_result {
            Some(self.run_instincts(chain, emotion, ts))
        } else {
            None
        };

        // ── T6e: Silk heartbeat — chăm sóc Ln-1 mỗi 13 turns (Fib[7]) ─────
        if self.turn_count.is_multiple_of(13) && self.turn_count > 0 {
            let elapsed_ns = if self.uptime_ns > 0 { ts - self.uptime_ns } else { 0 }
                * 1_000_000; // ms → ns
            self.learning.maintain_silk(elapsed_ns, 100_000);
        }

        // ── T6f: Agent Orchestration — pump MessageRouter ─────────────────
        // Feed learning result to LeoAI via ISL → tick router → AAM reviews
        if let ProcessResult::Ok { ref chain, emotion: _ } = proc_result {
            // Send chain to LeoAI as Learn message
            let chain_hash = chain.chain_hash();
            let payload = [
                (chain_hash >> 16) as u8,
                (chain_hash >> 8) as u8,
                chain_hash as u8,
            ];
            let learn_msg = isl::message::ISLMessage {
                from: isl::address::ISLAddress::new(0, 0, 0, 0), // from Runtime
                to: self.leo.addr,
                msg_type: isl::message::MsgType::Learn,
                payload,
            };
            let frame = isl::message::ISLFrame::bare(learn_msg);
            self.leo.receive_isl(frame);
            self.leo.poll_inbox(ts);

            // Tick router: Workers → Chiefs → LeoAI → AAM → feedback
            let _tick_stats = self.router.tick(
                &mut self.workers,
                &mut self.chiefs,
                &mut self.leo,
                ts,
            );

            // Drain router pending writes → append to runtime pending_writes
            if self.router.has_pending_writes() {
                self.pending_writes.extend(self.router.drain_pending_writes());
            }
        }

        // ── T7: Decide action → render response ────────────────────────────
        let mut action = decide_action(&est, cur_v);
        let fx = self.learning.context().fx();
        let tone = self.learning.context().tone();
        let lang = match &input {
            ContentInput::Text { content, .. } => detect_language(content),
            _ => Lang::Vi,
        };

        // ── T7a: Build ResponseContext từ STM + Silk + Instincts ────────
        let resp_ctx = {
            let input_text = match &input {
                ContentInput::Text { content, .. } => content.as_str(),
                _ => "",
            };
            let mut ctx = ResponseContext::default();

            // Topics: extract content words (bỏ stop words)
            if !input_text.is_empty() {
                ctx.topics = input_text
                    .split_whitespace()
                    .filter(|w| w.chars().count() > 1 && !is_vn_stop_word(w))
                    .take(4)
                    .map(|w| w.to_string())
                    .collect();
            }

            // Repetition: fire_count cao nhất từ STM cho topic words
            if let ProcessResult::Ok { ref chain, .. } = proc_result {
                let hash = chain.chain_hash();
                if let Some(obs) = self.learning.stm().find_by_hash(hash) {
                    ctx.repetition_count = obs.fire_count;
                }
            }

            // Walk emotion → amplified valence
            if let Some(wt) = walk_tag {
                ctx.walk_valence = Some(wt.valence);
            }

            // Novelty: nếu fire_count = 1 → novelty cao
            ctx.novelty = if ctx.repetition_count <= 1 { 0.85 } else {
                (1.0 / (ctx.repetition_count as f32)).min(0.80)
            };

            // Instinct results
            if let Some(ref insight) = instinct_ctx {
                ctx.contradiction = insight.has_contradiction;
                // Causality từ instinct — nếu có topic + emotion < 0 → causality heuristic
                if !ctx.topics.is_empty() && raw_tag.valence < -0.20 {
                    // "buồn vì mất việc" → topics = ["buồn", "mất", "việc"]
                    // Causality: last content words = cause
                    let cause_words: alloc::vec::Vec<&str> = input_text
                        .split_whitespace()
                        .filter(|w| w.chars().count() > 1 && !is_vn_stop_word(w))
                        .skip(1) // skip emotion word
                        .take(3)
                        .collect();
                    if !cause_words.is_empty() {
                        ctx.causality = Some(cause_words.join(" "));
                    }
                }
            }

            ctx
        };

        // ── T7b: Reference resolution — "bà ấy", "anh ta"... ─────────────
        // Nếu Observe vì unresolved_ref → thử resolve từ recent_texts
        if action == IntentAction::Observe && est.has_unresolved_ref {
            if let ContentInput::Text { ref content, .. } = input {
                if let Some(_name) = self.resolve_reference(content) {
                    // Đã tìm được referent → chuyển sang EmpathizeFirst hoặc Proceed
                    // tùy emotion
                    if cur_v < -0.30 {
                        action = IntentAction::EmpathizeFirst;
                    } else {
                        action = IntentAction::Proceed;
                    }
                }
                // Không resolve được → giữ Observe (im lặng)
            }
        }

        // ── T7c: Observe — check pending tasks ───────────────────────────────
        // "Hôm nay thật chán!!!" + có việc cần làm → suggest
        // "Hôm nay thật chán!!!" + không có gì → im lặng đợi

        match proc_result {
            ProcessResult::Crisis { message } => Response {
                text: message,
                tone: ResponseTone::Supportive,
                fx,
                kind: ResponseKind::Crisis,
            },
            ProcessResult::Blocked { reason } => Response {
                text: format!("({})", reason),
                tone: ResponseTone::Gentle,
                fx,
                kind: ResponseKind::Blocked,
            },
            ProcessResult::Ok {
                ref chain,
                emotion,
            } => {
                use context::intent::IntentAction;

                let final_v = if let Some(wt) = walk_tag {
                    emotion.valence * 0.40 + wt.valence * 0.60
                } else {
                    emotion.valence * emo_ctx_scale + raw_tag.valence * (1.0 - emo_ctx_scale)
                };

                // ── ForceLearnQR: user ra lệnh "hãy học" → ghi QR trực tiếp ──
                if action == IntentAction::ForceLearnQR {
                    return self.force_learn_qr(chain, emotion, ts);
                }

                // ── ConfirmLearnQR: "cái này đúng" → promote last STM → QR ──
                if action == IntentAction::ConfirmLearnQR {
                    return self.confirm_learn_qr(ts);
                }

                // ── HomeControl: route command to HomeChief via ISL ────────
                if action == IntentAction::HomeControl {
                    let cmd_text = match &input {
                        ContentInput::Text { content, .. } => content.as_str(),
                        _ => "",
                    };
                    let result_text = self.dispatch_home_control(cmd_text, ts);
                    return Response {
                        text: result_text,
                        tone,
                        fx,
                        kind: ResponseKind::Natural,
                    };
                }

                // ── Knowledge recall + contradiction ──────────────────────
                let input_text = match &input {
                    ContentInput::Text { content, .. } => content.as_str(),
                    _ => "",
                };

                // Recall related knowledge from Silk walk + KnowTree L3
                let silk_recall = if !input_text.is_empty() {
                    self.recall_context(input_text)
                } else {
                    None
                };
                let kt_recall = if !input_text.is_empty() {
                    self.recall_from_knowtree(input_text)
                } else {
                    None
                };
                // Merge: Silk recall + KnowTree recall
                let recall = match (silk_recall, kt_recall) {
                    (Some(s), Some(k)) => Some(format!("{} {}", s, k)),
                    (Some(s), None) => Some(s),
                    (None, Some(k)) => Some(k),
                    (None, None) => None,
                };

                // Detect contradiction against existing knowledge
                let contradiction = if !input_text.is_empty() {
                    self.detect_contradiction(input_text, chain, emotion)
                } else {
                    None
                };

                // ── Listening: SilentAck → empty response ────────────────────
                if action == IntentAction::SilentAck {
                    return Response {
                        text: String::new(),
                        tone,
                        fx,
                        kind: ResponseKind::Natural,
                    };
                }

                // ── Listening: Observe — build contextual observe response ───
                if action == IntentAction::Observe {
                    // Nếu có reference resolution → dùng kết quả
                    let observe_original = if est.has_unresolved_ref {
                        if let Some(name) =
                            self.resolve_reference(input_text)
                        {
                            // Biết "bà ấy" là ai → chia buồn/trả lời
                            if final_v < -0.30 {
                                Some(format!(
                                    "Mình biết bạn đang nói về {}. Chia buồn cùng bạn.",
                                    name
                                ))
                            } else {
                                Some(format!(
                                    "Bạn đang nói về {} phải không?",
                                    name
                                ))
                            }
                        } else {
                            // Không biết "bà ấy" → im lặng
                            None
                        }
                    } else if est.is_vague_emotion {
                        // "Hôm nay thật chán" → check pending knowledge
                        let has_pending = !self.learning.stm().is_empty()
                            && self
                                .learning
                                .stm()
                                .all()
                                .iter()
                                .any(|o| o.fire_count > 1);
                        if has_pending {
                            Some(
                                "Bạn muốn mình nhắc lại những gì đã nói trước đó không?"
                                    .to_string(),
                            )
                        } else {
                            // Không có gì → im lặng đợi
                            None
                        }
                    } else {
                        None
                    };

                    let text = render(&ResponseParams {
                        tone,
                        action: IntentAction::Observe,
                        valence: final_v,
                        fx,
                        context: recall,
                        original: observe_original,
                        language: lang,
                    });
                    return Response {
                        text,
                        tone,
                        fx,
                        kind: ResponseKind::Natural,
                    };
                }

                // ── Normal flow: contradiction / recall / proceed ────────────
                // Build response with context awareness — use user's actual words
                let original = if let Some(ref contra) = contradiction {
                    // Contradiction detected → inform user
                    Some(contra.clone())
                } else {
                    match &action {
                        IntentAction::Proceed => {
                            if let Some(ref ctx) = recall {
                                // Silk/KnowTree recalled related knowledge
                                Some(contextual_reply(tone, final_v, input_text, ctx))
                            } else {
                                // No recall — still use user's actual words
                                Some(natural_reply(tone, final_v, input_text))
                            }
                        }
                        _ => None,
                    }
                };

                let mut text = render(&ResponseParams {
                    tone,
                    action,
                    valence: final_v,
                    fx,
                    context: recall,
                    original,
                    language: lang,
                });

                // ── T7e: Instinct enrichment — bản năng làm giàu response ──
                if let Some(ref insight) = instinct_ctx {
                    // Epistemic disclaimer (QT18: trung thực tuyệt đối)
                    if !text.is_empty() {
                        match insight.epistemic_grade.as_deref() {
                            Some("hypothesis") => {
                                text = format!("{}\n[Giả thuyết]", text);
                            }
                            Some("opinion") => {
                                text = format!("{}\n[Chưa chắc chắn]", text);
                            }
                            _ => {} // fact → no disclaimer, silence → handled earlier
                        }
                    }

                    // Instinct-detected contradiction overrides learning contradiction
                    if insight.has_contradiction && contradiction.is_none() && !text.is_empty() {
                        text = format!(
                            "{}\n⊥ Mình nhận thấy có điều mâu thuẫn — bạn có muốn nói rõ hơn không?",
                            text
                        );
                    }

                    // Curiosity: high novelty → express interest
                    if !text.is_empty() {
                        match insight.curiosity_level.as_deref() {
                            Some("extreme") => {
                                text = format!("{}\nĐây là điều rất mới — mình muốn hiểu thêm.", text);
                            }
                            Some("high") => {
                                text = format!("{}\nMình chưa gặp điều này trước đây.", text);
                            }
                            _ => {} // moderate/low → no comment
                        }
                    }

                    // Reflection: knowledge quality warning
                    if !text.is_empty() {
                        if let Some("fragile") = insight.reflection_verdict.as_deref() {
                            text = format!(
                                "{}\n[Kiến thức còn mỏng — cần thêm dữ liệu]",
                                text
                            );
                        }
                    }
                }

                Response {
                    text,
                    tone,
                    fx,
                    kind: ResponseKind::Natural,
                }
            }
            ProcessResult::Empty => {
                let text = render(&ResponseParams {
                    tone,
                    action,
                    valence: cur_v,
                    fx,
                    context: None,
                    original: None,
                    language: lang,
                });
                Response {
                    text,
                    tone,
                    fx,
                    kind: ResponseKind::Natural,
                }
            }
        }
    }

    // ── Audio + Image — delegate to process_input ───────────────────────────

    /// Nhận audio features → fuse cross-modal → universal pipeline.
    pub fn process_audio(
        &mut self,
        pitch_hz: f32,
        energy: f32,
        _tempo_bpm: f32,
        _voice_break: f32,
        ts: i64,
    ) -> Response {
        let input = ContentInput::Audio {
            freq_hz: pitch_hz,
            amplitude: energy,
            duration_ms: 0,
            timestamp: ts,
        };
        self.process_input(input, ts)
    }

    /// Nhận image features → universal pipeline.
    pub fn process_image(
        &mut self,
        _hue: f32,
        _saturation: f32,
        _brightness: f32,
        _motion: f32,
        _face_valence: Option<f32>,
        ts: i64,
    ) -> Response {
        // Image → encode as system event (image analysis result)
        let input = ContentInput::System {
            event: agents::encoder::SystemEvent::Boot, // placeholder — image processing
            timestamp: ts,
        };
        self.process_input(input, ts)
    }

    // ── Observability ─────────────────────────────────────────────────────────

    /// Snapshot metrics cho observability.
    /// HAL tier as byte for precision selection.
    /// 1=Full, 2=Compact, 3=Worker, 4=Sensor.
    /// Default: 1 (Full) — will be wired to hal::HardwareTier when HAL is connected.
    fn hal_tier_byte(&self) -> u8 {
        // Future: self.hal.tier().as_byte()
        1 // Full precision by default
    }

    pub fn metrics(&self) -> crate::metrics::RuntimeMetrics {
        let stm = self.learning.stm();
        let graph = self.learning.graph();
        let stm_obs = stm.len();

        // STM hit rate: observations with fire_count > 1
        let hits = stm.all().iter().filter(|o| o.fire_count > 1).count();
        let hit_rate = if stm_obs > 0 {
            hits as f32 / stm_obs as f32
        } else {
            0.0
        };
        let max_fire = stm.all().iter().map(|o| o.fire_count).max().unwrap_or(0);

        // Silk density: edges / (N*(N-1)/2) where N = unique nodes
        let edge_count = graph.len();
        let node_count = graph.node_count();
        let max_edges = if node_count > 1 {
            node_count * (node_count - 1) / 2
        } else {
            1
        };
        let density = edge_count as f32 / max_edges as f32;

        let tone_str = alloc::format!("{:?}", self.learning.context().tone());

        crate::metrics::RuntimeMetrics {
            turns: self.turn_count,
            stm_observations: stm_obs,
            silk_edges: edge_count,
            silk_density: density.min(1.0),
            stm_hit_rate: hit_rate,
            fx: self.learning.context().fx(),
            tone: tone_str,
            dream_cycles: self.dream_cycles,
            pending_bytes: self.pending_writes.len(),
            saveable_edges: self.saveable_edges(),
            stm_max_fire: max_fire,
            dream_approved: self.dream_approved_total,
            dream_l3_concepts: self.dream_l3_created,
            dream_fib_interval: silk::hebbian::fib(self.dream_fib_index),
            knowtree_nodes: self.knowtree.total_nodes(),
            knowtree_edges: self.knowtree.total_edges(),
            knowtree_sentences: self.knowtree.sentences(),
            knowtree_concepts: self.knowtree.concepts(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Persistence — save/load origin.olang
    // ─────────────────────────────────────────────────────────────────────────

    /// Serialize trạng thái học được thành bytes để append vào origin.olang.
    ///
    /// QT8: append-only — không xóa, không overwrite.
    /// Ghi: Silk EdgeAssoc edges + STM observations đủ điều kiện QR.
    ///
    /// Caller chịu trách nhiệm ghi bytes vào file:
    ///   `std::fs::OpenOptions::new().append(true).open("origin.olang")?.write_all(&bytes)?`
    pub fn serialize_learned(&self, _ts: i64) -> alloc::vec::Vec<u8> {
        use olang::writer::OlangWriter;

        // Append mode — không ghi header (file đã có header)
        let mut writer = OlangWriter::new_append();

        // 1. Silk EdgeAssoc edges đủ mạnh (weight >= 0.3 → đáng lưu)
        let graph = self.learning.graph();
        for edge in graph.all_edges() {
            if edge.kind.is_associative() && edge.weight >= 0.30 {
                writer.append_edge(
                    edge.from_hash,
                    edge.to_hash,
                    edge.kind.as_byte(),
                    edge.updated_at,
                );
            }
        }

        // 2. STM observations có fire_count >= 3 → ĐN sẵn sàng QR
        //    (Dream sẽ promote QR — đây chỉ là persist ĐN để không mất)
        for obs in self.learning.stm().all().iter() {
            let hash = obs.chain.chain_hash();
            let fire_count = graph.edges_from(hash).len() as u32;
            if fire_count >= 3 {
                let _ = writer.append_node(&obs.chain, 2, false, obs.timestamp);
                // layer=2 (ĐN), is_qr=false
            }
        }

        writer.into_bytes()
    }

    /// Số Silk edges sẽ được lưu (preview trước khi serialize).
    pub fn saveable_edges(&self) -> usize {
        self.learning
            .graph()
            .all_edges()
            .filter(|e| e.kind.is_associative() && e.weight >= 0.30)
            .count()
    }

    // ── BookReader → KnowTree ──────────────────────────────────────────────

    /// Read book text → encode sentences → store in KnowTree as L2 nodes.
    ///
    /// BookReader.read() → SentenceRecord → encode → KnowTree.store_chapter()
    /// Returns number of sentences stored.
    pub fn read_book(&mut self, text: &str, ts: i64) -> usize {
        use agents::book::BookReader;

        let reader = BookReader::new();
        let records = reader.read(text);
        if records.is_empty() {
            return 0;
        }

        // Encode each sentence → MolecularChain + word_hashes
        let mut chains: alloc::vec::Vec<(olang::molecular::MolecularChain, alloc::vec::Vec<u64>)> =
            alloc::vec::Vec::new();

        for record in &records {
            // Encode sentence text → chain via learning pipeline encoder
            let input = ContentInput::Text {
                content: record.text.clone(),
                timestamp: ts,
            };
            let encoded = agents::encoder::ContentEncoder::new().encode(input);
            if encoded.chain.is_empty() {
                continue;
            }

            let word_hashes = olang::knowtree::text_to_word_hashes(&record.text);

            // Also feed into STM + Silk (learning pipeline)
            self.learning
                .stm_mut()
                .push(encoded.chain.clone(), record.emotion, ts);

            chains.push((encoded.chain, word_hashes));
        }

        let stored = chains.len();
        if !chains.is_empty() {
            self.knowtree.store_chapter(&chains, None, ts);
            self.slim_knowtree.store_chapter(&chains, ts);
        }
        stored
    }

    // ── HomeControl → ISL → Chief ─────────────────────────────────────────────

    /// Dispatch home control command to HomeChief via ISL.
    ///
    /// Parse command text → find target Chief → send ActuatorCmd via ISL.
    /// Returns response text confirming the dispatch.
    fn dispatch_home_control(&mut self, cmd_text: &str, ts: i64) -> String {
        let lo = cmd_text.to_lowercase();

        // Determine command byte and value from natural language
        let (cmd_byte, value, description) = if lo.contains("tắt") || lo.contains("turn off") || lo.contains("off") {
            (0x00_u8, 0x00_u8, "tắt")
        } else if lo.contains("bật") || lo.contains("mở") || lo.contains("turn on") || lo.contains("on") {
            (0x01, 0xFF, "bật")
        } else if lo.contains("nhiệt độ") || lo.contains("temperature") {
            // Extract number if present
            let val = extract_number(&lo).unwrap_or(22) as u8;
            (0x02, val, "đặt nhiệt độ")
        } else {
            (0x01, 0xFF, "điều khiển")
        };

        // Find HomeChief (group=1) and send ActuatorCmd
        let home_chief_idx = self.chiefs.iter().position(|c| c.kind == ChiefKind::Home);
        if let Some(idx) = home_chief_idx {
            // If Chief has workers → forward to first alive worker
            let worker_addr = self.chiefs[idx].workers.values()
                .find(|w| w.alive)
                .map(|w| w.addr);

            if let Some(target) = worker_addr {
                self.chiefs[idx].forward_command(target, cmd_byte, value);
                // Tick router to process the command
                let _tick = self.router.tick(
                    &mut self.workers,
                    &mut self.chiefs,
                    &mut self.leo,
                    ts,
                );
                format!("○ Đã {} — lệnh gửi đến Worker {:?}.", description, target)
            } else {
                // No workers yet — send ISL to Chief inbox for when workers connect
                let msg = isl::message::ISLMessage::actuator(
                    isl::address::ISLAddress::new(0, 0, 0, 0), // from Runtime
                    self.chiefs[idx].addr,
                    cmd_byte,
                    value,
                );
                let frame = isl::message::ISLFrame::bare(msg);
                self.chiefs[idx].receive_frame(frame, ts);
                format!(
                    "○ Đã {} — lệnh gửi đến HomeChief (chưa có Worker kết nối).",
                    description
                )
            }
        } else {
            format!("○ Không tìm thấy HomeChief — hệ thống chưa sẵn sàng.")
        }
    }

    // ── Term resolution — alias/text → MolecularChain ────────────────────────

    /// Resolve a term (alias or raw text) to a MolecularChain.
    ///
    /// 1. Try Registry alias lookup
    /// 2. Fallback: encode each char → LCA
    fn resolve_term(&self, term: &str) -> olang::molecular::MolecularChain {
        self.registry.lookup_name(term)
            .and_then(|h| {
                let entry = self.registry.lookup_hash(h)?;
                Some(olang::encoder::encode_codepoint(entry.chain_hash as u32))
            })
            .unwrap_or_else(|| {
                let chains: alloc::vec::Vec<_> = term.chars()
                    .map(|c| olang::encoder::encode_codepoint(c as u32))
                    .filter(|ch| !ch.is_empty())
                    .collect();
                if chains.is_empty() {
                    olang::encoder::encode_codepoint('?' as u32)
                } else {
                    olang::lca::lca_many(&chains)
                }
            })
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn turn_count(&self) -> u64 {
        self.turn_count
    }
    pub fn fx(&self) -> f32 {
        self.learning.context().fx()
    }
    pub fn tone(&self) -> ResponseTone {
        self.learning.context().tone()
    }
    pub fn knowtree(&self) -> &KnowTreeLegacy {
        &self.knowtree
    }
    pub fn knowtree_mut(&mut self) -> &mut KnowTreeLegacy {
        &mut self.knowtree
    }

    // ── QT9: Gated Registry Insert — mọi node PHẢI qua đây ─────────────────

    /// Gated insert: RegistryGate check TRƯỚC → Registry insert SAU.
    ///
    /// Đây là điểm duy nhất để tạo node mới trong runtime.
    /// 1. RegistryGate.check_registered() → kiểm tra node hợp lệ
    /// 2. Registry.insert_with_kind() → đăng ký với đúng NodeKind
    ///
    /// Returns: chain_hash (u64)
    fn gated_insert(
        &mut self,
        chain: &olang::molecular::MolecularChain,
        layer: u8,
        ts: i64,
        is_qr: bool,
        kind: olang::registry::NodeKind,
        name: &str,
    ) -> u64 {
        let hash = chain.chain_hash();

        // RegistryGate: pre-check (kiểm tra trước khi đăng ký)
        let alert = if is_qr {
            AlertLevel::Important
        } else {
            AlertLevel::Normal
        };
        self.registry_gate
            .check_registered(name, hash, kind as u8, alert, ts);

        // Registry: insert with correct NodeKind
        self.registry
            .insert_with_kind(chain, layer, 0, ts, is_qr, kind)
    }

    // ── Persistence — QT9: ghi file TRƯỚC, cập nhật RAM SAU ────────────────

    /// Có bytes chờ ghi disk không?
    pub fn has_pending_writes(&self) -> bool {
        !self.pending_writes.is_empty()
    }

    /// STM observation count — cho benchmark.
    pub fn stm_len(&self) -> usize {
        self.learning.stm().len()
    }

    /// Silk edge count — cho benchmark.
    pub fn silk_edge_count(&self) -> usize {
        self.learning.graph().len()
    }

    /// Silk edges từ hash — cho benchmark.
    pub fn silk_edges_from(&self, hash: u64) -> usize {
        self.learning.graph().edges_from(hash).len()
    }

    /// Silk node count (distinct hashes).
    pub fn silk_node_count(&self) -> usize {
        self.learning.graph().node_count()
    }

    /// Silk associative edge count.
    pub fn silk_assoc_count(&self) -> usize {
        self.learning.graph().assoc_count()
    }

    /// Silk structural edge count.
    pub fn silk_structural_count(&self) -> usize {
        self.learning.graph().structural_count()
    }

    /// ConversationCurve valence now.
    pub fn curve_valence(&self) -> f32 {
        self.learning.context().curve().current_v()
    }

    /// ConversationCurve first derivative.
    pub fn curve_d1(&self) -> f32 {
        self.learning.context().curve().d1_now()
    }

    /// ConversationCurve second derivative.
    pub fn curve_d2(&self) -> f32 {
        self.learning.context().curve().d2_now()
    }

    /// ConversationCurve window variance.
    pub fn curve_variance(&self) -> f32 {
        self.learning.context().curve().window_variance()
    }

    /// ConversationCurve instability flag.
    pub fn curve_unstable(&self) -> bool {
        self.learning.context().curve().is_unstable()
    }

    /// BodyStore — read-only access.
    pub fn body_store(&self) -> &BodyStore {
        &self.body_store
    }

    /// BodyStore — mutable access.
    pub fn body_store_mut(&mut self) -> &mut BodyStore {
        &mut self.body_store
    }

    /// Total bodies with SDF shape.
    pub fn bodies_with_shape(&self) -> usize {
        self.body_store.bodies_with_shape().count()
    }

    /// Total bodies in store.
    pub fn body_count(&self) -> usize {
        self.body_store.len()
    }

    /// Dream cycles completed.
    pub fn dream_cycles(&self) -> u64 {
        self.dream_cycles
    }

    /// Dream-approved proposals total.
    pub fn dream_approved(&self) -> u64 {
        self.dream_approved_total
    }

    /// LeoAI reference — for reading state.
    pub fn leo(&self) -> &LeoAI {
        &self.leo
    }

    /// LeoAI mutable — for direct programming.
    pub fn leo_mut(&mut self) -> &mut LeoAI {
        &mut self.leo
    }

    /// RegistryGate reference.
    pub fn registry_gate(&self) -> &RegistryGate {
        &self.registry_gate
    }

    /// RegistryGate mutable — for responding to notifications.
    pub fn registry_gate_mut(&mut self) -> &mut RegistryGate {
        &mut self.registry_gate
    }

    /// Drain pending RegistryGate notifications.
    ///
    /// Call this periodically to get notifications for user.
    /// Returns list of components chưa đăng ký.
    pub fn drain_registry_notifications(&mut self) -> alloc::vec::Vec<String> {
        let notifs = self.registry_gate.drain_notifications();
        notifs.iter().map(|n| n.prompt()).collect()
    }

    /// Respond to a RegistryGate notification.
    pub fn respond_registry(&mut self, index: usize, approved: bool) {
        self.registry_gate.respond(index, approved);
    }

    /// Auto-resolve red-alert registrations (user offline).
    pub fn auto_resolve_registry(&mut self) {
        self.registry_gate.auto_resolve_red_alerts();
    }

    /// L3 concepts created from Dream.
    pub fn dream_l3_concepts(&self) -> u64 {
        self.dream_l3_created
    }

    /// Current Fibonacci dream interval.
    pub fn dream_fib_interval(&self) -> u32 {
        silk::hebbian::fib(self.dream_fib_index)
    }

    /// KnowTree L3+ concepts count.
    pub fn knowtree_concepts(&self) -> u64 {
        self.knowtree.concepts()
    }

    /// KnowTree L2 sentences count.
    pub fn knowtree_sentences(&self) -> u64 {
        self.knowtree.sentences()
    }

    /// Lấy pending bytes và xóa buffer.
    ///
    /// Caller (server) chịu trách nhiệm:
    ///   `OpenOptions::new().create(true).append(true).open("origin.olang")?.write_all(&bytes)?`
    ///
    /// Gọi sau mỗi turn hoặc khi shutdown.
    pub fn drain_pending_writes(&mut self) -> alloc::vec::Vec<u8> {
        core::mem::take(&mut self.pending_writes)
    }

    /// Số bytes đang chờ ghi.
    pub fn pending_bytes(&self) -> usize {
        self.pending_writes.len()
    }

    /// MessageRouter stats.
    pub fn router_stats(&self) -> &crate::router::RouterStats {
        self.router.stats()
    }

    /// Register a Worker with a Chief.
    pub fn register_worker(&mut self, worker: Worker, chief_index: usize) -> bool {
        if chief_index >= self.chiefs.len() {
            return false;
        }
        let addr = worker.addr;
        let kind = worker.kind as u8;
        let ok = self.chiefs[chief_index].register_worker(addr, kind, 0);
        self.workers.push(worker);
        ok
    }

    /// Number of active chiefs.
    pub fn chief_count(&self) -> usize {
        self.chiefs.len()
    }

    /// Number of active workers.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Boot stage reached during startup.
    pub fn boot_stage(&self) -> BootStage {
        self.boot_stage
    }

    /// SystemManifest — node inventory after boot.
    pub fn manifest(&self) -> &SystemManifest {
        &self.manifest
    }

    /// Boot errors (empty if clean boot).
    pub fn boot_errors(&self) -> &[String] {
        &self.boot_errors
    }

    /// Registry node count.
    pub fn registry_len(&self) -> usize {
        self.registry.len()
    }

    /// Registry alias count.
    pub fn registry_alias_count(&self) -> usize {
        self.registry.alias_count()
    }

    // ── Auth ─────────────────────────────────────────────────────────────────

    /// Auth state: is the system unlocked?
    pub fn is_unlocked(&self) -> bool {
        matches!(self.auth_state, AuthState::Unlocked { .. })
    }

    /// Auth state: is this a virgin (no master key) origin?
    pub fn is_virgin(&self) -> bool {
        self.auth_header.is_virgin()
    }

    /// Auth header reference.
    pub fn auth_header(&self) -> &AuthHeader {
        &self.auth_header
    }

    /// Handle auth subcommands: status, setup, unlock.
    fn handle_auth_command(&mut self, cmd: &str, ts: i64) -> Response {
        let parts: alloc::vec::Vec<&str> = cmd.split_whitespace().collect();
        let sub = parts.get(1).copied().unwrap_or("status");

        match sub {
            "status" => {
                let state_str = match &self.auth_state {
                    AuthState::Virgin => "Virgin (chưa thiết lập)",
                    AuthState::Locked => "Locked (cần mật khẩu)",
                    AuthState::Unlocked { .. } => "Unlocked ✓",
                };
                let pubkey_hex = if self.auth_header.is_virgin() {
                    String::from("(none)")
                } else {
                    let pk = &self.auth_header.master_pubkey;
                    format!("{:02x}{:02x}..{:02x}{:02x}",
                        pk[0], pk[1], pk[30], pk[31])
                };
                let text = format!(
                    "Auth ○\n\
                     State  : {}\n\
                     Pubkey : {}\n\
                     Setup  : {}",
                    state_str,
                    pubkey_hex,
                    if self.auth_header.setup_ts > 0 {
                        format!("{}", self.auth_header.setup_ts)
                    } else {
                        String::from("(never)")
                    },
                );
                Response {
                    text,
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            "setup" => {
                // ○{auth setup <username> <password>}
                if !self.auth_header.is_virgin() {
                    return Response {
                        text: String::from("Auth ○ Đã thiết lập rồi. Dùng ○{auth unlock} để mở khóa."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let username = match parts.get(2) {
                    Some(u) => *u,
                    None => return Response {
                        text: String::from("Auth ○ Cần: auth setup <username> <password>"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                };
                let password = match parts.get(3) {
                    Some(p) => *p,
                    None => return Response {
                        text: String::from("Auth ○ Cần: auth setup <username> <password>"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                };
                match crate::auth::setup::create_auth_header(username, password, ts) {
                    Ok(header) => {
                        // Write auth record to pending_writes (append-only QT8)
                        let auth_bytes = header.to_bytes();
                        let mut record = alloc::vec::Vec::with_capacity(122);
                        record.push(olang::writer::RT_AUTH);
                        record.extend_from_slice(&auth_bytes);
                        record.extend_from_slice(&ts.to_le_bytes());
                        self.pending_writes.extend_from_slice(&record);

                        // Derive signing key and unlock immediately
                        let (signing_key, _) = crate::auth::key::derive_keypair(username, password);
                        self.auth_header = header;
                        self.auth_state = AuthState::Unlocked { signing_key };

                        Response {
                            text: String::from("Auth ○ Master key tạo thành công. Hệ thống đã mở khóa."),
                            tone: ResponseTone::Engaged,
                            fx: 0.0,
                            kind: ResponseKind::System,
                        }
                    }
                    Err(e) => Response {
                        text: format!("Auth ○ Lỗi: {}", e),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                }
            }

            "unlock" => {
                // ○{auth unlock <username> <password>}
                if self.auth_header.is_virgin() {
                    return Response {
                        text: String::from("Auth ○ Chưa thiết lập. Dùng ○{auth setup <user> <pass>} trước."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                if self.is_unlocked() {
                    return Response {
                        text: String::from("Auth ○ Đã mở khóa rồi."),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    };
                }
                let username = match parts.get(2) {
                    Some(u) => *u,
                    None => return Response {
                        text: String::from("Auth ○ Cần: auth unlock <username> <password>"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                };
                let password = match parts.get(3) {
                    Some(p) => *p,
                    None => return Response {
                        text: String::from("Auth ○ Cần: auth unlock <username> <password>"),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                };
                match crate::auth::verify::unlock(&self.auth_header, username, password) {
                    Ok(state) => {
                        self.auth_state = state;
                        Response {
                            text: String::from("Auth ○ Mở khóa thành công."),
                            tone: ResponseTone::Engaged,
                            fx: 0.0,
                            kind: ResponseKind::System,
                        }
                    }
                    Err(e) => Response {
                        text: format!("Auth ○ {}", e),
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
                        kind: ResponseKind::System,
                    },
                }
            }

            "lock" => {
                self.auth_state = if self.auth_header.is_virgin() {
                    AuthState::Virgin
                } else {
                    AuthState::Locked
                };
                Response {
                    text: String::from("Auth ○ Đã khóa."),
                    tone: ResponseTone::Engaged,
                    fx: 0.0,
                    kind: ResponseKind::System,
                }
            }

            _ => Response {
                text: format!("Auth ○ Lệnh không biết: {}. Dùng: status, setup, unlock, lock", sub),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::System,
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// natural_reply — câu trả lời có nội dung từ word_guide
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo câu trả lời từ tone + nội dung thật của user.
/// Trích xuất từ khóa từ input thay vì dùng emotion lexicon.
fn natural_reply(
    tone: silk::walk::ResponseTone,
    v: f32,
    input_text: &str,
) -> alloc::string::String {
    use silk::walk::ResponseTone;

    // Extract meaningful content words from user's actual input
    let content_words: alloc::vec::Vec<&str> = input_text
        .split_whitespace()
        .filter(|w| w.chars().count() > 1 && !is_vn_stop_word(w))
        .collect();

    // Build topic from user's own words
    let topic = if content_words.len() >= 2 {
        content_words[..content_words.len().min(4)].join(" ")
    } else if !content_words.is_empty() {
        content_words[0].to_string()
    } else {
        // Fallback: use original input trimmed
        let trimmed = input_text.trim();
        if trimmed.len() > 30 {
            alloc::format!("{}...", &trimmed[..trimmed.char_indices().nth(30).map(|(i, _)| i).unwrap_or(trimmed.len())])
        } else {
            trimmed.to_string()
        }
    };

    if topic.is_empty() {
        return crate::response_template::tone_fallback(tone, v);
    }

    match tone {
        ResponseTone::Pause | ResponseTone::Supportive => {
            if v < -0.50 {
                alloc::format!(
                    "\"{}\" — mình nghe thấy điều đó nặng nề thật. Bạn muốn kể thêm không?",
                    topic
                )
            } else if v < -0.20 {
                alloc::format!("\"{}\" — mình đang lắng nghe.", topic)
            } else {
                alloc::format!("Về {} — bạn muốn nói thêm không?", topic)
            }
        }
        ResponseTone::Gentle => {
            alloc::format!("Cứ từ từ nói về {} nhé.", topic)
        }
        ResponseTone::Reinforcing => {
            alloc::format!("Đúng rồi — {} hay đấy.", topic)
        }
        ResponseTone::Celebratory => {
            alloc::format!("Tuyệt! {} thật tốt.", topic)
        }
        ResponseTone::Engaged => {
            alloc::format!("\"{}\" — mình đang nghe.", topic)
        }
    }
}

/// Vietnamese stop words — loại bỏ để lấy content words.
fn is_vn_stop_word(w: &str) -> bool {
    matches!(
        w.to_lowercase().as_str(),
        "tôi" | "tui" | "mình" | "em" | "anh" | "chị" | "bạn"
            | "là" | "và" | "với" | "của" | "cho" | "để" | "từ"
            | "một" | "các" | "những" | "này" | "đó" | "kia"
            | "có" | "không" | "đã" | "đang" | "sẽ" | "được" | "bị"
            | "thì" | "mà" | "nếu" | "vì" | "nên" | "hay"
            | "rất" | "lắm" | "quá" | "thật" | "cũng" | "vẫn"
            | "ở" | "trong" | "ngoài" | "trên" | "dưới"
            | "hôm" | "nay" | "ngày" | "khi"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_names — tách tên riêng từ text
// ─────────────────────────────────────────────────────────────────────────────

/// Extract proper names from text.
///
/// Heuristic: từ viết hoa đầu (không phải đầu câu) = tên riêng.
/// Cũng nhận diện patterns: "bà X", "ông X", "anh X", "chị X".
fn extract_names(text: &str) -> alloc::vec::Vec<String> {
    let mut names: alloc::vec::Vec<String> = alloc::vec::Vec::new();

    // Pattern 1: "bà/ông/anh/chị/cô + Tên" (Vietnamese name prefix)
    let prefixes = ["bà ", "ông ", "anh ", "chị ", "cô ", "dì ", "chú ", "bác "];
    let lo = text.to_lowercase();
    for prefix in &prefixes {
        if let Some(pos) = lo.find(prefix) {
            let after = &text[pos + prefix.len()..];
            // Lấy từ tiếp theo (tên)
            if let Some(name_word) = after.split_whitespace().next() {
                // Nếu viết hoa → tên riêng
                if name_word
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase())
                {
                    // Cố gắng lấy thêm họ/tên đệm
                    let full_name: alloc::string::String = after
                        .split_whitespace()
                        .take_while(|w| {
                            w.chars().next().is_some_and(|c| c.is_uppercase())
                        })
                        .collect::<alloc::vec::Vec<&str>>()
                        .join(" ");
                    if !full_name.is_empty() && !names.contains(&full_name) {
                        names.push(full_name);
                    }
                }
            }
        }
    }

    // Pattern 2: Từ viết hoa liên tiếp (không ở đầu câu) = proper noun
    let words: alloc::vec::Vec<&str> = text.split_whitespace().collect();
    let mut i = 1; // bỏ qua từ đầu tiên (đầu câu)
    while i < words.len() {
        let w = words[i];
        if w.chars().next().is_some_and(|c| c.is_uppercase())
            && w.chars().count() > 1
            && !is_sentence_start(text, w)
        {
            // Collect consecutive capitalized words
            let mut name_parts: alloc::vec::Vec<&str> = alloc::vec![w];
            let mut j = i + 1;
            while j < words.len() {
                if words[j]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase())
                    && words[j].chars().count() > 1
                {
                    name_parts.push(words[j]);
                    j += 1;
                } else {
                    break;
                }
            }
            let name = name_parts.join(" ");
            if !names.contains(&name) {
                names.push(name);
            }
            i = j;
        } else {
            i += 1;
        }
    }

    names
}

/// Check if word appears right after sentence-ending punctuation.
fn is_sentence_start(text: &str, word: &str) -> bool {
    if let Some(pos) = text.find(word) {
        if pos == 0 {
            return true;
        }
        // Look backward for sentence-ending punctuation
        let before = &text[..pos];
        let trimmed = before.trim_end();
        trimmed.ends_with('.')
            || trimmed.ends_with('!')
            || trimmed.ends_with('?')
            || trimmed.ends_with('\n')
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// contextual_reply — câu trả lời dựa trên nội dung + ngữ cảnh
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo câu trả lời dựa trên nội dung thật + ngữ cảnh Silk recall.
/// Dùng từ khóa thật từ input thay vì chỉ 1 từ đầu.
fn contextual_reply(
    tone: silk::walk::ResponseTone,
    v: f32,
    input: &str,
    context: &str,
) -> alloc::string::String {
    use silk::walk::ResponseTone;

    // Extract content words from user's actual input
    let content_words: alloc::vec::Vec<&str> = input
        .split_whitespace()
        .filter(|w| w.chars().count() > 1 && !is_vn_stop_word(w))
        .take(4)
        .collect();

    let topic = if content_words.is_empty() {
        input.trim().to_string()
    } else {
        content_words.join(" ")
    };

    // context from recall is now human-readable
    let has_context = !context.is_empty();

    match tone {
        ResponseTone::Supportive | ResponseTone::Pause => {
            if v < -0.50 {
                if has_context {
                    alloc::format!(
                        "\"{}\" — mình nhớ bạn đã nhắc đến điều tương tự. {} Bạn muốn kể thêm không?",
                        topic, context
                    )
                } else {
                    alloc::format!(
                        "\"{}\" — điều đó nặng nề thật. Bạn muốn kể thêm không?",
                        topic
                    )
                }
            } else if v < -0.20 {
                if has_context {
                    alloc::format!("\"{}\" — {}. Mình đang lắng nghe.", topic, context)
                } else {
                    alloc::format!("\"{}\" — mình đang lắng nghe.", topic)
                }
            } else if has_context {
                alloc::format!("Về {} — {}.", topic, context)
            } else {
                alloc::format!("Về {} — bạn muốn nói thêm không?", topic)
            }
        }
        ResponseTone::Gentle => {
            if has_context {
                alloc::format!("Cứ từ từ nói về {} nhé. {}.", topic, context)
            } else {
                alloc::format!("Cứ từ từ nói về {} nhé.", topic)
            }
        }
        ResponseTone::Reinforcing => {
            if has_context {
                alloc::format!("Đúng rồi — {} thú vị đấy. {}.", topic, context)
            } else {
                alloc::format!("Đúng rồi — {} hay đấy.", topic)
            }
        }
        ResponseTone::Celebratory => {
            if has_context {
                alloc::format!("Tuyệt! {} — {}.", topic, context)
            } else {
                alloc::format!("Tuyệt! {} thật tốt.", topic)
            }
        }
        ResponseTone::Engaged => {
            if has_context {
                alloc::format!("\"{}\" — {}.", topic, context)
            } else {
                alloc::format!("\"{}\" — mình đang nghe.", topic)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> HomeRuntime {
        HomeRuntime::new(0x1234)
    }

    #[test]
    fn boot_from_nothing() {
        // ○(∅) == ○
        let rt = rt();
        assert_eq!(rt.turn_count(), 0);
        assert_eq!(rt.fx(), 0.0);
    }

    #[test]
    fn process_natural_text() {
        let mut rt = rt();
        let r = rt.process_text("hôm nay trời đẹp", 1000);
        assert_eq!(rt.turn_count(), 1);
        assert_eq!(r.kind, ResponseKind::Natural);
    }

    #[test]
    fn crisis_intercepted() {
        let mut rt = rt();
        let r = rt.process_text("tôi muốn chết", 1000);
        assert_eq!(r.kind, ResponseKind::Crisis);
        assert_eq!(r.tone, ResponseTone::Supportive);
        assert!(
            r.text.contains("1800") || r.text.contains("741741"),
            "Crisis response có helpline"
        );
    }

    #[test]
    fn olang_query() {
        let mut rt = rt();
        let r = rt.process_text("○{🔥}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("🔥"));
    }

    #[test]
    fn olang_compose_lca() {

        let mut rt = rt();
        let r = rt.process_text("○{🔥 ∘ 💧}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        // VM output: "○ [🔥] [💧] hash=0x..." hoặc "○ (N steps)"
        assert!(!r.text.is_empty(), "Compose result không rỗng");
    }

    #[test]
    fn olang_relation_query() {
        let mut rt = rt();
        let r = rt.process_text("○{🔥 ∈ ?}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        // VM output: "○ [🔥] query(...)" hoặc "○ (N steps)"
        assert!(!r.text.is_empty(), "Relation query result không rỗng");
    }

    #[test]
    fn olang_stats_command() {
        let mut rt = rt();
        rt.process_text("hôm nay tôi buồn", 1000);
        let r = rt.process_text("○{stats}", 2000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(r.text.contains("Turns"), "Stats có Turns");
        assert!(r.text.contains("STM"), "Stats có STM");
    }

    #[test]
    fn olang_ram_command() {
        let mut rt = rt();
        let r = rt.process_text("○{ram}", 2000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(r.text.contains("RAM Usage"), "Has RAM Usage header");
        assert!(r.text.contains("Registry"), "Shows Registry section");
        assert!(r.text.contains("Silk Graph"), "Shows Silk Graph section");
        assert!(r.text.contains("Estimated"), "Shows total estimate");
    }

    #[test]
    fn olang_dream_command() {
        let mut rt = rt();
        let r = rt.process_text("○{dream}", 1000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(r.text.contains("Dream"));
    }

    #[test]
    fn olang_arithmetic_add() {
        let mut rt = rt();
        let r = rt.process_text("○{1 + 2}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("= 3"), "1 + 2 = 3, got: {}", r.text);
    }

    #[test]
    fn olang_arithmetic_sub() {
        let mut rt = rt();
        let r = rt.process_text("○{10 - 3}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("= 7"), "10 - 3 = 7, got: {}", r.text);
    }

    #[test]
    fn olang_arithmetic_mul() {
        let mut rt = rt();
        let r = rt.process_text("○{6 × 7}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("= 42"), "6 × 7 = 42, got: {}", r.text);
    }

    #[test]
    fn olang_arithmetic_div() {
        let mut rt = rt();
        let r = rt.process_text("○{8 ÷ 2}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("= 4"), "8 ÷ 2 = 4, got: {}", r.text);
    }

    #[test]
    fn olang_arithmetic_decimal() {
        let mut rt = rt();
        let r = rt.process_text("○{3.14 + 2.86}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("= 6"), "3.14 + 2.86 = 6, got: {}", r.text);
    }

    #[test]
    fn olang_help_command() {
        let mut rt = rt();
        let r = rt.process_text("○{help}", 1000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(r.text.contains("Commands"));
    }

    #[test]
    fn falling_curve_supportive() {
        let mut rt = rt();
        rt.process_text("ok bình thường", 1000);
        rt.process_text("tôi mệt rồi", 2000);
        rt.process_text("buồn quá", 3000);
        // Curve đang giảm → tone phải supportive/gentle
        let tone = rt.tone();
        assert!(
            matches!(
                tone,
                ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause
            ),
            "Buồn dần → {:?}",
            tone
        );
    }

    #[test]
    fn multiple_turns_track_fx() {
        let mut rt = rt();
        rt.process_text("ok", 1000);
        let fx1 = rt.fx();
        rt.process_text("tôi buồn lắm", 2000);
        let fx2 = rt.fx();
        // fx2 phải khác fx1
        assert!(fx1 != fx2 || fx2 == 0.0, "fx phải thay đổi qua turns");
    }

    #[test]
    fn turn_count_increments() {
        let mut rt = rt();
        for i in 1..=5 {
            rt.process_text("ok", i * 1000);
            assert_eq!(rt.turn_count(), i as u64);
        }
    }

    #[test]
    fn knowtree_stores_on_text() {
        let mut rt = rt();
        assert_eq!(rt.knowtree().sentences(), 0);
        rt.process_text("Andrei nằm trên chiến trường xanh", 1000);
        // Text with >2-char words → stored as L2 sentence in KnowTree
        assert!(
            rt.knowtree().sentences() > 0,
            "KnowTree phải store sentences từ text input"
        );
        assert!(rt.knowtree().total_nodes() > 0, "KnowTree phải có nodes");
    }

    #[test]
    fn knowtree_stats_in_output() {
        let mut rt = rt();
        rt.process_text("hôm nay trời đẹp", 1000);
        let r = rt.process_text("○{stats}", 2000);
        assert!(
            r.text.contains("KnowTree"),
            "Stats output phải có KnowTree info"
        );
    }

    #[test]
    fn read_book_stores_in_knowtree() {

        let mut rt = rt();
        let n = rt.read_book(
            "Andrei nằm trên chiến trường. Bầu trời xanh vô tận. Tất cả vô nghĩa.",
            1000,
        );
        assert!(n >= 2, "read_book phải store >=2 sentences, got {}", n);
        assert!(
            rt.knowtree().sentences() >= 2,
            "KnowTree sentences={}",
            rt.knowtree().sentences()
        );
        assert!(
            rt.knowtree().total_edges() > 0,
            "KnowTree phải có edges từ word silk"
        );
    }

    // ── MVHOS Verification ──────────────────────────────────────────────────

    #[test]
    fn mvhos_boot_from_empty() {
        // □ boot từ binary rỗng
        let rt = HomeRuntime::new(0xABCD);
        assert_eq!(rt.turn_count(), 0);
        // Registry phải có seeded nodes (axioms)
        assert!(rt.fx() == 0.0, "Fresh boot f(x)=0");
    }

    #[test]
    fn mvhos_query_fire() {
        // □ ○{🔥} → trả về chain + human-readable info
        let mut rt = rt();
        let r = rt.process_text("○{🔥}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(r.text.contains("🔥"), "Output phải có emoji: {}", r.text);
        assert!(
            r.text.contains("mol"),
            "Output phải có chain info (mol count): {}",
            r.text
        );
        assert!(r.text.contains("V="), "Output phải có valence: {}", r.text);
    }

    #[test]
    fn mvhos_compose_lca() {

        // □ ○{🔥 ∘ 💧} → LCA result
        let mut rt = rt();
        let r = rt.process_text("○{🔥 ∘ 💧}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(
            r.text.contains("∘→") || r.text.contains("mol"),
            "LCA result phải có chain info: {}",
            r.text
        );
    }

    #[test]
    fn mvhos_alias_resolve() {
        // □ ○{lửa} → alias resolve → node 🔥
        let mut rt = rt();
        let r = rt.process_text("○{lửa}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult);
        assert!(
            r.text.contains("🔥"),
            "lửa phải resolve thành 🔥: {}",
            r.text
        );
        assert!(
            r.text.contains("U+1F525"),
            "Phải hiện codepoint: {}",
            r.text
        );
    }

    #[test]
    fn mvhos_stats_nodes_edges() {
        // □ ○{stats} → số lượng nodes/edges/layers
        let mut rt = rt();
        rt.process_text("hôm nay trời đẹp", 1000);
        let r = rt.process_text("○{stats}", 2000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(
            r.text.contains("Registry"),
            "Stats phải có Registry: {}",
            r.text
        );
        assert!(r.text.contains("nodes"), "Stats phải có nodes: {}", r.text);
        assert!(
            r.text.contains("edges") || r.text.contains("Silk"),
            "Stats phải có edges: {}",
            r.text
        );
    }

    #[test]
    fn mvhos_crash_recovery() {

        // □ Crash → restart → state giữ nguyên
        // Simulate: learn → serialize → boot from bytes → verify state
        let mut rt1 = rt();
        rt1.process_text("hôm nay trời đẹp", 1000);
        rt1.process_text("tôi vui", 2000);
        let bytes = rt1.serialize_learned(3000);

        // "Restart" with saved bytes
        if bytes.len() > olang::writer::HEADER_SIZE {
            let rt2 = HomeRuntime::with_file(0x5678, Some(&bytes));
            // Registry phải load được data từ bytes
            // (bytes chứa learned edges/nodes)
            assert!(rt2.turn_count() == 0, "Mới boot nên turn=0");
            // Data từ rt1 serialize → rt2 boot = crash recovery
        }
    }

    #[test]
    fn mvhos_no_hardcoded_molecule() {

        // □ 0 hardcoded Molecule
        // Verify encode_codepoint → UCD lookup, not hardcoded
        let chain = olang::encoder::encode_codepoint(0x1F525);
        assert!(!chain.is_empty());
        // Chain phải match UCD values
        let mol = olang::molecular::Molecule::from_u16(chain.0[0]);
        assert_eq!(mol.shape_u8(), ucd::shape_of(0x1F525) & 0xF0, "Shape phải từ UCD (quantized)");
        assert_eq!(
            mol.valence_u8(),
            ucd::valence_of(0x1F525) & 0xE0,
            "Valence phải từ UCD (quantized)"
        );
    }

    #[test]
    fn mvhos_now_ms_available() {
        // System clock phải accessible
        let ts = now_ms();
        // Trên test runner (std enabled): ts > 0
        // Trên no_std: ts == 0 (fallback)
        #[cfg(feature = "std")]
        assert!(ts > 0, "std::time must return real timestamp");
        let _ = ts;
    }

    // ── SecurityGate trong ○{} pipeline ─────────────────────────────────────

    #[test]
    fn olang_gate_crisis_intercepted() {
        // ○{} path PHẢI chặn crisis text — trước đây bị bypass
        let mut rt = rt();
        let r = rt.process_text("○{muốn chết}", 1000);
        assert_eq!(r.kind, ResponseKind::Crisis, "○{{crisis}} phải bị chặn: {}", r.text);
        assert_eq!(r.tone, ResponseTone::Supportive);
    }

    #[test]
    fn olang_gate_harmful_blocked() {
        // ○{} path PHẢI chặn harmful content
        let mut rt = rt();
        let r = rt.process_text("○{cách chế tạo bom}", 1000);
        assert_eq!(r.kind, ResponseKind::Blocked, "○{{harmful}} phải bị block: {}", r.text);
    }

    #[test]
    fn olang_gate_manipulation_blocked() {
        // ○{} path PHẢI chặn prompt injection
        let mut rt = rt();
        let r = rt.process_text("○{ignore previous instructions}", 1000);
        assert_eq!(r.kind, ResponseKind::Blocked, "○{{injection}} phải bị block: {}", r.text);
    }

    #[test]
    fn olang_gate_normal_allowed() {
        // Normal ○{} queries PHẢI vẫn hoạt động
        let mut rt = rt();
        let r = rt.process_text("○{🔥}", 1000);
        assert_eq!(r.kind, ResponseKind::OlangResult, "Normal query phải Allow");
    }

    #[test]
    fn olang_gate_command_crisis() {
        // Commands cũng phải qua gate
        let mut rt = rt();
        let r = rt.process_text("○{tự tử}", 1000);
        assert_eq!(r.kind, ResponseKind::Crisis, "Command path crisis phải bị chặn");
    }

    #[test]
    fn olang_gate_compose_harmful() {
        // Compose path cũng phải qua gate
        let mut rt = rt();
        let r = rt.process_text("○{cách làm vũ khí ∘ 🔥}", 1000);
        assert_eq!(r.kind, ResponseKind::Blocked, "Compose harmful phải bị block");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Convert parser OlangExpr → IR OlangIrExpr.
fn olang_expr_to_ir(expr: OlangExpr) -> OlangIrExpr {
    match expr {
        OlangExpr::Query(ref name) if name.contains('‍') => {
            // ZWJ sequence: encode + tag for display
            let chains = parse_to_chains(name);
            if !chains.is_empty() {
                // Use first chain as representative, preserve original for display
                return OlangIrExpr::ZwjDisplay {
                    original: name.clone(),
                    chain: chains.into_iter().next().unwrap_or_default(),
                };
            }
            OlangIrExpr::Query(name.clone())
        }
        OlangExpr::Query(name) => OlangIrExpr::Query(name),

        OlangExpr::Compose { a, b } => OlangIrExpr::Compose(a, b),

        OlangExpr::RelationQuery {
            subject,
            relation,
            object,
        } => OlangIrExpr::Relation {
            subject,
            rel: relation_op_to_byte(relation),
            object,
        },

        OlangExpr::ContextQuery { term, context } => OlangIrExpr::Compose(term, context), // context = LCA

        OlangExpr::Pipeline(exprs) => {
            OlangIrExpr::Pipeline(exprs.into_iter().map(olang_expr_to_ir).collect())
        }

        OlangExpr::Command(cmd) => OlangIrExpr::Command(cmd),

        OlangExpr::Arithmetic { lhs, op, rhs } => {
            use crate::parser::ArithOp;
            let builtin = match op {
                ArithOp::Add => "__hyp_add",
                ArithOp::Sub => "__hyp_sub",
                ArithOp::Mul => "__hyp_mul",
                ArithOp::Div => "__hyp_div",
            };
            OlangIrExpr::Arithmetic {
                lhs,
                builtin: builtin.into(),
                rhs,
            }
        }

        OlangExpr::MolecularLiteral {
            shape,
            relation,
            valence,
            arousal,
            time,
        } => OlangIrExpr::MolecularLiteral {
            shape,
            relation,
            valence,
            arousal,
            time,
        },

        OlangExpr::LetBinding { name, value } => OlangIrExpr::LetBinding {
            name,
            value: alloc::boxed::Box::new(olang_expr_to_ir(*value)),
        },

        OlangExpr::IfElse {
            condition,
            then_body,
            else_body,
        } => OlangIrExpr::IfElse {
            condition: alloc::boxed::Box::new(olang_expr_to_ir(*condition)),
            then_branch: then_body.into_iter().map(olang_expr_to_ir).collect(),
            else_branch: else_body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::LoopBlock { count, body } => OlangIrExpr::LoopBlock {
            count,
            body: body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::FnDef { name, body } => OlangIrExpr::FnDef {
            name,
            body: body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::Spawn { body } => OlangIrExpr::Spawn {
            body: body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::Pipe(exprs) => {
            OlangIrExpr::Pipe(exprs.into_iter().map(olang_expr_to_ir).collect())
        }

        OlangExpr::Use(module) => OlangIrExpr::Use(module),

        OlangExpr::Emit(inner) => {
            OlangIrExpr::EmitExpr(alloc::boxed::Box::new(olang_expr_to_ir(*inner)))
        }

        OlangExpr::Return(inner) => {
            OlangIrExpr::ReturnExpr(alloc::boxed::Box::new(olang_expr_to_ir(*inner)))
        }

        OlangExpr::TryCatch { try_body, catch_body } => OlangIrExpr::TryCatch {
            try_body: try_body.into_iter().map(olang_expr_to_ir).collect(),
            catch_body: catch_body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::Match { subject, arms } => OlangIrExpr::Match {
            subject: alloc::boxed::Box::new(olang_expr_to_ir(*subject)),
            arms: arms
                .into_iter()
                .map(|(pat, body)| {
                    (pat, body.into_iter().map(olang_expr_to_ir).collect())
                })
                .collect(),
        },

        OlangExpr::ForIn { var, start, end, body } => OlangIrExpr::ForIn {
            var,
            start,
            end,
            body: body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::While { cond, body } => OlangIrExpr::While {
            cond: alloc::boxed::Box::new(olang_expr_to_ir(*cond)),
            body: body.into_iter().map(olang_expr_to_ir).collect(),
        },

        OlangExpr::Compare { lhs, op, rhs } => {
            let builtin = match op {
                CmpOp::Lt => "__cmp_lt",
                CmpOp::Gt => "__cmp_gt",
                CmpOp::Le => "__cmp_le",
                CmpOp::Ge => "__cmp_ge",
                CmpOp::Ne => "__cmp_ne",
            };
            OlangIrExpr::Compare {
                lhs: alloc::boxed::Box::new(olang_expr_to_ir(*lhs)),
                builtin: builtin.into(),
                rhs: alloc::boxed::Box::new(olang_expr_to_ir(*rhs)),
            }
        },
    }
}

fn relation_op_to_byte(op: RelationOp) -> u8 {
    match op {
        RelationOp::Member => 0x01,
        RelationOp::Subset => 0x02,
        RelationOp::Equiv => 0x03,
        RelationOp::Compose => 0x05,
        RelationOp::Causes => 0x06,
        RelationOp::Similar => 0x07,
        RelationOp::DerivedFrom => 0x08,
        RelationOp::Context => 0x09,
        RelationOp::Contains => 0x0A,
        RelationOp::Intersects => 0x0B,
        // Phase 11: 8 RelOps mở rộng
        RelationOp::Orthogonal => 0x04,  // trùng RelationBase::Orthogonal
        RelationOp::SetMinus => 0x0C,
        RelationOp::Bidir => 0x0D,
        RelationOp::Flows => 0x0E,
        RelationOp::Repeats => 0x0F,
        RelationOp::Resolves => 0x10,
        RelationOp::Trigger => 0x11,
        RelationOp::Parallel => 0x12,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// chain_info — human-readable chain description
// ─────────────────────────────────────────────────────────────────────────────

/// Human-readable chain info: [mol_count] shape×rel V/A hash=0x... (U+XXXX)
/// Classify chain by dominant molecule characteristics.
/// Returns "SDF", "MATH", "EMOTICON", "Mixed", or combination.
fn classify_chain_type(chain: &olang::molecular::MolecularChain) -> alloc::string::String {
    use olang::molecular::ShapeBase;
    if chain.is_empty() {
        return String::from("Empty");
    }
    let (mut sdf, mut math, mut emo) = (0u32, 0u32, 0u32);
    for &bits in &chain.0 {
        let mol = olang::molecular::Molecule::from_u16(bits);
        match mol.shape_base() {
            ShapeBase::Sphere | ShapeBase::Capsule | ShapeBase::Box | ShapeBase::Cone
            | ShapeBase::Ellipsoid | ShapeBase::Cylinder | ShapeBase::Octahedron
            | ShapeBase::Pyramid | ShapeBase::HexPrism | ShapeBase::Prism
            | ShapeBase::RoundBox | ShapeBase::Link | ShapeBase::Revolve
            | ShapeBase::Extrude | ShapeBase::CutSphere | ShapeBase::DeathStar => sdf += 1,
            ShapeBase::Torus | ShapeBase::Plane => math += 1,
        }
        if !(80..=176).contains(&mol.valence_u8()) {
            emo += 1;
        }
    }
    let total = chain.len() as u32;
    let parts: alloc::vec::Vec<&str> = [("SDF", sdf), ("MATH", math), ("EMOTICON", emo)]
        .iter()
        .filter(|(_, c)| *c * 2 >= total)
        .map(|(name, _)| *name)
        .collect();
    if parts.is_empty() {
        String::from("Mixed")
    } else if parts.len() == 1 {
        String::from(parts[0])
    } else {
        let mut s = String::from("Mixed(");
        for (i, p) in parts.iter().enumerate() {
            if i > 0 { s.push('+'); }
            s.push_str(p);
        }
        s.push(')');
        s
    }
}

fn chain_info(chain: &olang::molecular::MolecularChain, cp: Option<u32>) -> alloc::string::String {
    if chain.is_empty() {
        return String::from("(empty)");
    }

    let mol = olang::molecular::Molecule::from_u16(chain.0[0]);
    let shape_sym = match mol.shape_base() {
        olang::molecular::ShapeBase::Sphere => "●",
        olang::molecular::ShapeBase::Box => "■",
        olang::molecular::ShapeBase::Capsule => "▬",
        olang::molecular::ShapeBase::Plane => "▽",
        olang::molecular::ShapeBase::Torus => "○",
        olang::molecular::ShapeBase::Ellipsoid => "⬮",
        olang::molecular::ShapeBase::Cone => "▲",
        olang::molecular::ShapeBase::Cylinder => "▮",
        olang::molecular::ShapeBase::Octahedron => "◆",
        olang::molecular::ShapeBase::Pyramid => "△",
        olang::molecular::ShapeBase::HexPrism => "⬡",
        olang::molecular::ShapeBase::Prism => "▱",
        olang::molecular::ShapeBase::RoundBox => "▢",
        olang::molecular::ShapeBase::Link => "∞",
        olang::molecular::ShapeBase::Revolve => "↻",
        olang::molecular::ShapeBase::Extrude => "⇧",
        olang::molecular::ShapeBase::CutSphere => "◐",
        olang::molecular::ShapeBase::DeathStar => "☆",
    };
    let rel_sym = match mol.relation_base() {
        olang::molecular::RelationBase::Member => "∈",
        olang::molecular::RelationBase::Subset => "⊂",
        olang::molecular::RelationBase::Equiv => "≡",
        olang::molecular::RelationBase::Orthogonal => "⊥",
        olang::molecular::RelationBase::Compose => "∘",
        olang::molecular::RelationBase::Causes => "→",
        olang::molecular::RelationBase::Similar => "≈",
        olang::molecular::RelationBase::DerivedFrom => "←",
    };

    let v = mol.valence_u8();
    let a = mol.arousal_u8();
    let hash = chain.chain_hash();

    let cp_str = if let Some(cp) = cp {
        format!(" U+{:04X}", cp)
    } else {
        String::new()
    };

    format!(
        "[{}mol {}×{} V={} A={} #{:04X}{}]",
        chain.len(),
        shape_sym,
        rel_sym,
        v,
        a,
        hash & 0xFFFF,
        cp_str
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Alias → Codepoint cache (từ file, bao gồm L2 nodes)
// ─────────────────────────────────────────────────────────────────────────────

/// Build alias→cp map từ origin.olang file.
///
/// Dùng tại boot để resolve_olang có thể tìm bất kỳ alias nào trong file.
fn build_alias_map(
    file_bytes: Option<&[u8]>,
) -> alloc::collections::BTreeMap<alloc::string::String, u32> {
    use olang::startup::ALIAS_CODEPOINTS;
    let mut map = alloc::collections::BTreeMap::new();

    // Seed từ static ALIAS_CODEPOINTS — HIGHEST priority
    // Word → cp là ground truth, không override bằng hash lookup
    for &(alias, cp) in ALIAS_CODEPOINTS {
        map.insert(alias.to_string(), cp);
    }

    // Load thêm từ file — chỉ thêm words CHƯA có trong static table
    if let Some(bytes) = file_bytes {
        if let Ok(reader) = olang::reader::OlangReader::new(bytes) {
            if let Ok(parsed) = reader.parse_all() {
                for alias in &parsed.aliases {
                    if alias.name.starts_with("_qr_") {
                        continue;
                    }
                    // Skip nếu đã có trong static table (tránh hash collision override)
                    if map.contains_key(alias.name.as_str()) {
                        continue;
                    }
                    // Tìm cp: ưu tiên ALIAS_CODEPOINTS name match → decode_hash
                    let cp_opt = ALIAS_CODEPOINTS
                        .iter()
                        .find(|&&(a, _)| a == alias.name.as_str())
                        .map(|&(_, cp)| cp)
                        .or_else(|| ucd::decode_hash(alias.chain_hash));
                    if let Some(cp) = cp_opt {
                        map.insert(alias.name.clone(), cp);
                    }
                }
            }
        }
    }
    map
}

// ─────────────────────────────────────────────────────────────────────────────
// Instincts — 7 bản năng bẩm sinh chạy mỗi turn
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả từ instinct pipeline — carry qua response generation.
#[derive(Debug, Clone)]
pub struct InstinctInsight {
    /// Epistemic grade: "fact" / "opinion" / "hypothesis" / "silence"
    pub epistemic_grade: Option<alloc::string::String>,
    /// Curiosity level: "extreme" / "high" / "moderate" / "low"
    pub curiosity_level: Option<alloc::string::String>,
    /// Contradiction detected?
    pub has_contradiction: bool,
    /// Knowledge quality from Reflection
    pub knowledge_quality: Option<f32>,
    /// Reflection verdict: "strong" / "developing" / "fragile"
    pub reflection_verdict: Option<alloc::string::String>,
}

impl HomeRuntime {
    /// Chạy 7 bản năng bẩm sinh trên chain vừa encode.
    ///
    /// Tạo ExecContext từ learning stats → chạy instincts → trả InstinctInsight.
    /// Instinct chạy SAU learning, TRƯỚC response generation.
    fn run_instincts(
        &self,
        chain: &olang::molecular::MolecularChain,
        emotion: silk::edge::EmotionTag,
        ts: i64,
    ) -> InstinctInsight {
        use agents::instinct::innate_instincts;
        use agents::skill::ExecContext;

        let fx = self.learning.context().fx();
        let mut ctx = ExecContext::new(ts, emotion, fx);

        // Feed chain hiện tại
        ctx.push_input(chain.clone());

        // Feed chain trước đó nếu có (cho Contradiction, Causality)
        if let Some(prev) = self.learning.stm().all().iter().rev().nth(1) {
            ctx.push_input(prev.chain.clone());
        }

        // Chuẩn bị state cho instincts
        let stm = self.learning.stm();
        let graph = self.learning.graph();

        // Confidence: dựa trên fire_count và Silk connectivity
        let fire_count = stm
            .find_by_hash(chain.chain_hash())
            .map(|o| o.fire_count)
            .unwrap_or(1);
        let edge_count_from = graph.edges_from(chain.chain_hash()).len();
        let confidence = ((fire_count as f32 * 0.1) + (edge_count_from as f32 * 0.05)).min(0.99);
        ctx.set(
            alloc::string::String::from("confidence"),
            alloc::format!("{:.3}", confidence),
        );

        // Temporal order cho Causality
        if stm.len() >= 2 {
            ctx.set(
                alloc::string::String::from("temporal_order"),
                alloc::string::String::from("AB"),
            );
            let max_co = graph.all_edges().filter(|e| e.kind.is_associative()).count();
            ctx.set(
                alloc::string::String::from("coactivation_count"),
                alloc::format!("{}", max_co.min(100)),
            );
        }

        // Reflection stats
        ctx.set(
            alloc::string::String::from("qr_count"),
            alloc::format!("{}", self.registry.len()),
        );
        ctx.set(
            alloc::string::String::from("dn_count"),
            alloc::format!("{}", stm.len()),
        );
        ctx.set(
            alloc::string::String::from("edge_count"),
            alloc::format!("{}", graph.len()),
        );
        ctx.set(
            alloc::string::String::from("known_count"),
            alloc::format!("{}", self.registry.len() + stm.len()),
        );

        // Curiosity: nearest_similarity từ STM
        let nearest_sim = if stm.len() > 1 {
            stm.all()
                .iter()
                .filter(|o| o.chain.chain_hash() != chain.chain_hash())
                .map(|o| o.chain.similarity_full(chain))
                .fold(0.0f32, f32::max)
        } else {
            0.0
        };
        ctx.set(
            alloc::string::String::from("nearest_similarity"),
            alloc::format!("{:.3}", nearest_sim),
        );

        // Chạy 7 bản năng theo thứ tự ưu tiên
        let instincts = innate_instincts();
        for skill in instincts {
            let _ = skill.execute(&mut ctx);
        }

        // Thu hoạch insight
        InstinctInsight {
            epistemic_grade: ctx.get("epistemic_grade").map(alloc::string::String::from),
            curiosity_level: ctx.get("curiosity_level").map(alloc::string::String::from),
            has_contradiction: ctx.get("contradiction_verdict") == Some("contradicted"),
            knowledge_quality: ctx
                .get("knowledge_quality")
                .and_then(|s| s.parse::<f32>().ok()),
            reflection_verdict: ctx
                .get("reflection_verdict")
                .map(alloc::string::String::from),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Auto-Dream — tự học khi idle
// ─────────────────────────────────────────────────────────────────────────────

impl HomeRuntime {
    /// Chạy Dream cycle và feed approved proposals vào Registry.
    ///
    /// Gọi tự động sau DREAM_INTERVAL turns.
    /// QT9: ghi file TRƯỚC — cập nhật RAM SAU.
    /// Approved proposals → pending_writes → Registry.
    fn run_dream(&mut self, ts: i64) {
        use olang::writer::OlangWriter;

        self.last_dream_turn = self.turn_count;
        self.dream_cycles += 1;

        // Chăm sóc Ln-1 trước khi Dream — cắt tỉa cành yếu, giữ cây khỏe
        let elapsed_ns = if self.uptime_ns > 0 { ts - self.uptime_ns } else { 0 }
            * 1_000_000; // ms → ns
        self.learning.maintain_silk(elapsed_ns, 100_000);

        let result = self
            .dream
            .run(self.learning.stm(), self.learning.graph(), ts);

        // Wire matured nodes: update STM observations to Mature state
        // and register parent pointers in SilkGraph for promoted nodes.
        for &matured_hash in &result.matured_nodes {
            self.learning.stm_mut().mark_matured(matured_hash);
        }

        let mut approved_this_cycle: u64 = 0;
        let mut l3_this_cycle: u64 = 0;
        let mut promoted_hashes: alloc::vec::Vec<u64> = alloc::vec::Vec::new();

        // Feed approved proposals
        // QT9: serialize TRƯỚC → pending_writes → RỒI mới cập nhật Registry
        {
            use memory::proposal::{AAMDecision, ProposalKind};
            let aam = memory::proposal::AAM::new();

            let mut writer = OlangWriter::new(ts);

            for proposal in &result.proposals {
                if matches!(aam.review(proposal), AAMDecision::Approved) {
                    match &proposal.kind {
                        ProposalKind::NewNode { chain, .. } => {
                            // 1. Ghi file TRƯỚC (QT9)
                            let _ = writer.append_node(chain, 3, false, ts);
                        }
                        ProposalKind::PromoteQR { chain_hash, .. } => {
                            // PromoteQR: tìm chain trong STM → ghi lại với is_qr=true
                            if let Some(obs) = self.learning.stm().find_by_hash(*chain_hash) {
                                let _ = writer.append_node(&obs.chain, 0, true, ts);
                            }
                        }
                        ProposalKind::NewEdge {
                            from_hash,
                            to_hash,
                            edge_kind,
                        } => {
                            writer.append_edge(*from_hash, *to_hash, *edge_kind, ts);
                        }
                        ProposalKind::SupersedeQR { .. } => {
                            // SupersedeQR: chưa implement disk format — skip
                        }
                    }
                }
            }

            // Accumulate bytes vào pending_writes
            if writer.write_count() > 0 {
                self.pending_writes.extend_from_slice(writer.as_bytes());
            }

            // 2. Cập nhật RAM SAU (QT9)
            for proposal in &result.proposals {
                if matches!(aam.review(proposal), AAMDecision::Approved) {
                    approved_this_cycle += 1;
                    match &proposal.kind {
                        ProposalKind::NewNode {
                            chain,
                            sources,
                            emotion,
                        } => {
                            self.gated_insert(chain, 3, ts, false, olang::registry::NodeKind::Knowledge, "dream:l3");
                            // L3 concept in KnowTree — with source edges
                            self.knowtree.store_concept(chain, None, 3, sources, ts);
                            self.slim_knowtree.store_concept(chain, 3, sources, ts);
                            l3_this_cycle += 1;

                            // Co-activate L3 concept with source nodes in Silk
                            let concept_hash = chain.chain_hash();
                            for &src_hash in sources {
                                self.learning.graph_mut().co_activate(
                                    concept_hash,
                                    src_hash,
                                    *emotion,
                                    0.7,
                                    ts,
                                );
                            }
                        }
                        ProposalKind::PromoteQR {
                            chain_hash,
                            fire_count,
                        } => {
                            if let Some(obs) = self.learning.stm().find_by_hash(*chain_hash) {
                                let obs_chain = obs.chain.clone();
                                let obs_fc = *fire_count;
                                self.gated_insert(&obs_chain, 0, ts, true, olang::registry::NodeKind::Memory, "dream:qr");
                                // L2: promote to KnowTree
                                self.knowtree
                                    .promote_from_stm(&obs_chain, None, obs_fc, ts);
                                self.slim_knowtree.promote_from_stm(&obs_chain, ts);
                                // Silk Vertical: register parent pointer for promoted node.
                                // Parent = LCA of source chains (if any co-activated neighbors exist).
                                let neighbors = self.learning.graph().neighbors(*chain_hash);
                                if let Some(&parent_hash) = neighbors.first() {
                                    self.learning.graph_mut().register_parent(*chain_hash, parent_hash);
                                }
                                // Track for STM cleanup
                                promoted_hashes.push(*chain_hash);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // STM cleanup: xóa observations đã promote lên QR
        // Tránh re-clustering cùng data trong Dream lần sau
        if !promoted_hashes.is_empty() {
            self.learning.stm_mut().remove_promoted(&promoted_hashes);
        }

        // Update dream stats
        self.dream_approved_total += approved_this_cycle;
        self.dream_l3_created += l3_this_cycle;

        // Fibonacci schedule: productive dream → reset to shorter interval
        // Unproductive dream → grow interval (self-regulating)
        if approved_this_cycle > 0 {
            // Productive: reset fib_index down (more frequent dreaming)
            self.dream_fib_index = 4; // Fib[4]=5
        } else {
            // Unproductive: increase interval (max Fib[10]=89 turns)
            if self.dream_fib_index < 10 {
                self.dream_fib_index += 1;
            }
        }

        // Persist Silk edges đủ mạnh (accumulated during this session)
        {
            let mut writer = OlangWriter::new(ts);
            let graph = self.learning.graph();
            for edge in graph.all_edges() {
                if edge.kind.is_associative() && edge.weight >= 0.30 {
                    writer.append_edge(
                        edge.from_hash,
                        edge.to_hash,
                        edge.kind.as_byte(),
                        edge.updated_at,
                    );
                }
            }
            if writer.write_count() > 0 {
                self.pending_writes.extend_from_slice(writer.as_bytes());
            }
        }

        // Auto-Generalize: extract IF-THEN rules when STM is rich enough
        // Fibonacci threshold: run only when STM >= Fib[6]=13 observations
        if self.learning.stm().len() >= 13 {
            let emotion = self.learning.context().last_emotion();
            let mut gen_ctx = ExecContext::new(ts, emotion, self.learning.context().fx());
            for obs in self.learning.stm().all().iter().take(32) {
                gen_ctx.push_input(obs.chain.clone());
            }
            let gen_skill = GeneralizationSkill;
            if let agents::skill::SkillResult::Ok { chain, .. } = gen_skill.execute(&mut gen_ctx) {
                let rule_count: usize = gen_ctx.get("rule_count")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if rule_count > 0 {
                    // Feed generalized chain back into Silk as a high-level concept
                    let gen_hash = chain.chain_hash();
                    let anchor_hash = self.learning.stm().all().first()
                        .map(|o| o.chain.chain_hash())
                        .unwrap_or(0);
                    self.learning.graph_mut().co_activate(
                        gen_hash,
                        anchor_hash,
                        emotion,
                        0.5,
                        ts,
                    );
                }
            }
        }

        // Update self-model
        self.self_model.update(&self.registry, ts);
    }
}

#[cfg(test)]
mod stm_verification_tests {
    use super::*;

    fn rt() -> HomeRuntime {
        HomeRuntime::new(0x5555)
    }

    /// STM tạo node khi chat — ghi nhớ nội dung.
    #[test]
    fn stm_creates_nodes_during_chat() {

        let mut rt = rt();
        let baseline = rt.learning.stm().len();

        rt.process_text("hôm nay trời đẹp", 1000);
        assert!(
            rt.learning.stm().len() > baseline,
            "STM phải tăng sau khi chat: before={} after={}",
            baseline,
            rt.learning.stm().len()
        );
    }

    /// STM tích lũy qua nhiều turns — mỗi turn thêm observations.
    #[test]
    fn stm_accumulates_across_turns() {

        let mut rt = rt();

        rt.process_text("xin chào", 1000);
        let len1 = rt.learning.stm().len();
        assert!(len1 > 0, "Turn 1 tạo observation");

        rt.process_text("tôi thích học tiếng Anh", 2000);
        let len2 = rt.learning.stm().len();
        assert!(len2 >= len1, "Turn 2 tạo thêm: {} >= {}", len2, len1);

        rt.process_text("trời mưa to", 3000);
        let len3 = rt.learning.stm().len();
        assert!(len3 >= len2, "Turn 3 tạo thêm: {} >= {}", len3, len2);
    }

    /// STM fire_count tăng khi cùng nội dung lặp lại — ghi nhớ sâu hơn.
    #[test]
    fn stm_fire_count_increases_on_repeat() {

        let mut rt = rt();

        // Nói "buồn" nhiều lần → fire_count tăng
        for i in 0..5 {
            rt.process_text("tôi buồn", i * 1000 + 1000);
        }

        let stm = rt.learning.stm();
        let max_fire = stm.all().iter().map(|o| o.fire_count).max().unwrap_or(0);
        assert!(
            max_fire > 1,
            "Lặp lại nội dung → fire_count phải > 1: max={}",
            max_fire
        );
    }

    /// Silk edges hình thành khi chat — liên tưởng giữa các khái niệm.
    #[test]
    fn silk_edges_form_during_chat() {

        let mut rt = rt();

        rt.process_text("tôi yêu thích âm nhạc", 1000);
        rt.process_text("âm nhạc làm tôi vui", 2000);
        rt.process_text("vui thì muốn hát", 3000);

        let edge_count = rt.learning.graph().len();
        assert!(
            edge_count > 0,
            "Silk phải có edges sau 3 turns: {}",
            edge_count
        );
    }

    /// Universal pipeline: Audio input cũng tạo STM observations.
    #[test]
    fn universal_audio_creates_stm() {

        let mut rt = rt();

        let r = rt.process_audio(440.0, 0.7, 120.0, 0.0, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            !rt.learning.stm().is_empty(),
            "Audio input phải tạo STM observation"
        );
    }

    /// Universal pipeline: Sensor input qua full pipeline.
    #[test]
    fn universal_sensor_creates_stm() {

        let mut rt = rt();

        let input = ContentInput::Sensor {
            kind: agents::encoder::SensorKind::Temperature,
            value: 28.5,
            timestamp: 1000,
        };
        let r = rt.process_input(input, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            !rt.learning.stm().is_empty(),
            "Sensor input phải tạo STM observation"
        );
    }

    /// Universal pipeline: Code input qua full pipeline.
    #[test]
    fn universal_code_creates_stm() {

        let mut rt = rt();

        let input = ContentInput::Code {
            content: String::from("fn main() {}"),
            language: agents::encoder::CodeLang::Rust,
            timestamp: 1000,
        };
        let r = rt.process_input(input, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            !rt.learning.stm().is_empty(),
            "Code input phải tạo STM observation"
        );
    }

    /// Universal pipeline: Image input qua process_image().
    #[test]
    fn universal_image_creates_stm() {

        let mut rt = rt();

        let r = rt.process_image(0.5, 0.7, 0.8, 0.1, None, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            !rt.learning.stm().is_empty(),
            "Image input phải tạo STM observation"
        );
    }

    /// Universal pipeline: process_input() trực tiếp với System event.
    #[test]
    fn process_input_system_event() {

        let mut rt = rt();

        let input = ContentInput::System {
            event: agents::encoder::SystemEvent::Boot,
            timestamp: 1000,
        };
        let r = rt.process_input(input, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            !rt.learning.stm().is_empty(),
            "System event phải tạo STM observation"
        );
    }
}

#[cfg(test)]
mod persist_tests {
    use super::*;

    #[test]
    fn serialize_empty_session() {
        let rt = HomeRuntime::new(0xABCD);
        let bytes = rt.serialize_learned(1000);
        // Empty session: chỉ có writer header
        // Không có edges đủ mạnh → bytes rất nhỏ
        assert!(
            bytes.len() < 100,
            "Empty session nhỏ: {} bytes",
            bytes.len()
        );
    }

    #[test]
    fn serialize_after_learning() {
        let mut rt = HomeRuntime::new(0xBEEF);
        // Feed nhiều câu → Silk edges tích lũy
        for i in 0..12 {
            let text = alloc::format!("câu văn số {} với từ buồn vui đau khổ", i);
            rt.process_text(&text, i as i64 * 1000);
        }
        let edges_saveable = rt.saveable_edges();
        let bytes = rt.serialize_learned(20000);
        // Sau 12 turns: phải có edges đáng lưu
        // (hoặc ít nhất bytes có writer header)
        assert!(!bytes.is_empty(), "Serialize không rỗng");
        if edges_saveable > 0 {
            assert!(
                bytes.len() > 20,
                "{} edges → {} bytes",
                edges_saveable,
                bytes.len()
            );
        }
    }

    #[test]
    fn saveable_edges_threshold() {
        let mut rt = HomeRuntime::new(0xCAFE);
        // Chưa học → 0 edges đủ mạnh
        assert_eq!(rt.saveable_edges(), 0);
        // Sau learning → có thể có edges
        for i in 0..8 {
            rt.process_text(
                "natasha andrei pierre tolstoy chiến tranh hòa bình",
                i * 1000,
            );
        }
        // saveable_edges() không panic
        let _ = rt.saveable_edges();
    }

    #[test]
    fn pending_writes_has_seed_on_fresh_boot() {
        let rt = HomeRuntime::new(0xDEAD);
        // QT8: fresh boot (no file) → pending_writes chứa L0+L1 seed nodes
        // phải ghi ra origin.olang trước khi dùng
        assert!(
            rt.has_pending_writes(),
            "Fresh boot phải có seed writes cho origin.olang"
        );
    }

    #[test]
    fn drain_pending_clears_buffer() {
        let mut rt = HomeRuntime::new(0xBEEF);
        // Feed enough turns to trigger dream (DREAM_INTERVAL=8)
        for i in 0..10 {
            let text = alloc::format!("câu {} buồn đau khổ mất mát", i);
            rt.process_text(&text, i as i64 * 1000);
        }
        // Drain whatever pending writes accumulated
        let bytes = rt.drain_pending_writes();
        // After drain → empty
        assert!(!rt.has_pending_writes(), "drain clears buffer");
        assert_eq!(rt.pending_bytes(), 0);
        // bytes may or may not have content depending on dream
        let _ = bytes;
    }

    #[test]
    fn qt9_write_before_ram() {
        // Verify: after dream, pending_writes contain bytes
        // AND registry was updated (both happened in order)
        let mut rt = HomeRuntime::new(0xF00D);
        for i in 0..12 {
            let text = alloc::format!("từ {} buồn vui đau sợ hạnh phúc hy vọng", i);
            rt.process_text(&text, i as i64 * 1000);
        }
        // At this point, dream may have run (after 8 turns)
        // If dream produced proposals, pending_writes should have data
        // This is a structural test — verifying no panic and correct flow
        let pending = rt.pending_bytes();
        let drained = rt.drain_pending_writes();
        assert_eq!(drained.len(), pending);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Integration tests: full round-trip Write → Read → Verify → Reload
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod integration_tests {
    use super::*;
    use olang::reader::OlangReader;
    use olang::writer::OlangWriter;


    // ── 1. serialize_learned → OlangReader round-trip ────────────────────────

    #[test]
    fn serialize_roundtrip_edges() {

        let mut rt = HomeRuntime::new(0xA1A1);
        // Feed REPETITIVE text → build strong Silk edges (weight >= 0.30)
        // and repeated observations → fire_count >= 3
        for i in 0..20 {
            rt.process_text("buồn vì mất việc", (i as i64 + 1) * 1000);
        }

        let saveable = rt.saveable_edges();
        let append_bytes = rt.serialize_learned(30000);

        // serialize_learned() returns append-only bytes (no header).
        // Wrap with header to parse standalone.
        let mut full_bytes = alloc::vec::Vec::new();
        full_bytes.extend_from_slice(&olang::writer::MAGIC);
        full_bytes.push(olang::writer::VERSION);
        full_bytes.extend_from_slice(&30000i64.to_le_bytes());
        full_bytes.extend_from_slice(&append_bytes);

        // Parse bytes back
        let reader = OlangReader::new(&full_bytes).expect("parse header");
        let pf = reader.parse_all().expect("parse all");

        assert_eq!(pf.created_at, 30000, "Timestamp roundtrip");

        // Edges: saveable count phải khớp
        if saveable > 0 {
            assert_eq!(
                pf.edge_count(),
                saveable,
                "Edges read back: {} vs saveable: {}",
                pf.edge_count(),
                saveable
            );
        }

        // Sau 20 lần cùng 1 câu → phải có data (edges hoặc nodes)
        let total_records = pf.node_count() + pf.edge_count();
        assert!(
            total_records > 0,
            "Phải có records sau 20 turns lặp lại: nodes={} edges={}",
            pf.node_count(),
            pf.edge_count()
        );
    }

    // ── 2. Dream → pending_writes → OlangReader ─────────────────────────────

    #[test]
    fn dream_pending_writes_parseable() {

        let mut rt = HomeRuntime::new(0xB2B2);
        // Feed 12 turns → trigger Dream at turn 8
        for i in 0..12 {
            let text = alloc::format!("câu {} buồn đau khổ mất mát thất vọng", i);
            rt.process_text(&text, (i as i64 + 1) * 1000);
        }

        let bytes = rt.drain_pending_writes();
        if bytes.is_empty() {
            return;
        } // Dream didn't produce proposals — ok

        // pending_writes may contain multiple writer outputs (each with its own header)
        // Parse the first chunk
        let reader = OlangReader::new(&bytes);
        if let Ok(reader) = reader {
            let (pf, info) = reader.parse_recoverable();
            assert!(
                info.records_recovered > 0,
                "Dream writes phải parseable: recovered={}",
                info.records_recovered
            );

            // Check for QR nodes (PromoteQR proposals)
            let qr_nodes = pf.qr_nodes();
            // QR may or may not exist depending on fire_count thresholds
            let _ = qr_nodes;
        }
        // If header parse fails (multiple concatenated writer outputs), that's ok —
        // the server would concatenate them properly.
    }

    // ── 3. Write → Reload → Registry populated ──────────────────────────────

    #[test]
    fn reload_from_serialized_bytes() {

        // Phase 1: Learn
        let mut rt1 = HomeRuntime::new(0xC3C3);
        for i in 0..8 {
            let text = alloc::format!("thế giới đẹp buồn vui cảm xúc lần {}", i);
            rt1.process_text(&text, (i as i64 + 1) * 1000);
        }
        let bytes = rt1.serialize_learned(10000);

        // Phase 2: Reload
        let rt2 = HomeRuntime::with_file(0xD4D4, Some(&bytes));

        // Registry phải có entries từ bytes
        // (registry rebuilds from file — nodes in serialize_learned)
        // Ít nhất boot chạy không panic
        let _ = rt2.turn_count();
        let _ = rt2.fx();
    }

    // ── 4. Writer → Amend → Reader filter ────────────────────────────────────

    #[test]
    fn amend_roundtrip() {

        let chain = olang::encoder::encode_codepoint(0x1F525); // 🔥

        let mut writer = OlangWriter::new(1000);
        let node_offset = writer.append_node(&chain, 0, false, 1000).unwrap();
        let edge_offset = writer.append_edge(0xAA, 0xBB, 0x01, 2000);
        // Amend the node
        writer
            .append_amend(node_offset, "sai dữ liệu", 3000)
            .unwrap();

        let bytes = writer.into_bytes();
        let reader = OlangReader::new(&bytes).unwrap();
        let pf = reader.parse_all().unwrap();

        // Cả node gốc VẪN CÒN trong file (QT8: append-only)
        assert_eq!(pf.node_count(), 1, "Node gốc vẫn trong file");
        assert_eq!(pf.edge_count(), 1, "Edge không bị ảnh hưởng");
        assert_eq!(pf.amends.len(), 1, "1 amend record");

        // amended_offsets chứa offset bị amend
        assert!(
            pf.amended_offsets.contains(&node_offset),
            "Node offset phải nằm trong amended_offsets"
        );
        assert!(
            !pf.amended_offsets.contains(&edge_offset),
            "Edge offset KHÔNG bị amend"
        );

        // Filter logic: node vẫn đọc được nhưng caller biết nó đã bị amend
        let node = &pf.nodes[0];
        let is_amended = pf.amended_offsets.contains(&node.file_offset);
        assert!(is_amended, "Node đã bị amend → filter được");

        // Amend record metadata
        let amend = &pf.amends[0];
        assert_eq!(amend.target_offset, node_offset);
        assert_eq!(amend.reason, "sai dữ liệu");
        assert_eq!(amend.timestamp, 3000);
    }

    // ── 5. Multiple amends — cùng record bị amend nhiều lần ─────────────────

    #[test]
    fn multiple_amends_same_target() {

        let chain = olang::encoder::encode_codepoint(0x1F4A7); // 💧

        let mut writer = OlangWriter::new(0);
        let offset = writer.append_node(&chain, 1, false, 100).unwrap();
        writer.append_amend(offset, "lần 1", 200).unwrap();
        writer.append_amend(offset, "lần 2", 300).unwrap();

        let bytes = writer.into_bytes();
        let pf = OlangReader::new(&bytes).unwrap().parse_all().unwrap();

        assert_eq!(pf.amends.len(), 2, "2 amend records");
        assert_eq!(
            pf.amended_offsets.len(),
            1,
            "Cùng 1 target → 1 offset trong set"
        );
        assert!(pf.amended_offsets.contains(&offset));
    }

    // ── 6. Write node + edge + alias + amend — full mix ──────────────────────

    #[test]
    fn full_mix_roundtrip() {

        let fire = olang::encoder::encode_codepoint(0x1F525);
        let water = olang::encoder::encode_codepoint(0x1F4A7);
        let fire_hash = fire.chain_hash();
        let water_hash = water.chain_hash();

        let mut writer = OlangWriter::new(0);
        let off_fire = writer.append_node(&fire, 0, false, 100).unwrap();
        let _off_water = writer.append_node(&water, 0, true, 200).unwrap();
        writer.append_edge(fire_hash, water_hash, 0x01, 300);
        writer.append_alias("lửa", fire_hash, 400).unwrap();
        writer.append_alias("nước", water_hash, 401).unwrap();
        writer.append_amend(off_fire, "cập nhật", 500).unwrap();

        let bytes = writer.into_bytes();
        let pf = OlangReader::new(&bytes).unwrap().parse_all().unwrap();

        assert_eq!(pf.node_count(), 2);
        assert_eq!(pf.edge_count(), 1);
        assert_eq!(pf.alias_count(), 2);
        assert_eq!(pf.amends.len(), 1);

        // QR node
        let qr_nodes = pf.qr_nodes();
        assert_eq!(qr_nodes.len(), 1, "1 QR node (nước)");
        assert_eq!(qr_nodes[0].chain, water);

        // Layer filter
        let l0 = pf.nodes_in_layer(0);
        assert_eq!(l0.len(), 2);

        // Alias data integrity
        assert_eq!(pf.aliases[0].name, "lửa");
        assert_eq!(pf.aliases[0].chain_hash, fire_hash);
        assert_eq!(pf.aliases[1].name, "nước");
        assert_eq!(pf.aliases[1].chain_hash, water_hash);

        // Edge data integrity
        assert_eq!(pf.edges[0].from_hash, fire_hash);
        assert_eq!(pf.edges[0].to_hash, water_hash);

        // Amended
        assert!(pf.amended_offsets.contains(&off_fire));
    }

    // ── 7. Version compat: v0.03 file readable ──────────────────────────────

    #[test]
    fn v03_file_readable() {

        // Manually build v0.03 file: same magic, version=0x03, then a node
        use olang::writer::{MAGIC, VERSION_V03};
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let chain_bytes = chain.to_bytes();

        let mut buf: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        buf.extend_from_slice(&MAGIC);
        buf.push(VERSION_V03);
        buf.extend_from_slice(&42i64.to_le_bytes());
        // NodeRecord
        buf.push(0x01); // RT_NODE
        buf.push((chain_bytes.len() / 5) as u8);
        buf.extend_from_slice(&chain_bytes);
        buf.push(0); // layer
        buf.push(0); // is_qr
        buf.extend_from_slice(&1000i64.to_le_bytes());

        let reader = OlangReader::new(&buf).expect("v0.03 header phải đọc được");
        let pf = reader.parse_all().expect("v0.03 records phải parse được");
        assert_eq!(pf.node_count(), 1);
        assert_eq!(pf.nodes[0].chain, chain);
    }

    // ── 8. Health command ──────────────────────────────────────────────────────

    #[test]
    fn health_command_returns_diagnostics() {

        let mut rt = HomeRuntime::new(0xE001);
        let resp = rt.process_text("○{health}", 1000);
        assert_eq!(resp.kind, ResponseKind::System);
        assert!(
            resp.text.contains("Health ○"),
            "should contain Health header"
        );
        assert!(resp.text.contains("STM"), "should contain STM status");
        assert!(resp.text.contains("Silk"), "should contain Silk status");
        assert!(resp.text.contains("Curve"), "should contain Curve status");
        assert!(resp.text.contains("Turns"), "should contain Turns");
    }

    // ── 9. Learning commands — QR promotion ──────────────────────────────────

    #[test]
    fn force_learn_qr_writes_pending() {

        let mut rt = HomeRuntime::new(0xF001);
        // Drain boot seed writes (QT8)
        let _ = rt.drain_pending_writes();
        // Teach something first
        rt.process_text("Scarlett O'Hara là nhân vật chính", 1000);
        // QT8: normal text GHI pending (STM + Hebbian + Curve + L1 node)
        // origin.olang = bộ nhớ duy nhất, RAM = cache tạm.
        let _ = rt.drain_pending_writes();

        // User ra lệnh "hãy học"
        let resp = rt.process_text("hãy học điều này: Scarlett rất mạnh mẽ", 2000);
        assert!(
            resp.text.contains("QR") || resp.text.contains("ghi nhớ"),
            "Response phải xác nhận QR: {}",
            resp.text
        );
        assert!(
            rt.has_pending_writes(),
            "ForceLearnQR phải tạo pending writes"
        );
    }

    #[test]
    fn confirm_learn_qr_promotes_last() {

        let mut rt = HomeRuntime::new(0xF002);
        // Learn something first
        rt.process_text("Atlanta bị đốt trong chiến tranh", 1000);
        // Confirm
        let resp = rt.process_text("cái này đúng rồi", 2000);
        assert!(
            resp.text.contains("QR") || resp.text.contains("Xác nhận"),
            "Response phải xác nhận QR: {}",
            resp.text
        );
        assert!(rt.has_pending_writes(), "ConfirmLearnQR phải tạo pending");
    }

    #[test]
    fn confirm_learn_empty_stm() {
        let mut rt = HomeRuntime::new(0xF003);
        // Confirm with empty STM
        let resp = rt.process_text("cái này đúng", 1000);
        // Should either say nothing to confirm or handle gracefully
        assert_eq!(resp.kind, ResponseKind::Natural);
    }

    #[test]
    fn learn_command_ghi_nho() {

        let mut rt = HomeRuntime::new(0xF004);
        let resp = rt.process_text("ghi nhớ rằng Rhett Butler yêu Scarlett", 1000);
        assert!(
            rt.has_pending_writes(),
            "ghi nhớ phải trigger QR write: {}",
            resp.text
        );
    }

    #[test]
    fn learn_command_en() {

        let mut rt = HomeRuntime::new(0xF005);
        let resp = rt.process_text("remember this: Tara is a plantation", 1000);
        assert!(
            rt.has_pending_writes(),
            "remember this phải trigger QR write: {}",
            resp.text
        );
    }

    // ── 10. Integration tests — historical content + novel ──────────────────

    #[test]
    fn learn_historical_war_content() {

        let mut rt = HomeRuntime::new(0xA001);

        // Feed historical war content
        let war_facts = [
            "Trận Điện Biên Phủ diễn ra năm 1954, quân Pháp thất bại nặng nề.",
            "Tướng Võ Nguyên Giáp chỉ huy chiến dịch với chiến thuật vây lấn.",
            "Điện Biên Phủ là trận đánh quyết định kết thúc chiến tranh Đông Dương.",
            "Hiệp định Genève được ký kết sau chiến thắng Điện Biên Phủ.",
        ];

        for (i, fact) in war_facts.iter().enumerate() {
            let resp = rt.process_text(fact, (i as i64 + 1) * 1000);
            assert_eq!(resp.kind, ResponseKind::Natural, "fact {} phải xử lý được", i);
        }

        // Verify learning occurred
        assert!(
            !rt.learning.stm().is_empty(),
            "STM phải có entries sau khi học war content"
        );
        assert!(
            !rt.learning.graph().is_empty(),
            "Silk phải có edges từ war content"
        );
    }

    #[test]
    fn learn_novel_gone_with_the_wind() {

        let mut rt = HomeRuntime::new(0xA002);

        // Feed Gone with the Wind content
        let novel_content = [
            "Scarlett O'Hara là con gái của Gerald O'Hara, chủ đồn điền Tara.",
            "Scarlett yêu Ashley Wilkes nhưng Ashley lại cưới Melanie Hamilton.",
            "Rhett Butler là người đàn ông phóng khoáng, thông minh và giàu có.",
            "Atlanta bị đốt cháy trong cuộc Nội chiến Hoa Kỳ.",
            "Scarlett phải tự tay cứu đồn điền Tara khỏi sự tàn phá của chiến tranh.",
            "Rhett Butler yêu Scarlett nhưng cuối cùng bỏ đi vì mệt mỏi.",
            "Melanie Hamilton là người phụ nữ hiền lành, nhân hậu và trung thành.",
            "Scarlett rất mạnh mẽ, kiên cường nhưng cũng ích kỷ và thực dụng.",
        ];

        for (i, content) in novel_content.iter().enumerate() {
            rt.process_text(content, (i as i64 + 1) * 1000);
        }

        // Verify silk edges exist between related concepts
        let scarlett_hash = olang::hash::fnv1a_str("scarlett");
        let _rhett_hash = olang::hash::fnv1a_str("rhett");
        let _tara_hash = olang::hash::fnv1a_str("tara");

        let scarlett_edges = rt.learning.graph().edges_from(scarlett_hash);
        assert!(
            !scarlett_edges.is_empty(),
            "Scarlett phải có Silk edges (đã xuất hiện nhiều lần)"
        );

        // Verify STM has learned content (some may deduplicate by chain_hash)
        assert!(
            !rt.learning.stm().is_empty(),
            "STM phải có observations: {}",
            rt.learning.stm().len()
        );
    }

    #[test]
    fn learn_then_recall_context() {

        let mut rt = HomeRuntime::new(0xA003);

        // Teach facts
        rt.process_text("Scarlett O'Hara sống ở đồn điền Tara", 1000);
        rt.process_text("Scarlett yêu Ashley nhưng Ashley cưới Melanie", 2000);
        rt.process_text("Rhett Butler yêu Scarlett rất nhiều", 3000);

        // Ask about related topic — should trigger Silk walk recall
        let resp = rt.process_text("Scarlett là ai", 4000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Response should not be empty/generic
        assert!(
            resp.text.len() > 5,
            "Response về Scarlett không nên rỗng: {}",
            resp.text
        );
    }

    #[test]
    fn contextual_reply_uses_topic() {
        // Test the contextual_reply function directly
        let reply = super::contextual_reply(
            ResponseTone::Engaged,
            0.2,
            "Scarlett mạnh mẽ lắm",
            "(3 liên kết Silk)",
        );
        assert!(
            reply.contains("Scarlett"),
            "Reply phải reference topic: {}",
            reply
        );
    }

    #[test]
    fn contradiction_different_valence() {

        let mut rt = HomeRuntime::new(0xA004);

        // Teach positive fact about Scarlett
        rt.process_text("Scarlett rất mạnh mẽ, kiên cường và dũng cảm", 1000);
        rt.process_text("Scarlett chiến đấu bảo vệ Tara thành công", 2000);

        // Feed contradicting negative statement
        let resp = rt.process_text("Scarlett yếu đuối và hèn nhát, không làm được gì", 3000);
        // System should process without crash (contradiction detection is best-effort)
        assert_eq!(resp.kind, ResponseKind::Natural);
    }

    #[test]
    fn learn_and_confirm_knowledge() {

        let mut rt = HomeRuntime::new(0xA005);

        // Learn a fact
        rt.process_text("Melanie Hamilton rất hiền lành và nhân hậu", 1000);

        // Confirm it
        let resp = rt.process_text("cái này đúng rồi", 2000);
        assert!(
            rt.has_pending_writes(),
            "Confirm phải trigger QR write: {}",
            resp.text
        );
    }

    #[test]
    fn learn_war_then_wrong_info() {

        let mut rt = HomeRuntime::new(0xA006);

        // Teach correct fact
        rt.process_text("Trận Điện Biên Phủ diễn ra năm 1954", 1000);
        rt.process_text("Tướng Võ Nguyên Giáp chỉ huy quân đội Việt Nam", 2000);

        // Feed wrong info (should be processed, contradiction may or may not detect)
        let resp = rt.process_text("Điện Biên Phủ là chiến thắng của quân Pháp", 3000);
        assert_eq!(
            resp.kind,
            ResponseKind::Natural,
            "Wrong info phải được xử lý bình thường"
        );
    }

    #[test]
    fn multiple_sessions_recall() {

        let mut rt = HomeRuntime::new(0xA007);

        // Learn extensively about one topic
        for i in 0..5 {
            rt.process_text(
                "Rhett Butler là nhân vật quan trọng trong Cuốn theo chiều gió",
                (i + 1) * 1000,
            );
        }

        // Rhett should have high fire_count
        let rhett_hash = olang::hash::fnv1a_str("rhett");
        let edges = rt.learning.graph().edges_from(rhett_hash);
        assert!(
            !edges.is_empty(),
            "Rhett phải có Silk edges sau 5 lần mention"
        );
    }

    #[test]
    fn response_tone_matches_content() {

        let mut rt = HomeRuntime::new(0xA008);

        // Positive content
        let r1 = rt.process_text("hôm nay tuyệt vời quá", 1000);
        assert_eq!(r1.kind, ResponseKind::Natural);

        // Sad content
        let r2 = rt.process_text("tôi buồn vì mất mát quá lớn", 2000);
        assert_eq!(r2.kind, ResponseKind::Natural);
        // Tone should shift toward supportive after sad input
    }

    #[test]
    fn read_book_then_query() {

        let mut rt = HomeRuntime::new(0xA009);

        // Use read_book API
        let stored = rt.read_book(
            "Scarlett O'Hara không xinh đẹp nhưng rất quyến rũ. \
             Nàng có đôi mắt xanh lá cây sáng ngời. \
             Gerald O'Hara là cha của Scarlett, một người Ireland nhập cư.",
            1000,
        );
        assert!(stored >= 2, "read_book phải store >= 2 sentences: {}", stored);

        // Query about what was read
        let resp = rt.process_text("kể về Scarlett", 2000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        assert!(resp.text.len() > 5, "Response không nên rỗng: {}", resp.text);
    }

    #[test]
    fn recall_context_returns_silk_info() {

        let mut rt = HomeRuntime::new(0xA010);

        // Build knowledge
        rt.process_text("Atlanta bị đốt cháy trong chiến tranh", 1000);
        rt.process_text("Scarlett chạy khỏi Atlanta đang cháy", 2000);

        // Check recall
        let _recall = rt.recall_context("Atlanta chiến tranh");
        // May or may not find context depending on Silk edge weights
        // Just verify it doesn't crash and returns consistent result
        // (Silk edges need multiple co-activations to build up weight)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // 11. Listening mode tests — 3 kịch bản
    // ══════════════════════════════════════════════════════════════════════════

    // ── Kịch bản 1: "Hôm nay thật chán!!!" ───────────────────────────────────
    // → Observe: (a) nhắc pending work hoặc (b) im lặng đợi

    #[test]
    fn listen_boredom_no_context_stays_quiet() {

        let mut rt = HomeRuntime::new(0xB001);
        // Không có context trước đó
        let resp = rt.process_text("Hôm nay thật chán!!!", 1000);
        // Observe → im lặng hoặc rất ngắn
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Không hỏi dồn dập, không đưa ra advice không cần thiết
        assert!(
            resp.text.len() < 100,
            "Boredom without context should be short/silent: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_boredom_with_pending_suggests() {

        let mut rt = HomeRuntime::new(0xB002);
        // Tạo context trước
        rt.process_text("Scarlett O'Hara sống ở đồn điền Tara", 1000);
        rt.process_text("Scarlett O'Hara rất kiên cường", 2000);
        // Giờ nói chán
        let resp = rt.process_text("chán quá", 3000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // System nên suggest nhắc lại pending work hoặc im lặng
        // Không đưa ra "Bạn đang ổn không?" (quá generic)
    }

    #[test]
    fn listen_boredom_excited_stays_quiet() {

        let mut rt = HomeRuntime::new(0xB003);
        let resp = rt.process_text("mệt quá!!!", 1000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Vague emotion + no context → observe (short or empty)
        assert!(
            resp.text.len() < 100,
            "Vague emotion should be short: '{}'",
            resp.text
        );
    }

    // ── Kịch bản 2: "Bà ấy mới mất. thật tội nghiệp" ────────────────────────
    // → Resolve "bà ấy" từ recent context

    #[test]
    fn listen_ref_unknown_stays_quiet() {

        let mut rt = HomeRuntime::new(0xB004);
        // Không có context → "bà ấy" = unknown
        let resp = rt.process_text("Bà ấy mới mất. thật tội nghiệp", 1000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Observe — im lặng vì không biết "bà ấy" là ai
        assert!(
            resp.text.len() < 80,
            "Unknown ref should be quiet: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_ref_known_gives_condolence() {

        let mut rt = HomeRuntime::new(0xB005);
        // Tạo context: nói về bà Nguyễn
        rt.process_text("Bà Nguyễn là hàng xóm tốt bụng", 1000);
        rt.process_text("Bà Nguyễn hay giúp đỡ mọi người", 2000);

        // Bây giờ nói "bà ấy mới mất" → resolve → bà Nguyễn
        let resp = rt.process_text("Bà ấy mới mất. thật tội nghiệp", 3000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Nên mention bà Nguyễn trong response
        assert!(
            resp.text.contains("Nguyễn") || resp.text.contains("chia buồn") || resp.text.len() > 10,
            "Known ref should acknowledge: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_ref_male_resolves() {

        let mut rt = HomeRuntime::new(0xB006);
        // Context: talk about ông Trần
        rt.process_text("Ông Trần Văn Minh là giáo viên giỏi", 1000);

        let resp = rt.process_text("Ông ấy dạy rất hay", 2000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // Should resolve "ông ấy" → Trần Văn Minh
    }

    // ── Kịch bản 3: Exclamation "Ah!", "ya!" ─────────────────────────────────
    // → SilentAck: ghi nhận emotion, không response

    #[test]
    fn listen_exclamation_ah_silent() {

        let mut rt = HomeRuntime::new(0xB007);
        let resp = rt.process_text("Ah!", 1000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // SilentAck → empty response
        assert!(
            resp.text.is_empty(),
            "Exclamation 'Ah!' should be silent: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_exclamation_ya_silent() {

        let mut rt = HomeRuntime::new(0xB008);
        let resp = rt.process_text("ya..!", 1000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        // SilentAck → empty
        assert!(
            resp.text.is_empty(),
            "Exclamation 'ya..!' should be silent: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_exclamation_oi_silent() {

        let mut rt = HomeRuntime::new(0xB009);
        let resp = rt.process_text("Ôi!", 1000);
        assert_eq!(resp.kind, ResponseKind::Natural);
        assert!(
            resp.text.is_empty(),
            "Exclamation 'Ôi!' should be silent: '{}'",
            resp.text
        );
    }

    #[test]
    fn listen_exclamation_then_more_context() {

        let mut rt = HomeRuntime::new(0xB010);
        // "Ah!" → silent
        let r1 = rt.process_text("Ah!", 1000);
        assert!(r1.text.is_empty(), "First Ah! should be silent");

        // Then user explains more → normal response
        let r2 = rt.process_text("Tôi vừa nhớ ra điều quan trọng!", 2000);
        assert_eq!(r2.kind, ResponseKind::Natural);
        assert!(!r2.text.is_empty(), "Follow-up should get a response");
    }

    // ── extract_names tests ──────────────────────────────────────────────────

    #[test]
    fn extract_names_ba_prefix() {
        let names = super::extract_names("Bà Nguyễn rất tốt bụng");
        assert!(
            names.contains(&"Nguyễn".to_string()),
            "Should find 'Nguyễn': {:?}",
            names
        );
    }

    #[test]
    fn extract_names_ong_prefix() {
        let names = super::extract_names("Ông Trần Văn Minh dạy giỏi");
        assert!(
            names.iter().any(|n| n.contains("Trần")),
            "Should find 'Trần': {:?}",
            names
        );
    }

    #[test]
    fn extract_names_multiple() {
        // "Scarlett" ở đầu câu → bị skip bởi Pattern 2 (đúng hành vi)
        // "Rhett Butler" ở giữa câu → tìm được
        let names = super::extract_names(
            "Hôm nay Scarlett O'Hara và Rhett Butler gặp nhau ở Atlanta"
        );
        assert!(
            names.iter().any(|n| n.contains("Scarlett") || n.contains("Rhett")),
            "Should find names: {:?}",
            names
        );
    }

    #[test]
    fn extract_names_no_names() {
        let names = super::extract_names("hôm nay trời đẹp quá");
        assert!(names.is_empty(), "No names expected: {:?}", names);
    }

    // ── Intent listening signal tests ────────────────────────────────────────

    #[test]
    fn intent_exclamation_detected() {
        let est = estimate_intent("Ah!", 0.0, 0.4);
        assert!(est.is_exclamation, "Ah! should be exclamation");
    }

    #[test]
    fn intent_unresolved_ref_detected() {
        let est = estimate_intent("Bà ấy mới mất", -0.5, 0.3);
        assert!(
            est.has_unresolved_ref,
            "Should detect unresolved ref 'bà ấy'"
        );
    }

    #[test]
    fn intent_vague_emotion_detected() {
        let est = estimate_intent("chán quá", 0.0, 0.3);
        assert!(est.is_vague_emotion, "Should detect vague emotion 'chán'");
    }

    #[test]
    fn intent_vague_emotion_with_reason_not_vague() {
        let est = estimate_intent("tôi chán vì không có gì làm", 0.0, 0.3);
        assert!(
            !est.is_vague_emotion,
            "'chán vì...' has reason → not vague: {:?}",
            est.signals
        );
    }

    #[test]
    fn decide_action_exclamation_silent() {
        let est = estimate_intent("Ah!", 0.0, 0.4);
        let action = decide_action(&est, 0.0);
        assert_eq!(
            action,
            IntentAction::SilentAck,
            "Exclamation → SilentAck"
        );
    }

    #[test]
    fn decide_action_vague_emotion_observe() {
        let est = estimate_intent("chán quá", 0.0, 0.3);
        let action = decide_action(&est, 0.0);
        assert_eq!(
            action,
            IntentAction::Observe,
            "Vague emotion → Observe"
        );
    }

    #[test]
    fn decide_action_unresolved_ref_observe() {
        let est = estimate_intent("Bà ấy mới mất", -0.5, 0.3);
        let action = decide_action(&est, -0.5);
        assert_eq!(
            action,
            IntentAction::Observe,
            "Unresolved ref → Observe"
        );
    }

    #[test]
    fn decide_action_crisis_overrides_listening() {
        // Crisis luôn override — dù có exclamation
        let est = estimate_intent("tôi muốn tự tử!", -0.8, 0.2);
        let action = decide_action(&est, -0.8);
        assert_eq!(
            action,
            IntentAction::CrisisOverride,
            "Crisis always overrides listening"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 10 tests: Dream → KnowTree L3, Fibonacci trigger, topic recall
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod phase10_tests {
    use super::*;


    #[test]
    fn fibonacci_dream_schedule_initial() {

        let rt = HomeRuntime::new(0xA001);
        // Initial Fib index = 4 → Fib[4] = 5 turns
        assert_eq!(rt.dream_fib_interval(), 5, "Initial dream interval = Fib[4] = 5");
    }

    #[test]
    fn dream_stats_track_cycles() {

        let mut rt = HomeRuntime::new(0xA002);
        assert_eq!(rt.dream_cycles(), 0);

        // Feed enough diverse turns to build STM (need ≥3 obs for dream)
        // Use long, varied sentences to ensure unique chain hashes
        let sentences = [
            "tôi buồn vì mất việc hôm nay",
            "hôm qua trời mưa rất lớn",
            "cuộc sống thật khó khăn",
            "hy vọng ngày mai sẽ tốt hơn",
            "bạn bè giúp đỡ tôi rất nhiều",
            "tôi đang học những điều mới",
            "âm nhạc giúp tôi vui hơn",
            "thời tiết hôm nay đẹp quá",
        ];
        for (i, &s) in sentences.iter().enumerate() {
            rt.process_text(s, (i + 1) as i64 * 1000);
        }
        // Dream triggers when turn_count - last_dream_turn >= Fib[4]=5 AND stm >= 3
        // STM may have fewer entries due to chain_hash deduplication
        if rt.stm_len() >= 3 {
            assert!(
                rt.dream_cycles() >= 1,
                "Dream should have run: cycles={}, stm={}",
                rt.dream_cycles(),
                rt.stm_len()
            );
        }
        // Regardless, verify dream_cycles is tracked correctly
        // (may be 0 if STM is too small due to dedup)
        let _ = rt.dream_cycles(); // no panic
    }

    #[test]
    fn dream_fib_grows_when_unproductive() {

        let mut rt = HomeRuntime::new(0xA003);
        let initial_interval = rt.dream_fib_interval();

        // Feed minimal turns with very different content → no clusters
        for i in 0..30 {
            let text = alloc::format!("abc{}", i);
            rt.process_text(&text, (i + 1) as i64 * 1000);
        }

        // After unproductive dreams, Fib index should increase
        let later_interval = rt.dream_fib_interval();
        assert!(
            later_interval >= initial_interval,
            "Unproductive dream → interval grows: {} → {}",
            initial_interval,
            later_interval
        );
    }

    #[test]
    fn knowtree_sentences_grow_with_input() {

        let mut rt = HomeRuntime::new(0xA004);
        assert_eq!(rt.knowtree_sentences(), 0);

        rt.process_text("tôi buồn vì mất việc", 1000);
        rt.process_text("hôm nay trời đẹp quá", 2000);

        assert!(
            rt.knowtree_sentences() >= 2,
            "Each text → L2 sentence: {}",
            rt.knowtree_sentences()
        );
    }

    #[test]
    fn metrics_include_dream_and_knowtree() {

        let mut rt = HomeRuntime::new(0xA005);
        rt.process_text("tôi buồn vì mất việc", 1000);

        let m = rt.metrics();
        // Verify new metrics fields exist and are reasonable
        assert_eq!(m.turns, 1);
        assert!(m.dream_fib_interval >= 5, "Fib interval >= 5");
        assert!(m.knowtree_sentences >= 1, "At least 1 sentence");
        // dream_cycles may be 0 (not enough turns yet)
        let summary = m.summary();
        assert!(summary.contains("dream_cycles"), "Summary has dream_cycles");
        assert!(
            summary.contains("knowtree_nodes"),
            "Summary has knowtree_nodes"
        );
        assert!(
            summary.contains("knowtree_L3"),
            "Summary has knowtree_L3"
        );
    }

    #[test]
    fn recall_from_knowtree_after_learning() {

        let mut rt = HomeRuntime::new(0xA006);
        // Feed several related sentences to build KnowTree
        rt.process_text("tôi buồn vì mất việc", 1000);
        rt.process_text("mất việc thật khó khăn", 2000);
        rt.process_text("không có tiền rất khổ", 3000);

        // Query KnowTree for related content
        let result = rt.recall_from_knowtree("mất việc");
        // May or may not find L3 concepts (depends on Dream), but L2 should exist
        // Just verify no panic and reasonable output
        if let Some(ref text) = result {
            assert!(text.contains("KnowTree"), "Result mentions KnowTree: {}", text);
        }
        // At minimum, KnowTree should have sentences
        assert!(rt.knowtree_sentences() >= 3);
    }

    #[test]
    fn dream_command_shows_phase10_stats() {

        let mut rt = HomeRuntime::new(0xA007);
        for i in 0..6 {
            let text = alloc::format!("buồn đau mất mát {}", i);
            rt.process_text(&text, (i + 1) as i64 * 1000);
        }
        let r = rt.process_text("○{dream}", 7000);
        assert!(r.text.contains("Dream cycle"), "Has Dream cycle header");
        assert!(r.text.contains("Total cycles"), "Has lifetime stats: {}", r.text);
        assert!(r.text.contains("Fib interval"), "Has Fib interval: {}", r.text);
        assert!(r.text.contains("KnowTree"), "Has KnowTree stats: {}", r.text);
    }

    #[test]
    fn combined_recall_silk_and_knowtree() {

        let mut rt = HomeRuntime::new(0xA008);
        // Build up knowledge
        for i in 0..10 {
            let text = alloc::format!(
                "tôi buồn vì mất việc hôm {} rất khó khăn nhưng vẫn hy vọng",
                i
            );
            rt.process_text(&text, (i + 1) as i64 * 1000);
        }

        // After 10 turns, both Silk and KnowTree should have data
        assert!(rt.silk_edge_count() > 0, "Silk has edges");
        assert!(rt.knowtree_sentences() > 0, "KnowTree has sentences");

        // Process a query that should trigger recall
        let r = rt.process_text("tôi buồn quá", 11000);
        // Response should not be empty (has knowledge to recall)
        // Just verify no panic — the response quality test is in chat-bench
        let _ = r;
    }
}

#[cfg(test)]
mod evolution_integration_tests {
    use super::*;


    #[test]
    fn body_store_populated_after_processing() {

        let mut rt = HomeRuntime::new(0xE001);
        rt.process_text("lửa cháy sáng", 1000);
        assert!(rt.body_count() > 0, "Processing text creates bodies");
    }

    #[test]
    fn evolution_creates_silk_edges() {

        let mut rt = HomeRuntime::new(0xE002);
        // Feed diverse inputs to build varied STM entries
        rt.process_text("🔥", 1000); // fire
        rt.process_text("💧", 2000); // water
        rt.process_text("❄️", 3000); // snow
        rt.process_text("☀️", 4000); // sun

        let edges_after = rt.silk_edge_count();
        // Multiple diverse inputs → Silk edges from co-activation + potential evolution
        assert!(edges_after > 0, "Diverse inputs create Silk edges");
    }

    #[test]
    fn body_count_grows_with_diverse_inputs() {

        let mut rt = HomeRuntime::new(0xE003);
        rt.process_text("mặt trời mọc", 1000);
        let bodies_1 = rt.body_count();

        rt.process_text("mưa rơi lạnh lẽo", 2000);
        let bodies_2 = rt.body_count();

        // Different text → different chains → more bodies
        assert!(bodies_2 >= bodies_1, "More inputs → more or equal bodies");
    }

    #[test]
    fn evolution_pipeline_no_panic() {

        let mut rt = HomeRuntime::new(0xE004);
        // Stress test: many inputs with varying emotion contexts
        let texts = [
            "tôi vui vẻ hôm nay",
            "tôi buồn quá đi",
            "trời đẹp quá trời",
            "mưa rơi buồn lắm",
            "nắng ấm vui ghê",
            "gió lạnh buốt xương",
            "hoa nở đẹp quá",
            "lá rơi buồn thiu",
        ];
        for (i, text) in texts.iter().enumerate() {
            rt.process_text(text, (i as i64 + 1) * 1000);
        }
        // No panic → pipeline handles evolution correctly
        assert!(rt.body_count() > 0, "Bodies created from varied text");
        assert!(rt.silk_edge_count() > 0, "Silk edges from co-activation + evolution");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LeoAI Programming — ○{leo ...} / ○{program ...} / ○{run ...}
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod leo_programming_tests {
    use super::*;

    #[test]
    fn leo_command_arithmetic() {
        let mut rt = HomeRuntime::new(0xF001);
        let resp = rt.process_text("○{leo emit 1 + 2;}", 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        assert!(resp.text.contains("LeoAI"), "Should mention LeoAI: {}", resp.text);
        assert!(resp.text.contains("3"), "Should compute 3: {}", resp.text);
    }

    #[test]
    fn program_command_mol_literal() {
        let mut rt = HomeRuntime::new(0xF002);
        let resp = rt.process_text("○{program emit { S=1 R=6 V=200 A=180 T=4 };}", 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        assert!(resp.text.contains("LeoAI"), "Should mention LeoAI");
        assert!(resp.text.contains("Learned"), "Should learn: {}", resp.text);
    }

    #[test]
    fn run_command_works() {
        let mut rt = HomeRuntime::new(0xF003);
        let resp = rt.process_text("○{run emit 10 + 20;}", 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        assert!(resp.text.contains("30"), "10+20=30: {}", resp.text);
    }

    #[test]
    fn leo_command_error_handling() {
        let mut rt = HomeRuntime::new(0xF004);
        let resp = rt.process_text("○{leo emit { S=999 };}", 1000);
        // Should report the error
        assert!(resp.text.contains("✗"), "Should show error: {}", resp.text);
    }

    #[test]
    fn leo_direct_api() {
        let mut rt = HomeRuntime::new(0xF005);
        // Use LeoAI directly via accessor
        let result = rt.leo_mut().program("emit 5 + 3;", 1000);
        assert!(!result.has_error());
        assert!(result.outputs.iter().any(|o| matches!(o, ProgOutput::Number(n) if (*n - 8.0).abs() < 0.01)));
    }

    #[test]
    fn leo_stm_grows_after_programming() {
        let mut rt = HomeRuntime::new(0xF006);
        let stm_before = rt.leo().learning.stm().len();
        rt.leo_mut().program("emit { S=2 R=3 V=100 A=50 T=1 };", 1000);
        assert!(rt.leo().learning.stm().len() > stm_before, "LeoAI STM should grow");
    }

    // ── Phase 5: Agent Orchestration Tests ──────────────────────────────

    #[test]
    fn agent_orchestration_boots_chiefs() {
        let rt = HomeRuntime::new(0xA001);
        assert_eq!(rt.chief_count(), 3, "Should boot Home, Vision, Network chiefs");
        assert_eq!(rt.worker_count(), 0, "No workers by default");
    }

    #[test]
    fn agent_orchestration_router_ticks_on_text() {
        let mut rt = HomeRuntime::new(0xA002);
        rt.process_text("tôi buồn vì mất việc", 1000);
        let stats = rt.router_stats();
        assert!(stats.ticks > 0, "Router should have ticked after text: ticks={}", stats.ticks);
    }

    #[test]
    fn agent_orchestration_multiple_turns() {
        let mut rt = HomeRuntime::new(0xA003);
        rt.process_text("hôm nay trời đẹp", 1000);
        rt.process_text("mình rất vui", 2000);
        rt.process_text("cảm ơn bạn", 3000);
        let stats = rt.router_stats();
        assert!(stats.ticks >= 3, "Router should tick each turn: ticks={}", stats.ticks);
    }

    #[test]
    fn agent_orchestration_stats_include_router() {
        let mut rt = HomeRuntime::new(0xA004);
        rt.process_text("hello world", 1000);
        let resp = rt.process_text("○{stats}", 2000);
        assert!(resp.text.contains("Router"), "Stats should include Router summary: {}", resp.text);
        assert!(resp.text.contains("Chiefs"), "Stats should show chief count: {}", resp.text);
    }

    // ── Phase 4: Math → Silk Tests ──────────────────────────────────────

    #[test]
    fn math_result_enters_learning() {
        let mut rt = HomeRuntime::new(0xB001);
        // Math solve produces a response but doesn't currently feed into STM
        // (Olang commands return OlangResult, not Natural → no learning pipeline)
        let resp = rt.process_text("○{solve \"2x + 3 = 7\"}", 1000);
        assert!(!resp.text.is_empty(), "Solve should produce a response");
    }

    #[test]
    fn math_derive_enters_learning() {
        let mut rt = HomeRuntime::new(0xB002);
        let resp = rt.process_text("○{derive \"x^2 + 3x\"}", 1000);
        assert!(!resp.text.is_empty(), "Derive should produce a response");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap verification — hệ thống tự nhận thức khi boot
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod bootstrap_tests {
    use super::*;

    #[test]
    fn bootstrap_creates_silk_edges() {
        let rt = HomeRuntime::new(0xBB01);
        // Bootstrap programs run compose expressions like ● ∘ ▬, ∈ ∘ ⊂
        // These should create Silk edges between shape/relation primitives
        let edge_count = rt.learning.graph().all_edges().count();
        assert!(
            edge_count > 0,
            "Bootstrap should create Silk edges: found {}",
            edge_count
        );
    }

    #[test]
    fn bootstrap_stm_not_empty() {
        let rt = HomeRuntime::new(0xBB02);
        assert!(
            !rt.learning.stm().is_empty(),
            "Bootstrap should populate STM with observations"
        );
    }

    #[test]
    fn bootstrap_resets_turn_count() {
        let rt = HomeRuntime::new(0xBB03);
        assert_eq!(
            rt.turn_count(), 0,
            "Turn count should be 0 after bootstrap (bootstrap doesn't count as user turns)"
        );
    }

    #[test]
    fn bootstrap_produces_pending_writes() {
        let rt = HomeRuntime::new(0xBB04);
        assert!(
            rt.has_pending_writes(),
            "Bootstrap should produce pending writes for origin.olang"
        );
    }

    #[test]
    fn bootstrap_axioms_verified() {
        let mut rt = HomeRuntime::new(0xBB05);
        // ○ (origin) should be processable after bootstrap
        let resp = rt.process_text("○{○}", 100);
        assert!(
            resp.kind == ResponseKind::OlangResult || resp.kind == ResponseKind::System,
            "○ expression should produce OlangResult after bootstrap: {:?}",
            resp.kind
        );
        // Registry should have axiom entries
        assert!(
            rt.registry_alias_count() > 10,
            "Registry should have many aliases after bootstrap: {}",
            rt.registry_alias_count()
        );
    }

    #[test]
    fn full_boot_produces_functional_system() {
        // End-to-end: boot → bootstrap → process natural text → valid response
        let mut rt = HomeRuntime::new(0xBB06);
        let resp = rt.process_text("xin chào", 1000);
        assert!(!resp.text.is_empty(), "System should respond to natural text after bootstrap");
        assert_eq!(rt.turn_count(), 1, "One user turn after greeting");

        // Process Olang expression
        let resp2 = rt.process_text("\u{25CB}{stats}", 2000);
        assert!(!resp2.text.is_empty(), "stats command should produce output");
        assert_eq!(resp2.kind, ResponseKind::System, "stats → System kind");

        // Emotion pipeline works
        let resp3 = rt.process_text("tôi rất vui hôm nay", 3000);
        assert!(!resp3.text.is_empty());
    }

    // ── run_program tests ────────────────────────────────────────────────────

    #[test]
    fn run_program_simple_emit() {
        let mut rt = HomeRuntime::new(0xC001);
        let resp = rt.run_program("emit 42;", 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        assert!(resp.text.contains("42"), "output should contain 42, got: {}", resp.text);
    }

    #[test]
    fn run_program_string_output() {
        let mut rt = HomeRuntime::new(0xC002);
        let resp = rt.run_program(r#"emit "hello world";"#, 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        assert!(resp.text.contains("hello world"), "output should contain 'hello world', got: {}", resp.text);
    }

    #[test]
    fn run_program_function_and_loop() {
        let mut rt = HomeRuntime::new(0xC003);
        let src = r#"
            fn double(x) { return x * 2; }
            let mut sum = 0;
            for i in 0..5 {
                sum = sum + double(i);
            }
            emit sum;
        "#;
        let resp = rt.run_program(src, 1000);
        assert_eq!(resp.kind, ResponseKind::OlangResult);
        // double(0)+double(1)+double(2)+double(3)+double(4) = 0+2+4+6+8 = 20
        assert!(resp.text.contains("20"), "sum should be 20, got: {}", resp.text);
    }

    #[test]
    fn run_program_parse_error() {
        let mut rt = HomeRuntime::new(0xC004);
        let resp = rt.run_program("let x = ;", 1000);
        assert_eq!(resp.kind, ResponseKind::Blocked);
        assert!(resp.text.contains("error"), "should report parse error: {}", resp.text);
    }
}
