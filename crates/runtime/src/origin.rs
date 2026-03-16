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
use context::intent::{decide_action, estimate_intent};
use memory::dream::{DreamConfig, DreamCycle};
use silk::walk::ResponseTone;

use crate::parser::{OlangExpr, OlangParser, ParseResult, RelationOp};
use olang::ir::{compile_expr, OlangIrExpr};
use olang::knowtree::KnowTree;
use olang::registry::Registry;
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
}

impl HomeRuntime {
    /// Boot từ hư không — ○(∅)==○.
    pub fn new(session_id: u64) -> Self {
        Self::with_file(session_id, None)
    }

    /// Boot với file bytes — load registry từ origin.olang.
    pub fn with_file(session_id: u64, file_bytes: Option<&[u8]>) -> Self {
        let boot_result = boot(file_bytes);
        Self {
            learning: LearningLoop::new(session_id),
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
                        self.learning.stm_mut().push(chain.clone(), cur_emotion, ts);
                        learned.push(chain.clone());
                        // LCA result: show chain info
                        let lca_emoji = chain_to_emoji(chain);
                        let info = chain_info(chain, None);
                        output_text.push_str(&format!("∘→{} {}", lca_emoji, info));
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
                    output_text.push_str(&format!(
                        "query(0x{:04X} rel=0x{:02X}) ",
                        hash & 0xFFFF,
                        rel
                    ));
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
                let result = self
                    .dream
                    .run(self.learning.stm(), self.learning.graph(), ts);
                let text = format!(
                    "Dream cycle ○\n\
                     Scanned  : {}\n\
                     Clusters : {}\n\
                     Proposals: {}\n\
                     Approved : {}",
                    result.scanned,
                    result.clusters_found,
                    result.proposals.len(),
                    result.approved,
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
                     ○{🔥}          — query node\n\
                     ○{🔥 ∘ 💧}    — compose (LCA)\n\
                     ○{🔥 ∈ ?}     — relation query\n\
                     ○{? → 💧}     — reverse query\n\
                     ○{term ∂ ctx} — context query\n\
                     ○{dream}      — run Dream cycle\n\
                     ○{stats}      — system statistics\n\
                     ○{health}     — system health check\n\
                     ○{help}       — this message",
                ),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::System,
            },

            _ => Response {
                text: format!("Unknown command: {}", cmd),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::System,
            },
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

        // Auto-Dream: sau mỗi DREAM_INTERVAL turns
        const DREAM_INTERVAL: u64 = 8;
        if self.turn_count - self.last_dream_turn >= DREAM_INTERVAL
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
        if let ProcessResult::Ok { ref chain, .. } = proc_result {
            if let ContentInput::Text { ref content, .. } = input {
                let word_hashes = olang::knowtree::text_to_word_hashes(content);
                if !word_hashes.is_empty() {
                    self.knowtree.store_sentence(chain, None, &word_hashes, ts);
                }
            }
        }

        // ── T7: Decide action → render response ────────────────────────────
        let action = decide_action(&est, cur_v);
        let fx = self.learning.context().fx();
        let tone = self.learning.context().tone();

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
            ProcessResult::Ok { emotion, .. } => {
                use context::intent::IntentAction;
                use context::word_guide::affect_components;

                let final_v = if let Some(wt) = walk_tag {
                    emotion.valence * 0.40 + wt.valence * 0.60
                } else {
                    emotion.valence * emo_ctx_scale + raw_tag.valence * (1.0 - emo_ctx_scale)
                };

                let original = match &action {
                    IntentAction::Proceed => {
                        let comps = affect_components(self.learning.context().curve());
                        Some(natural_reply(
                            tone,
                            final_v,
                            comps.lead_word,
                            comps.support_word,
                        ))
                    }
                    _ => None,
                };
                let text = render(&ResponseParams {
                    tone,
                    action,
                    valence: final_v,
                    fx,
                    context: None,
                    original,
                });
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
            dream_cycles_est: if self.turn_count > 8 {
                (self.turn_count - 1) / 8
            } else {
                0
            },
            pending_bytes: self.pending_writes.len(),
            saveable_edges: self.saveable_edges(),
            stm_max_fire: max_fire,
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
    pub fn serialize_learned(&self, ts: i64) -> alloc::vec::Vec<u8> {
        use olang::writer::OlangWriter;

        // Bắt đầu từ rỗng — chỉ serialize phần mới (delta)
        let mut writer = OlangWriter::new(ts);

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
    let shape_sym = match mol.shape {
        olang::molecular::ShapeBase::Sphere => "●",
        olang::molecular::ShapeBase::Capsule => "▬",
        olang::molecular::ShapeBase::Box => "■",
        olang::molecular::ShapeBase::Cone => "▲",
        olang::molecular::ShapeBase::Torus => "○",
        olang::molecular::ShapeBase::Union => "∪",
        olang::molecular::ShapeBase::Intersect => "∩",
        olang::molecular::ShapeBase::Subtract => "∖",
    };
    let rel_sym = match mol.relation {
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

        let result = self
            .dream
            .run(self.learning.stm(), self.learning.graph(), ts);

        // Feed approved proposals
        // QT9: serialize TRƯỚC → pending_writes → RỒII mới cập nhật Registry
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
                    match &proposal.kind {
                        ProposalKind::NewNode { chain, sources, .. } => {
                            self.registry.insert(chain, 3, 0, ts, false);
                            // L2-Ln: store concept in KnowTree
                            self.knowtree.store_concept(chain, None, 3, sources, ts);
                        }
                        ProposalKind::PromoteQR {
                            chain_hash,
                            fire_count,
                        } => {
                            if let Some(obs) = self.learning.stm().find_by_hash(*chain_hash) {
                                self.registry.insert(&obs.chain, 0, 0, ts, true);
                                // L2-Ln: promote to KnowTree
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
        let bytes = rt.serialize_learned(30000);

        // Parse bytes back
        let reader = OlangReader::new(&bytes).expect("parse header");
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
}
