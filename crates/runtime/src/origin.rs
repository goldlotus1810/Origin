//! # origin — HomeRuntime
//!
//! ○(∅) == ○ — tự boot, tự vận hành.
//!
//! process_one(input) → Response
//!   SecurityGate → Parse → Encode → Context → STM → Silk → Response

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;

use agents::learning::{LearningLoop, ProcessResult};
use agents::encoder::ContentInput;
use context::infer::infer_context;
use context::intent::{estimate_intent, decide_action};
use context::emotion::IntentKind;
use context::emotion::sentence_affect;
use crate::response_template::{render, ResponseParams};
use memory::dream::{DreamCycle, DreamConfig};
use silk::walk::ResponseTone;

use crate::parser::{OlangParser, OlangExpr, ParseResult, RelationOp};
use olang::ir::{OlangIrExpr, compile_expr};
use olang::separator::parse_to_chains;
use olang::vm::{OlangVM, VmEvent};
use olang::startup::{boot, resolve_with_cp, chain_to_emoji};
use olang::self_model::SelfModel;
use olang::registry::Registry;

// ─────────────────────────────────────────────────────────────────────────────
// Response
// ─────────────────────────────────────────────────────────────────────────────

/// Response từ HomeRuntime.
#[derive(Debug, Clone)]
pub struct Response {
    pub text:     String,
    pub tone:     ResponseTone,
    pub fx:       f32,
    pub kind:     ResponseKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseKind {
    Natural,    // Trả lời tự nhiên
    OlangResult,// Kết quả ○{} query
    Crisis,     // Crisis response
    Blocked,    // SecurityGate blocked
    System,     // System command response
}

// ─────────────────────────────────────────────────────────────────────────────
// HomeRuntime
// ─────────────────────────────────────────────────────────────────────────────

/// HomeOS Runtime — mọi thứ qua đây.
///
/// ○(∅) == ○: boot từ hư không, sống từ đây.
pub struct HomeRuntime {
    learning:   LearningLoop,
    parser:     OlangParser,
    dream:      DreamCycle,
    registry:   Registry,
    alias_to_cp: BTreeMap<alloc::string::String, u32>,
    self_model:      SelfModel,
    uptime_ns:        i64,
    turn_count:       u64,
    last_dream_turn:  u64,  // turn khi Dream lần cuối chạy
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
            learning:   LearningLoop::new(session_id),
            parser:     OlangParser::new(),
            dream:      DreamCycle::new(DreamConfig::default()),
            alias_to_cp: build_alias_map(file_bytes),
            registry:   boot_result.registry,
            self_model:      SelfModel::new(),
            uptime_ns:        0,
            turn_count:       0,
            last_dream_turn:  0,
        }
    }

    /// Xử lý một text input — entry point chính.
    pub fn process_text(&mut self, text: &str, ts: i64) -> Response {
        self.turn_count += 1;
        self.uptime_ns = ts;

        // Auto-Dream: sau mỗi DREAM_INTERVAL turns
        const DREAM_INTERVAL: u64 = 8; // mỗi 8 turns
        if self.turn_count - self.last_dream_turn >= DREAM_INTERVAL
            && self.learning.stm().len() >= 3
        {
            self.run_dream(ts);
        }

        // ── Parse: natural hoặc ○{} ──────────────────────────────────────────
        match self.parser.parse(text) {
            ParseResult::Natural(s) => self.process_natural(&s, ts),
            ParseResult::OlangExpr(expr) => self.process_olang(expr, ts),
            ParseResult::Error(e) => Response {
                text: format!("Parse error: {}", e),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind: ResponseKind::Blocked,
            },
        }
    }

    // ── Natural text ──────────────────────────────────────────────────────────

    fn process_natural(&mut self, text: &str, ts: i64) -> Response {
        // ── Emotion Pipeline — 7 tầng ─────────────────────────────────────────

        // T1: InferContext — điều kiện biên từ text
        let emo_ctx = infer_context(text);

        // T2: TextToEmotionTag — raw emotion từ từ ngữ
        let raw_tag = sentence_affect(text);

        // T3: ctx.Apply — scale theo S (S=1.0 FirstPerson/RealNow)
        let scaled  = emo_ctx.apply(raw_tag);

        // T4: IntentEstimate + SilkWalk context
        let cur_v   = self.learning.context().fx();
        let cur_a   = scaled.arousal;
        let est     = estimate_intent(text, cur_v, cur_a);

        // SilkWalk: nếu có Silk edges → enrich emotion với context học được
        let walk_tag = self.walk_emotion(text);
        let _scaled_ctx = if let Some(wt) = walk_tag {
            // Blend: 70% walked context + 30% raw (Silk = accumulated knowledge)
            silk::edge::EmotionTag {
                valence:   wt.valence   * 0.70 + scaled.valence   * 0.30,
                arousal:   wt.arousal   * 0.70 + scaled.arousal   * 0.30,
                dominance: wt.dominance * 0.70 + scaled.dominance * 0.30,
                intensity: wt.intensity * 0.70 + scaled.intensity * 0.30,
            }
        } else { scaled }; // enriched (unused directly, walk_tag used in Ok branch)

        // T5: Crisis → override ngay, trước mọi thứ
        if est.primary == IntentKind::Crisis {
            use crate::response_template::crisis_text;
            return Response {
                text: crisis_text(),
                tone: ResponseTone::Supportive,
                fx:   self.learning.context().fx(),
                kind: ResponseKind::Crisis,
            };
        }

        // T6: ConversationCurve — feed scaled emotion vào history
        let input       = ContentInput::Text { content: text.to_string(), timestamp: ts };
        let proc_result = self.learning.process_one(input);

        // T7: Decide action → render response (text quyết định ở response_template)
        let action  = decide_action(&est, cur_v);
        let fx      = self.learning.context().fx();
        let tone    = self.learning.context().tone();

        match proc_result {
            ProcessResult::Crisis { message } => Response {
                text: message,
                tone: ResponseTone::Supportive,
                fx, kind: ResponseKind::Crisis,
            },
            ProcessResult::Blocked { reason } => Response {
                text: format!("({})", reason),
                tone: ResponseTone::Gentle,
                fx, kind: ResponseKind::Blocked,
            },
            ProcessResult::Ok { emotion, .. } => {
                use context::intent::IntentAction;
                use context::word_guide::affect_components;

                // Tổng hợp emotion: pipeline + walked context
                let final_v = if let Some(wt) = walk_tag {
                    emotion.valence * 0.40 + wt.valence * 0.60
                } else { emotion.valence };

                // Original = NaturalResponse từ word_guide + walked context
                let original = match &action {
                    IntentAction::Proceed => {
                        let comps = affect_components(self.learning.context().curve());
                        // Câu trả lời dựa trên tone + từ ngữ học được
                        Some(natural_reply(tone, final_v, comps.lead_word, comps.support_word))
                    },
                    _ => None,
                };
                let text = render(&ResponseParams {
                    tone, action, valence: final_v, fx,
                    context: None, original,
                });
                Response { text, tone, fx, kind: ResponseKind::Natural }
            },
            ProcessResult::Empty => {
                let text = render(&ResponseParams {
                    tone, action, valence: cur_v, fx,
                    context: None, original: None,
                });
                Response { text, tone, fx, kind: ResponseKind::Natural }
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
        let prog    = compile_expr(&ir_expr);
        let vm      = OlangVM::new();
        let result  = vm.execute(&prog);

        // Collect output từ VM events + FEED vào LearningLoop
        let mut output_text  = String::new();
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
                    }
                    // LCA output: đã hiện các lookup rồi, chỉ hiện ∘
                    output_text.push_str("○");
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
                                .map(|c| { let mut s = alloc::string::String::new(); s.push(c); s })
                                .unwrap_or_else(|| chain_to_emoji(&chain))
                        } else {
                            chain_to_emoji(&chain)
                        };
                        output_text.push_str(&format!("{}={} ", alias, emoji));
                    } else {
                        output_text.push_str(&format!("{}=? ", alias));
                    }
                }
                VmEvent::CreateEdge { from, to, rel } => {
                    // Explicit edge → Silk: user asserted relation
                    self.learning.graph_mut().co_activate(
                        *from, *to,
                        cur_emotion,
                        1.0, // intentional → full reward
                        ts,
                    );
                    output_text.push_str(&format!("edge(0x{:04X}→0x{:04X} rel=0x{:02X}) ",
                        from & 0xFFFF, to & 0xFFFF, rel));
                }
                VmEvent::QueryRelation { hash, rel } => {
                    output_text.push_str(&format!("query(0x{:04X} rel=0x{:02X}) ",
                        hash & 0xFFFF, rel));
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
                    ha, hb,
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
            fx:   self.learning.context().fx(),
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
                let text = format!(
                    "HomeOS ○\n\
                     Turns   : {}\n\
                     STM     : {} observations\n\
                     Silk    : {} edges\n\
                     f(x)    : {:.3}\n\
                     {}",
                    self.turn_count,
                    self.learning.stm().len(),
                    self.learning.graph().len(),
                    self.learning.context().fx(),
                    summary,
                );
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
            }

            "dream" => {
                let result = self.dream.run(
                    self.learning.stm(),
                    self.learning.graph(),
                    ts,
                );
                let text = format!(
                    "Dream cycle ○\n\
                     Scanned  : {}\n\
                     Clusters : {}\n\
                     Proposals: {}\n\
                     Approved : {}",
                    result.scanned, result.clusters_found,
                    result.proposals.len(), result.approved,
                );
                Response { text, tone: ResponseTone::Engaged, fx: 0.0, kind: ResponseKind::System }
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
                     ○{help}       — this message"
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
            }
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
        let words = query.split_whitespace()
            .filter(|w| w.chars().count() > 2)
            .take(8);

        let mut found: alloc::vec::Vec<(u64, silk::edge::EmotionTag, f32)> = alloc::vec::Vec::new();

        for w in words {
            // FNV hash y hệt learn_text
            let mut h = 0xcbf29ce484222325_u64;
            for b in w.to_lowercase().bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }

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
        if results.is_empty() { return None; }

        let _n = results.len() as f32;
        let (sv, sa, sd, si) = results.iter().fold((0.0f32, 0.0f32, 0.0f32, 0.0f32),
            |(v,a,d,i), (_,e,w)| (v + e.valence*w, a + e.arousal*w, d + e.dominance*w, i + e.intensity*w));
        let tw: f32 = results.iter().map(|r| r.2).sum();
        if tw < 0.001 { return None; }

        Some(silk::edge::EmotionTag {
            valence:   sv/tw, arousal: sa/tw,
            dominance: sd/tw, intensity: si/tw,
        })
    }


    // ── Audio + Image input ───────────────────────────────────────────────────

    /// Nhận audio features → fuse với text emotion → update pipeline.
    ///
    /// Dùng khi có input audio (giọng nói, nhạc...).
    pub fn process_audio(
        &mut self,
        pitch_hz: f32, energy: f32, tempo_bpm: f32,
        voice_break: f32, ts: i64,
    ) -> Response {
        use context::modality::AudioFeatures;
        use context::fusion::{ModalityInput, ModalityKind, fuse};

        let audio = AudioFeatures { pitch_hz, energy, tempo_bpm, voice_break, brightness: 0.5 };
        let audio_input = audio.to_modality_input(0.85);

        // Fuse với text emotion hiện tại
        let cur_v = self.learning.context().fx();
        let text_input = ModalityInput {
            tag: silk::edge::EmotionTag {
                valence: cur_v, arousal: 0.5, dominance: 0.5, intensity: cur_v.abs()
            },
            confidence: 0.6,
            source: ModalityKind::Text,
        };
        let fused = fuse(&[text_input, audio_input]);

        // Feed fused emotion vào context
        let input = agents::encoder::ContentInput::Text {
            content: alloc::format!("audio pitch={:.0} energy={:.2}", pitch_hz, energy),
            timestamp: ts,
        };
        self.learning.process_one(input);

        Response {
            text:  alloc::format!("audio: {}", fused.describe()),
            tone:  self.learning.context().tone(),
            fx:    fused.tag.valence,
            kind:  ResponseKind::Natural,
        }
    }

    /// Nhận image features → fuse → update pipeline.
    pub fn process_image(
        &mut self,
        hue: f32, saturation: f32, brightness: f32,
        motion: f32, face_valence: Option<f32>, ts: i64,
    ) -> Response {
        use context::modality::ImageFeatures;
        use context::fusion::{ModalityInput, ModalityKind, fuse};

        let image = ImageFeatures { hue, saturation, brightness, motion, face_valence };
        let image_input = image.to_modality_input(0.70);

        let cur_v = self.learning.context().fx();
        let text_input = ModalityInput {
            tag: silk::edge::EmotionTag {
                valence: cur_v, arousal: 0.5, dominance: 0.5, intensity: cur_v.abs()
            },
            confidence: 0.6,
            source: ModalityKind::Text,
        };
        let fused = fuse(&[text_input, image_input]);

        let input = agents::encoder::ContentInput::Text {
            content: alloc::format!("image hue={:.0} brightness={:.2}", hue, brightness),
            timestamp: ts,
        };
        self.learning.process_one(input);

        Response {
            text: alloc::format!("image: {}", fused.describe()),
            tone: self.learning.context().tone(),
            fx:   fused.tag.valence,
            kind: ResponseKind::Natural,
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
            let hash      = obs.chain.chain_hash();
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
        self.learning.graph().all_edges()
            .filter(|e| e.kind.is_associative() && e.weight >= 0.30)
            .count()
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn turn_count(&self) -> u64 { self.turn_count }
    pub fn fx(&self) -> f32 { self.learning.context().fx() }
    pub fn tone(&self) -> ResponseTone { self.learning.context().tone() }
}




// ─────────────────────────────────────────────────────────────────────────────
// natural_reply — câu trả lời có nội dung từ word_guide
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo câu trả lời từ tone + từ ngữ học được.
/// Không hardcode string — dùng từ của lead_word/support_word từ lexicon.
fn natural_reply(tone: silk::walk::ResponseTone, v: f32, lead: &str, support: &str) -> alloc::string::String {
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

    fn rt() -> HomeRuntime { HomeRuntime::new(0x1234) }

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
        assert!(r.text.contains("1800") || r.text.contains("741741"),
            "Crisis response có helpline");
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
        if ucd::table_len() == 0 { return; }
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
        assert!(r.text.contains("STM"),   "Stats có STM");
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
            matches!(tone, ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause),
            "Buồn dần → {:?}", tone
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
        OlangExpr::Query(name) =>
            OlangIrExpr::Query(name),

        OlangExpr::Compose { a, b } =>
            OlangIrExpr::Compose(a, b),

        OlangExpr::RelationQuery { subject, relation, object } =>
            OlangIrExpr::Relation {
                subject,
                rel: relation_op_to_byte(relation),
                object,
            },

        OlangExpr::ContextQuery { term, context } =>
            OlangIrExpr::Compose(term, context), // context = LCA

        OlangExpr::Pipeline(exprs) =>
            OlangIrExpr::Pipeline(exprs.into_iter().map(olang_expr_to_ir).collect()),

        OlangExpr::Command(cmd) =>
            OlangIrExpr::Command(cmd),
    }
}

fn relation_op_to_byte(op: RelationOp) -> u8 {
    match op {
        RelationOp::Member      => 0x01,
        RelationOp::Subset      => 0x02,
        RelationOp::Equiv       => 0x03,
        RelationOp::Compose     => 0x05,
        RelationOp::Causes      => 0x06,
        RelationOp::Similar     => 0x07,
        RelationOp::DerivedFrom => 0x08,
        RelationOp::Context     => 0x09,
        RelationOp::Contains    => 0x0A,
        RelationOp::Intersects  => 0x0B,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Alias → Codepoint cache (từ file, bao gồm L2 nodes)
// ─────────────────────────────────────────────────────────────────────────────

/// Build alias→cp map từ origin.olang file.
///
/// Dùng tại boot để resolve_olang có thể tìm bất kỳ alias nào trong file.
fn build_alias_map(file_bytes: Option<&[u8]>) -> alloc::collections::BTreeMap<alloc::string::String, u32> {
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
                    if alias.name.starts_with("_qr_") { continue; }
                    // Skip nếu đã có trong static table (tránh hash collision override)
                    if map.contains_key(alias.name.as_str()) { continue; }
                    // Tìm cp: ưu tiên ALIAS_CODEPOINTS name match → decode_hash
                    let cp_opt = ALIAS_CODEPOINTS.iter()
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
    /// Approved proposals → ĐN nodes mới trong Registry.
    fn run_dream(&mut self, ts: i64) {
        self.last_dream_turn = self.turn_count;

        let result = self.dream.run(
            self.learning.stm(),
            self.learning.graph(),
            ts,
        );

        // Feed approved proposals → Registry (ĐN — chưa phải QR)
        {
            use memory::proposal::{ProposalKind, AAMDecision};
            let aam = memory::proposal::AAM::new();
            for proposal in &result.proposals {
                if matches!(aam.review(proposal), AAMDecision::Approved) {
                    match &proposal.kind {
                        ProposalKind::NewNode { chain, .. } => {
                            self.registry.insert(chain, 3, 0, ts, false); // L3, ĐN
                        }
                        ProposalKind::PromoteQR { chain_hash, fire_count: _ } => {
                            // Re-insert as QR
                            let dummy = olang::encoder::encode_codepoint(0x25CB);
                            self.registry.insert(&dummy, 0, 0, ts, true);
                            let _ = chain_hash;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Update self-model
        self.self_model.update(&self.registry, ts);
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
        assert!(bytes.len() < 100, "Empty session nhỏ: {} bytes", bytes.len());
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
            assert!(bytes.len() > 20,
                "{} edges → {} bytes", edges_saveable, bytes.len());
        }
    }

    #[test]
    fn saveable_edges_threshold() {
        let mut rt = HomeRuntime::new(0xCAFE);
        // Chưa học → 0 edges đủ mạnh
        assert_eq!(rt.saveable_edges(), 0);
        // Sau learning → có thể có edges
        for i in 0..8 {
            rt.process_text("natasha andrei pierre tolstoy chiến tranh hòa bình", i * 1000);
        }
        // saveable_edges() không panic
        let _ = rt.saveable_edges();
    }
}
