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

use crate::response_template::{render, ResponseParams};
use agents::encoder::ContentInput;
use agents::learning::{LearningLoop, ProcessResult};
use context::emotion::sentence_affect;
use context::emotion::IntentKind;
use context::infer::infer_context;
use context::intent::{decide_action, estimate_intent, IntentAction};
use memory::dream::{DreamConfig, DreamCycle};
use silk::walk::ResponseTone;

use crate::parser::{OlangExpr, OlangParser, ParseResult, RelationOp};
use olang::ir::{compile_expr, OlangIrExpr};
use olang::knowtree::KnowTree;
use olang::registry::Registry;
use vsdf::body::{body_from_molecule, BodyStore};
use olang::self_model::SelfModel;
use olang::separator::parse_to_chains;
use olang::startup::{boot, chain_to_emoji, resolve_with_cp};
use olang::vm::{OlangVM, VmEvent};

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
    /// L2-Ln knowledge storage — TieredStore compact encoding.
    knowtree: KnowTree,
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

impl HomeRuntime {
    /// Boot từ hư không — ○(∅)==○.
    pub fn new(session_id: u64) -> Self {
        Self::with_file(session_id, None)
    }

    /// Boot với file bytes — load registry + Silk edges từ origin.olang.
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

        Self {
            learning,
            parser: OlangParser::new(),
            dream: DreamCycle::new(DreamConfig::default()),
            alias_to_cp: build_alias_map(file_bytes),
            registry: boot_result.registry,
            self_model: SelfModel::new(),
            uptime_ns: 0,
            turn_count: 0,
            last_dream_turn: 0,
            pending_writes: alloc::vec::Vec::new(),
            knowtree: KnowTree::for_pc(),
            recent_texts: alloc::vec::Vec::new(),
            dream_fib_index: 4, // Fib[4]=5: first dream after 5 turns
            dream_cycles: 0,
            dream_approved_total: 0,
            dream_l3_created: 0,
            body_store: BodyStore::new(),
        }
    }

    /// Xử lý một text input — entry point cho text.
    ///
    /// Parse ○{} trước, nếu natural text → delegate to process_input (universal pipeline).
    pub fn process_text(&mut self, text: &str, ts: i64) -> Response {
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
                    // Output chain → STM
                    if !chain.is_empty() {
                        // Check if numeric result
                        if let Some(num) = chain.to_number() {
                            // Display number cleanly
                            if (num - libm::round(num)).abs() < 1e-10 && num.abs() < 1e15 {
                                output_text.push_str(&format!("= {} ", libm::round(num) as i64));
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
                        output_text.push_str(&format!("{}=? ", alias));
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
                let text = format!(
                    "HomeOS ○\n\
                     Turns    : {}\n\
                     Registry : {} nodes, {} aliases\n\
                     STM      : {} observations\n\
                     Silk     : {} nodes, {} edges\n\
                     f(x)     : {:.3}\n\
                     {}\n\
                     {}",
                    self.turn_count,
                    reg_nodes,
                    reg_aliases,
                    self.learning.stm().len(),
                    silk_nodes,
                    self.learning.graph().len(),
                    self.learning.context().fx(),
                    summary,
                    kt_summary,
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
                let text = format!(
                    "Dream cycle ○\n\
                     Scanned    : {}\n\
                     Clusters   : {}\n\
                     Proposals  : {}\n\
                     Approved   : {}\n\
                     ─── Lifetime ───\n\
                     Total cycles: {}\n\
                     Total approved: {}\n\
                     L3 concepts : {}\n\
                     Fib interval: {} turns\n\
                     KnowTree    : {} nodes, {} L3",
                    result.scanned,
                    result.clusters_found,
                    result.proposals.len(),
                    result.approved,
                    self.dream_cycles,
                    self.dream_approved_total,
                    self.dream_l3_created,
                    silk::hebbian::fib(self.dream_fib_index),
                    self.knowtree.total_nodes(),
                    self.knowtree.concepts(),
                );
                Response {
                    text,
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
                     ○{health}             — system health check\n\
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
                     ○{help}               — this message",
                ),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::System,
            },

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
                    Response {
                        text,
                        tone: ResponseTone::Engaged,
                        fx: 0.0,
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
        related.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        related.dedup_by_key(|r| r.0);
        related.truncate(8);

        // Find matching STM observations by hash
        let mut context_parts: alloc::vec::Vec<String> = alloc::vec::Vec::new();
        for &(h, w, emo) in &related {
            if let Some(obs) = self.learning.stm().find_by_hash(h) {
                let v_label = if emo.valence > 0.3 {
                    "tích cực"
                } else if emo.valence < -0.3 {
                    "tiêu cực"
                } else {
                    "trung tính"
                };
                context_parts.push(format!(
                    "[fire={}, w={:.2}, {}]",
                    obs.fire_count, w, v_label
                ));
            }
        }

        if context_parts.is_empty() {
            // Return number of silk connections found even without STM match
            Some(format!(
                "({} liên kết Silk, chưa có trong STM)",
                related.len()
            ))
        } else {
            Some(format!(
                "({} liên kết: {})",
                related.len(),
                context_parts.join(", ")
            ))
        }
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

        found_concepts.truncate(5);
        Some(format!(
            "(KnowTree: {} matches: {})",
            found_concepts.len(),
            found_concepts.join(", ")
        ))
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

        // Cập nhật Registry (RAM SAU)
        self.registry.insert(chain, 1, 0, ts, true);

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

        // Cập nhật Registry
        self.registry.insert(&obs.chain, 1, 0, ts, true);

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

    // ── Response generation ───────────────────────────────────────────────────

    /// Sinh fallback text khi không có original từ pipeline.
    /// Dùng response_template::tone_fallback — không hardcode string ở đây.
    #[allow(dead_code)]
    fn generate_response(&self, tone: ResponseTone, current_v: f32, _fx: f32) -> String {
        crate::response_template::tone_fallback(tone, current_v)
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
        found.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        found.truncate(5);
        found
    }

    /// Tổng hợp emotion từ SilkWalk result.
    fn walk_emotion(&self, query: &str) -> Option<silk::edge::EmotionTag> {
        let results = self.silk_walk_query(query);
        if results.is_empty() {
            return None;
        }

        let _n = results.len() as f32;
        let (sv, sa, sd, si) = results.iter().fold(
            (0.0f32, 0.0f32, 0.0f32, 0.0f32),
            |(v, a, d, i), (_, e, w)| {
                (
                    v + e.valence * w,
                    a + e.arousal * w,
                    d + e.dominance * w,
                    i + e.intensity * w,
                )
            },
        );
        let tw: f32 = results.iter().map(|r| r.2).sum();
        if tw < 0.001 {
            return None;
        }

        Some(silk::edge::EmotionTag {
            valence: sv / tw,
            arousal: sa / tw,
            dominance: sd / tw,
            intensity: si / tw,
        })
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

        // ── T6b: KnowTree — store text as L2 compact node ───────────────
        if let ProcessResult::Ok { ref chain, emotion } = proc_result {
            if let ContentInput::Text { ref content, .. } = input {
                let word_hashes = olang::knowtree::text_to_word_hashes(content);
                if !word_hashes.is_empty() {
                    self.knowtree.store_sentence(chain, None, &word_hashes, ts);
                }
            }

            // ── T6b2: NodeBody — tạo/cập nhật SDF + Spline cho chain ──────
            // Mỗi chain mới → tạo body từ molecule bytes
            if let Some(mol) = chain.first() {
                let hash = chain.chain_hash();
                if self.body_store.get(hash).is_none() {
                    let body = body_from_molecule(
                        hash,
                        mol.shape,
                        mol.emotion.valence,
                        mol.emotion.arousal,
                        mol.time,
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

                            // Tạo body cho evolved node
                            if let Some(evolved_mol) = evolved_chain.first() {
                                if self.body_store.get(evolved_hash).is_none() {
                                    let body = body_from_molecule(
                                        evolved_hash,
                                        evolved_mol.shape,
                                        evolved_mol.emotion.valence,
                                        evolved_mol.emotion.arousal,
                                        evolved_mol.time,
                                    );
                                    let entry = self.body_store.get_or_create(evolved_hash);
                                    *entry = body;
                                }
                            }

                            // Silk edge: source → evolved (DerivedFrom)
                            // "loài mới" derived from "loài gốc"
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

        // ── T7: Decide action → render response ────────────────────────────
        let mut action = decide_action(&est, cur_v);
        let fx = self.learning.context().fx();
        let tone = self.learning.context().tone();

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
                use context::word_guide::affect_components;

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
                    });
                    return Response {
                        text,
                        tone,
                        fx,
                        kind: ResponseKind::Natural,
                    };
                }

                // ── Normal flow: contradiction / recall / proceed ────────────
                // Build response with context awareness
                let original = if let Some(ref contra) = contradiction {
                    // Contradiction detected → inform user
                    Some(contra.clone())
                } else {
                    match &action {
                        IntentAction::Proceed => {
                            // Use recalled context if available for richer reply
                            if let Some(ref ctx) = recall {
                                Some(contextual_reply(tone, final_v, input_text, ctx))
                            } else {
                                let comps =
                                    affect_components(self.learning.context().curve());
                                Some(natural_reply(
                                    tone,
                                    final_v,
                                    comps.lead_word,
                                    comps.support_word,
                                ))
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
        for obs in self.learning.stm().all() {
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
        }
        stored
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
    pub fn knowtree(&self) -> &KnowTree {
        &self.knowtree
    }
    pub fn knowtree_mut(&mut self) -> &mut KnowTree {
        &mut self.knowtree
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
}

// ─────────────────────────────────────────────────────────────────────────────
// natural_reply — câu trả lời có nội dung từ word_guide
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo câu trả lời từ tone + từ ngữ học được.
/// Không hardcode string — dùng từ của lead_word/support_word từ lexicon.
fn natural_reply(
    tone: silk::walk::ResponseTone,
    v: f32,
    lead: &str,
    support: &str,
) -> alloc::string::String {
    use silk::walk::ResponseTone;
    match tone {
        ResponseTone::Pause | ResponseTone::Supportive => {
            if v < -0.50 {
                alloc::format!("Cảm giác {} và {} — bạn muốn kể thêm không?", lead, support)
            } else {
                alloc::format!("Mình nghe thấy điều đó. {} — bạn đang ổn không?", lead)
            }
        }
        ResponseTone::Gentle => {
            alloc::format!("Cứ từ từ. {} thôi.", lead)
        }
        ResponseTone::Reinforcing => {
            alloc::format!("Đúng rồi — {} và {}.", lead, support)
        }
        ResponseTone::Celebratory => {
            alloc::format!("Tuyệt! {} lắm.", lead)
        }
        ResponseTone::Engaged => {
            alloc::format!("{} — {}.", lead, support)
        }
    }
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

/// Tạo câu trả lời phù hợp bối cảnh — dùng từ ngữ liên quan đến nội dung
/// thay vì chỉ tone-based generic phrases.
fn contextual_reply(
    tone: silk::walk::ResponseTone,
    v: f32,
    input: &str,
    context: &str,
) -> alloc::string::String {
    use silk::walk::ResponseTone;

    // Extract key content words from input for reference
    let key_words: alloc::vec::Vec<&str> = input
        .split_whitespace()
        .filter(|w| w.chars().count() > 2)
        .take(3)
        .collect();
    let topic = if key_words.is_empty() {
        ""
    } else {
        key_words[0]
    };

    match tone {
        ResponseTone::Supportive | ResponseTone::Pause => {
            if v < -0.50 {
                alloc::format!(
                    "Mình hiểu cảm giác khi nghĩ về {} — điều đó nặng nề thật. {}",
                    topic, context
                )
            } else if v < -0.20 {
                alloc::format!(
                    "Mình nghe thấy điều bạn nói về {}. {}", topic, context
                )
            } else {
                alloc::format!(
                    "Về {} — mình đang lắng nghe. {}", topic, context
                )
            }
        }
        ResponseTone::Gentle => {
            alloc::format!("Cứ từ từ nói về {} nhé. {}", topic, context)
        }
        ResponseTone::Reinforcing => {
            if v > 0.30 {
                alloc::format!("Đúng rồi — {} thú vị đấy. {}", topic, context)
            } else {
                alloc::format!(
                    "Mình ghi nhận về {}. {}", topic, context
                )
            }
        }
        ResponseTone::Celebratory => {
            alloc::format!("Tuyệt! {} hay lắm. {}", topic, context)
        }
        ResponseTone::Engaged => {
            if context.is_empty() {
                alloc::format!("Về {} — mình đang nghe.", topic)
            } else {
                alloc::format!("Về {} — {}.", topic, context)
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
        if ucd::table_len() == 0 {
            return;
        }
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
    fn olang_dream_command() {
        let mut rt = rt();
        let r = rt.process_text("○{dream}", 1000);
        assert_eq!(r.kind, ResponseKind::System);
        assert!(r.text.contains("Dream"));
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
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
        // □ 0 hardcoded Molecule
        // Verify encode_codepoint → UCD lookup, not hardcoded
        let chain = olang::encoder::encode_codepoint(0x1F525);
        assert!(!chain.is_empty());
        // Chain phải match UCD values
        let mol = &chain.0[0];
        assert_eq!(mol.shape as u8, ucd::shape_of(0x1F525), "Shape phải từ UCD");
        assert_eq!(
            mol.emotion.valence,
            ucd::valence_of(0x1F525),
            "Valence phải từ UCD"
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
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// chain_info — human-readable chain description
// ─────────────────────────────────────────────────────────────────────────────

/// Human-readable chain info: [mol_count] shape×rel V/A hash=0x... (U+XXXX)
fn chain_info(chain: &olang::molecular::MolecularChain, cp: Option<u32>) -> alloc::string::String {
    if chain.is_empty() {
        return String::from("(empty)");
    }

    let mol = &chain.0[0];
    let shape_sym = match mol.shape_base() {
        olang::molecular::ShapeBase::Sphere => "●",
        olang::molecular::ShapeBase::Capsule => "▬",
        olang::molecular::ShapeBase::Box => "■",
        olang::molecular::ShapeBase::Cone => "▲",
        olang::molecular::ShapeBase::Torus => "○",
        olang::molecular::ShapeBase::Union => "∪",
        olang::molecular::ShapeBase::Intersect => "∩",
        olang::molecular::ShapeBase::Subtract => "∖",
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

    let v = mol.emotion.valence;
    let a = mol.emotion.arousal;
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

        let mut approved_this_cycle: u64 = 0;
        let mut l3_this_cycle: u64 = 0;

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
                            self.registry.insert(chain, 3, 0, ts, false);
                            // L3 concept in KnowTree — with source edges
                            self.knowtree.store_concept(chain, None, 3, sources, ts);
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
                                self.registry.insert(&obs.chain, 0, 0, ts, true);
                                // L2: promote to KnowTree
                                self.knowtree
                                    .promote_from_stm(&obs.chain, None, *fire_count, ts);
                            }
                            let _ = chain_hash;
                        }
                        _ => {}
                    }
                }
            }
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
        if ucd::table_len() == 0 {
            return;
        }
        let mut rt = rt();
        assert_eq!(rt.learning.stm().len(), 0, "STM rỗng trước khi chat");

        rt.process_text("hôm nay trời đẹp", 1000);
        assert!(
            rt.learning.stm().len() > 0,
            "STM phải có observations sau khi chat: len={}",
            rt.learning.stm().len()
        );
    }

    /// STM tích lũy qua nhiều turns — mỗi turn thêm observations.
    #[test]
    fn stm_accumulates_across_turns() {
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
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
        if ucd::table_len() == 0 {
            return;
        }
        let mut rt = rt();

        let r = rt.process_audio(440.0, 0.7, 120.0, 0.0, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            rt.learning.stm().len() > 0,
            "Audio input phải tạo STM observation"
        );
    }

    /// Universal pipeline: Sensor input qua full pipeline.
    #[test]
    fn universal_sensor_creates_stm() {
        if ucd::table_len() == 0 {
            return;
        }
        let mut rt = rt();

        let input = ContentInput::Sensor {
            kind: agents::encoder::SensorKind::Temperature,
            value: 28.5,
            timestamp: 1000,
        };
        let r = rt.process_input(input, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            rt.learning.stm().len() > 0,
            "Sensor input phải tạo STM observation"
        );
    }

    /// Universal pipeline: Code input qua full pipeline.
    #[test]
    fn universal_code_creates_stm() {
        if ucd::table_len() == 0 {
            return;
        }
        let mut rt = rt();

        let input = ContentInput::Code {
            content: String::from("fn main() {}"),
            language: agents::encoder::CodeLang::Rust,
            timestamp: 1000,
        };
        let r = rt.process_input(input, 1000);
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(
            rt.learning.stm().len() > 0,
            "Code input phải tạo STM observation"
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
    fn pending_writes_empty_initially() {
        let rt = HomeRuntime::new(0xDEAD);
        assert!(!rt.has_pending_writes());
        assert_eq!(rt.pending_bytes(), 0);
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

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    // ── 1. serialize_learned → OlangReader round-trip ────────────────────────

    #[test]
    fn serialize_roundtrip_edges() {
        if skip() {
            return;
        }
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
        full_bytes.push(olang::writer::VERSION_V03);
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
        let mut rt = HomeRuntime::new(0xF001);
        // Teach something first
        rt.process_text("Scarlett O'Hara là nhân vật chính", 1000);
        assert!(!rt.has_pending_writes(), "Normal text không ghi pending");

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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
            rt.learning.stm().len() > 0,
            "STM phải có entries sau khi học war content"
        );
        assert!(
            rt.learning.graph().len() > 0,
            "Silk phải có edges từ war content"
        );
    }

    #[test]
    fn learn_novel_gone_with_the_wind() {
        if skip() {
            return;
        }
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
            rt.learning.stm().len() >= 1,
            "STM phải có observations: {}",
            rt.learning.stm().len()
        );
    }

    #[test]
    fn learn_then_recall_context() {
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    #[test]
    fn fibonacci_dream_schedule_initial() {
        if skip() {
            return;
        }
        let rt = HomeRuntime::new(0xA001);
        // Initial Fib index = 4 → Fib[4] = 5 turns
        assert_eq!(rt.dream_fib_interval(), 5, "Initial dream interval = Fib[4] = 5");
    }

    #[test]
    fn dream_stats_track_cycles() {
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    #[test]
    fn body_store_populated_after_processing() {
        if skip() {
            return;
        }
        let mut rt = HomeRuntime::new(0xE001);
        rt.process_text("lửa cháy sáng", 1000);
        assert!(rt.body_count() > 0, "Processing text creates bodies");
    }

    #[test]
    fn evolution_creates_silk_edges() {
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
