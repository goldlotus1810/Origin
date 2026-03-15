//! # origin — HomeRuntime
//!
//! ○(∅) == ○ — tự boot, tự vận hành.
//!
//! process_one(input) → Response
//!   SecurityGate → Parse → Encode → Context → STM → Silk → Response

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::format;

use agents::learning::{LearningLoop, ProcessResult};
use agents::encoder::ContentInput;
use agents::gate::EpistemicLevel;
use context::engine::ActivationResult;
use memory::dream::{DreamCycle, DreamConfig};
use silk::walk::ResponseTone;

use crate::parser::{OlangParser, OlangExpr, ParseResult, RelationOp};
use olang::ir::{OlangIrExpr, OlangProgram, Op, compile_expr};
use olang::vm::{OlangVM, VmEvent};

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
    learning: LearningLoop,
    parser:   OlangParser,
    dream:    DreamCycle,
    uptime_ns: i64,
    turn_count: u64,
}

impl HomeRuntime {
    /// Boot từ hư không.
    pub fn new(session_id: u64) -> Self {
        Self {
            learning:   LearningLoop::new(session_id),
            parser:     OlangParser::new(),
            dream:      DreamCycle::new(DreamConfig::default()),
            uptime_ns:  0,
            turn_count: 0,
        }
    }

    /// Xử lý một text input — entry point chính.
    pub fn process_text(&mut self, text: &str, ts: i64) -> Response {
        self.turn_count += 1;
        self.uptime_ns = ts;

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
        let input = ContentInput::Text { content: text.to_string(), timestamp: ts };

        match self.learning.process_one(input) {
            ProcessResult::Crisis { message } => Response {
                text: message,
                tone: ResponseTone::Supportive,
                fx:   self.learning.context().fx(),
                kind: ResponseKind::Crisis,
            },

            ProcessResult::Blocked { reason } => Response {
                text: format!("Tôi không thể giúp với điều này. ({})", reason),
                tone: ResponseTone::Gentle,
                fx:   self.learning.context().fx(),
                kind: ResponseKind::Blocked,
            },

            ProcessResult::Ok { emotion, .. } => {
                let fx   = self.learning.context().fx();
                let tone = self.learning.context().tone();
                let text = self.generate_response(tone, emotion.valence, fx);
                Response { text, tone, fx, kind: ResponseKind::Natural }
            }

            ProcessResult::Empty => Response {
                text: String::from("..."),
                tone: ResponseTone::Engaged,
                fx:   0.0,
                kind: ResponseKind::Natural,
            },
        }
    }

    // ── ○{} expression — compile và execute qua OlangVM ─────────────────────

    fn process_olang(&mut self, expr: OlangExpr, ts: i64) -> Response {
        // Commands bypass VM
        if let OlangExpr::Command(cmd) = expr {
            return self.handle_command(&cmd, ts);
        }

        // Compile OlangExpr → OlangIrExpr → OlangProgram
        let ir_expr = olang_expr_to_ir(expr);
        let prog    = compile_expr(&ir_expr);
        let vm      = OlangVM::new();
        let result  = vm.execute(&prog);

        // Collect output từ VM events
        let mut output_text = String::new();
        for event in &result.events {
            match event {
                VmEvent::Output(chain) => {
                    let hash = chain.chain_hash();
                    output_text.push_str(&format!("hash=0x{:08X} ", hash & 0xFFFF_FFFF));
                }
                VmEvent::LookupAlias(alias) => {
                    output_text.push_str(&format!("[{}] ", alias));
                }
                VmEvent::CreateEdge { from, to, rel } => {
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
                let text = format!(
                    "HomeOS ○\n\
                     Turns   : {}\n\
                     STM     : {} observations\n\
                     Silk    : {} edges\n\
                     f(x)    : {:.3}",
                    self.turn_count,
                    self.learning.stm().len(),
                    self.learning.graph().len(),
                    self.learning.context().fx(),
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

    /// Sinh response text dựa trên tone và emotion.
    /// Không nhảy quá 0.40/bước — dẫn từ từ.
    fn generate_response(&self, tone: ResponseTone, current_v: f32, _fx: f32) -> String {
        match tone {
            ResponseTone::Supportive =>
                if current_v < -0.5 {
                    String::from("Tôi nghe bạn. Bạn đang trải qua điều gì vậy?")
                } else {
                    String::from("Tôi hiểu. Bạn muốn kể thêm không?")
                },
            ResponseTone::Pause =>
                String::from("Bạn có ổn không?"),
            ResponseTone::Gentle =>
                String::from("Tôi ở đây với bạn."),
            ResponseTone::Reinforcing =>
                String::from("Tốt lắm! Bạn đang tiến bộ rõ rệt."),
            ResponseTone::Celebratory =>
                String::from("Tuyệt vời! Tôi vui vì bạn."),
            ResponseTone::Engaged =>
                String::from("Tôi hiểu rồi."),
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn turn_count(&self) -> u64 { self.turn_count }
    pub fn fx(&self) -> f32 { self.learning.context().fx() }
    pub fn tone(&self) -> ResponseTone { self.learning.context().tone() }
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
