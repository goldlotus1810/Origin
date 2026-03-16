//! # skill — Skill trait + ExecContext
//!
//! 5 Quy tắc Skill (QT4 · bất biến):
//!   ① 1 Skill = 1 trách nhiệm
//!   ② Skill không biết Agent là gì
//!   ③ Skill không biết Skill khác tồn tại
//!   ④ Skill giao tiếp qua ExecContext.State
//!   ⑤ Skill không giữ state — state nằm trong Agent
//!
//! Skill = hàm thuần (input → output), tất cả context qua ExecContext.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// SkillResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả trả về từ Skill.execute().
#[derive(Debug, Clone)]
pub enum SkillResult {
    /// Thành công — có chain output.
    Ok {
        chain: MolecularChain,
        emotion: EmotionTag,
        note: String,
    },
    /// Không đủ data — BlackCurtain (QT18: im lặng khi không biết).
    Insufficient,
    /// Lỗi.
    Error(String),
}

impl SkillResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, SkillResult::Ok { .. })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ExecContext — shared state giữa Agent ↔ Skill
// ─────────────────────────────────────────────────────────────────────────────

/// ExecContext mang state từ Agent vào Skill.
///
/// Skill KHÔNG biết Agent là gì (QT4②).
/// Skill KHÔNG giữ state (QT4⑤) — state nằm ở đây.
#[derive(Debug)]
pub struct ExecContext {
    /// Timestamp hiện tại.
    pub timestamp: i64,
    /// Emotion hiện tại của conversation.
    pub current_emotion: EmotionTag,
    /// f(x) — ConversationCurve value.
    pub fx: f32,
    /// Input chains (từ Agent feed vào).
    pub input_chains: Vec<MolecularChain>,
    /// Output chains (Skill ghi vào, Agent đọc ra).
    pub output_chains: Vec<MolecularChain>,
    /// Key-value state (Skill đọc/ghi, Agent quản lý).
    pub state: Vec<(String, String)>,
}

impl ExecContext {
    /// Tạo context mới.
    pub fn new(ts: i64, emotion: EmotionTag, fx: f32) -> Self {
        Self {
            timestamp: ts,
            current_emotion: emotion,
            fx,
            input_chains: Vec::new(),
            output_chains: Vec::new(),
            state: Vec::new(),
        }
    }

    /// Đọc state value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.state
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Ghi state value.
    pub fn set(&mut self, key: String, value: String) {
        if let Some(entry) = self.state.iter_mut().find(|(k, _)| *k == key) {
            entry.1 = value;
        } else {
            self.state.push((key, value));
        }
    }

    /// Thêm input chain.
    pub fn push_input(&mut self, chain: MolecularChain) {
        self.input_chains.push(chain);
    }

    /// Thêm output chain (Skill ghi kết quả).
    pub fn push_output(&mut self, chain: MolecularChain) {
        self.output_chains.push(chain);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Skill trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait cho mọi Skill trong HomeOS.
///
/// Quy tắc:
///   - `execute()` là hàm duy nhất — 1 Skill = 1 trách nhiệm (QT4①)
///   - Skill không biết Agent gọi nó (QT4②)
///   - Skill không biết Skill khác (QT4③)
///   - Tất cả I/O qua `ExecContext` (QT4④)
///   - Không `&mut self` — Skill stateless (QT4⑤)
pub trait Skill {
    /// Tên Skill — dùng cho logging/debug.
    fn name(&self) -> &str;

    /// Thực thi Skill.
    ///
    /// - Đọc input từ `ctx.input_chains` và `ctx.state`
    /// - Ghi output vào `ctx.output_chains` và `ctx.state`
    /// - Trả về `SkillResult`
    fn execute(&self, ctx: &mut ExecContext) -> SkillResult;
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Skill — đơn giản echo input.
    struct EchoSkill;

    impl Skill for EchoSkill {
        fn name(&self) -> &str {
            "Echo"
        }

        fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
            if ctx.input_chains.is_empty() {
                return SkillResult::Insufficient;
            }
            let chain = ctx.input_chains[0].clone();
            ctx.push_output(chain.clone());
            SkillResult::Ok {
                chain,
                emotion: ctx.current_emotion,
                note: String::from("echo"),
            }
        }
    }

    /// Test Skill — đọc/ghi state.
    struct CountSkill;

    impl Skill for CountSkill {
        fn name(&self) -> &str {
            "Count"
        }

        fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
            let count = ctx
                .get("count")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            ctx.set(String::from("count"), alloc::format!("{}", count + 1));
            SkillResult::Insufficient // chỉ đếm, không output chain
        }
    }

    #[test]
    fn skill_is_stateless() {
        let skill = EchoSkill;
        // Gọi 2 lần — Skill không nhớ gì giữa 2 lần
        let mut ctx1 = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);
        let mut ctx2 = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);
        let r1 = skill.execute(&mut ctx1);
        let r2 = skill.execute(&mut ctx2);
        // Cả 2 đều Insufficient vì không có input
        assert!(matches!(r1, SkillResult::Insufficient));
        assert!(matches!(r2, SkillResult::Insufficient));
    }

    #[test]
    fn skill_reads_input_from_context() {
        let skill = EchoSkill;
        let mut ctx = ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0);
        // Feed input chain
        let chain = olang::encoder::encode_codepoint(0x25CF); // ●
        ctx.push_input(chain);

        let result = skill.execute(&mut ctx);
        assert!(result.is_ok(), "Có input → Ok");
        assert_eq!(ctx.output_chains.len(), 1, "Skill ghi output vào context");
    }

    #[test]
    fn skill_communicates_via_state() {
        let skill = CountSkill;
        let mut ctx = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);

        // Lần 1: count = 0 → set count = 1
        skill.execute(&mut ctx);
        assert_eq!(ctx.get("count"), Some("1"));

        // Lần 2: count = 1 → set count = 2
        skill.execute(&mut ctx);
        assert_eq!(ctx.get("count"), Some("2"));

        // State nằm trong ExecContext, KHÔNG trong Skill
    }

    #[test]
    fn skill_does_not_know_agent() {
        // Skill trait không có &Agent, &Chief, &Worker — chỉ &ExecContext
        // Nếu code này compile → QT4② satisfied
        let skill: &dyn Skill = &EchoSkill;
        let mut ctx = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);
        let _ = skill.execute(&mut ctx);
        assert_eq!(skill.name(), "Echo");
    }

    #[test]
    fn exec_context_state_crud() {
        let mut ctx = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);
        assert!(ctx.get("key").is_none());

        ctx.set(String::from("key"), String::from("value1"));
        assert_eq!(ctx.get("key"), Some("value1"));

        // Overwrite
        ctx.set(String::from("key"), String::from("value2"));
        assert_eq!(ctx.get("key"), Some("value2"));
    }

    #[test]
    fn insufficient_when_no_evidence() {
        // BlackCurtain (QT18): không đủ evidence → im lặng
        let skill = EchoSkill;
        let mut ctx = ExecContext::new(0, EmotionTag::NEUTRAL, 0.0);
        let r = skill.execute(&mut ctx);
        assert!(matches!(r, SkillResult::Insufficient));
    }
}
