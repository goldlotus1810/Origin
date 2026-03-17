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
// ComposedSkill — pipe N skills together
// ─────────────────────────────────────────────────────────────────────────────

/// ComposedSkill — orchestrate N skills in sequence.
///
/// Respects QT4③: individual skills don't know each other.
/// The composition happens at Agent level — skills only see ExecContext.
///
/// Pipeline: skill[0].output_chains → skill[1].input_chains → ... → final result
pub struct ComposedSkill {
    /// Name of this composed skill.
    skill_name: String,
    /// Ordered list of skill names to execute.
    /// Each name maps to a concrete Skill via the executor callback.
    pub steps: Vec<String>,
}

/// Result of a composed skill execution.
#[derive(Debug, Clone)]
pub struct ComposedResult {
    /// Final SkillResult from last step.
    pub result: SkillResult,
    /// Number of steps executed successfully.
    pub steps_completed: usize,
    /// Total steps in pipeline.
    pub total_steps: usize,
    /// Step that failed (if any).
    pub failed_at: Option<usize>,
}

impl ComposedSkill {
    /// Create a new composed skill from ordered step names.
    pub fn new(name: String, steps: Vec<String>) -> Self {
        Self {
            skill_name: name,
            steps,
        }
    }

    /// Execute the pipeline using a skill resolver callback.
    ///
    /// `resolve` maps skill name → &dyn Skill.
    /// Each step's output_chains become the next step's input_chains.
    /// State accumulates across all steps (QT4④: all I/O via ExecContext).
    pub fn execute_with<F>(&self, ctx: &mut ExecContext, resolve: F) -> ComposedResult
    where
        F: for<'a> Fn(&'a str) -> Option<&'static dyn Skill>,
    {
        let mut last_result = SkillResult::Insufficient;
        let mut steps_done = 0;

        for (i, step_name) in self.steps.iter().enumerate() {
            let skill = match resolve(step_name.as_str()) {
                Some(s) => s,
                None => {
                    return ComposedResult {
                        result: SkillResult::Error(alloc::format!("unknown skill: {}", step_name)),
                        steps_completed: steps_done,
                        total_steps: self.steps.len(),
                        failed_at: Some(i),
                    };
                }
            };

            // Pipe: previous output_chains → current input_chains
            if i > 0 {
                let prev_output = core::mem::take(&mut ctx.output_chains);
                ctx.input_chains = prev_output;
            }

            last_result = skill.execute(ctx);

            match &last_result {
                SkillResult::Ok { .. } => {
                    steps_done += 1;
                }
                SkillResult::Insufficient => {
                    // BlackCurtain: not enough data → stop pipeline
                    return ComposedResult {
                        result: last_result,
                        steps_completed: steps_done,
                        total_steps: self.steps.len(),
                        failed_at: Some(i),
                    };
                }
                SkillResult::Error(_) => {
                    return ComposedResult {
                        result: last_result,
                        steps_completed: steps_done,
                        total_steps: self.steps.len(),
                        failed_at: Some(i),
                    };
                }
            }
        }

        ComposedResult {
            result: last_result,
            steps_completed: steps_done,
            total_steps: self.steps.len(),
            failed_at: None,
        }
    }

    /// Skill name.
    pub fn name(&self) -> &str {
        &self.skill_name
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SkillPattern — learned skill sequence
// ─────────────────────────────────────────────────────────────────────────────

/// A learned pattern of skill execution.
///
/// When DreamCycle detects a recurring successful skill sequence,
/// it creates a SkillPattern proposal. If AAM approves, the pattern
/// becomes a reusable ComposedSkill.
#[derive(Debug, Clone)]
pub struct SkillPattern {
    /// Ordered skill names in the pattern.
    pub steps: Vec<String>,
    /// How many times this sequence has been observed.
    pub observations: u32,
    /// Success rate ∈ [0, 1].
    pub effectiveness: f32,
    /// Timestamp of first observation.
    pub first_seen: i64,
    /// Timestamp of last observation.
    pub last_seen: i64,
}

impl SkillPattern {
    /// Create from observed sequence.
    pub fn new(steps: Vec<String>, ts: i64) -> Self {
        Self {
            steps,
            observations: 1,
            effectiveness: 1.0,
            first_seen: ts,
            last_seen: ts,
        }
    }

    /// Record another successful observation.
    pub fn observe_success(&mut self, ts: i64) {
        self.observations += 1;
        self.effectiveness = (self.effectiveness * (self.observations - 1) as f32 + 1.0)
            / self.observations as f32;
        self.last_seen = ts;
    }

    /// Record a failed observation.
    pub fn observe_failure(&mut self, ts: i64) {
        self.observations += 1;
        self.effectiveness = self.effectiveness * (self.observations - 1) as f32
            / self.observations as f32;
        self.last_seen = ts;
    }

    /// Convert to ComposedSkill if effective enough.
    pub fn to_composed(&self) -> Option<ComposedSkill> {
        if self.effectiveness >= 0.6 && self.observations >= 3 {
            Some(ComposedSkill::new(
                alloc::format!("pattern:{}", self.steps.join("→")),
                self.steps.clone(),
            ))
        } else {
            None
        }
    }

    /// Key for deduplication — canonical step sequence.
    pub fn key(&self) -> String {
        self.steps.join("|")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SkillPatternStore — accumulate and promote patterns
// ─────────────────────────────────────────────────────────────────────────────

/// Store for observed skill patterns.
///
/// Sits in Agent (LeoAI) — Skills don't know it exists.
/// Tracks sequences, promotes effective ones to ComposedSkill.
#[derive(Default)]
pub struct SkillPatternStore {
    /// Known patterns indexed by key.
    patterns: Vec<SkillPattern>,
    /// Promoted patterns → ready-to-use ComposedSkills.
    composed: Vec<ComposedSkill>,
}

impl SkillPatternStore {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            composed: Vec::new(),
        }
    }

    /// Record an observed skill sequence.
    pub fn record(&mut self, steps: Vec<String>, success: bool, ts: i64) {
        let key = steps.join("|");
        if let Some(p) = self.patterns.iter_mut().find(|p| p.key() == key) {
            if success {
                p.observe_success(ts);
            } else {
                p.observe_failure(ts);
            }
            // Auto-promote: if pattern is effective and observed enough
            if p.effectiveness >= 0.6 && p.observations >= 3 {
                let already = self.composed.iter().any(|c| c.steps == p.steps);
                if !already {
                    if let Some(cs) = p.to_composed() {
                        self.composed.push(cs);
                    }
                }
            }
        } else {
            let mut pat = SkillPattern::new(steps, ts);
            if !success {
                pat.effectiveness = 0.0;
            }
            self.patterns.push(pat);
        }
    }

    /// Get all promoted ComposedSkills.
    pub fn composed_skills(&self) -> &[ComposedSkill] {
        &self.composed
    }

    /// Number of observed patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Number of promoted composed skills.
    pub fn composed_count(&self) -> usize {
        self.composed.len()
    }

    /// Get patterns meeting promotion threshold but not yet promoted.
    pub fn promotable(&self) -> Vec<&SkillPattern> {
        self.patterns
            .iter()
            .filter(|p| {
                p.effectiveness >= 0.6
                    && p.observations >= 3
                    && !self.composed.iter().any(|c| c.steps == p.steps)
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

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

    // ── ComposedSkill tests ──────────────────────────────────────────────────

    /// UpperSkill — transforms output by adding a second chain (simulating transform).
    struct DoubleSkill;

    impl Skill for DoubleSkill {
        fn name(&self) -> &str {
            "Double"
        }

        fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
            if ctx.input_chains.is_empty() {
                return SkillResult::Insufficient;
            }
            let chain = ctx.input_chains[0].clone();
            // Push input chain twice to output (simulating transformation)
            ctx.push_output(chain.clone());
            ctx.push_output(chain.clone());
            SkillResult::Ok {
                chain,
                emotion: ctx.current_emotion,
                note: String::from("doubled"),
            }
        }
    }

    static ECHO: EchoSkill = EchoSkill;
    static DOUBLE: DoubleSkill = DoubleSkill;

    fn resolve_skill(name: &str) -> Option<&'static dyn Skill> {
        match name {
            "Echo" => Some(&ECHO),
            "Double" => Some(&DOUBLE),
            _ => None,
        }
    }

    #[test]
    fn composed_skill_pipeline_two_steps() {
        let composed = ComposedSkill::new(String::from("echo→double"), vec![
            String::from("Echo"),
            String::from("Double"),
        ]);

        let mut ctx = ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0);
        let chain = olang::encoder::encode_codepoint(0x25CF); // ●
        ctx.push_input(chain);

        let result = composed.execute_with(&mut ctx, resolve_skill);

        assert!(result.result.is_ok());
        assert_eq!(result.steps_completed, 2);
        assert_eq!(result.total_steps, 2);
        assert!(result.failed_at.is_none());
        // Double outputs 2 chains
        assert_eq!(ctx.output_chains.len(), 2);
    }

    #[test]
    fn composed_skill_stops_on_error() {
        let composed = ComposedSkill::new(String::from("test"), vec![
            String::from("Echo"),
            String::from("Missing"),
            String::from("Echo"),
        ]);

        let mut ctx = ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0);
        let chain = olang::encoder::encode_codepoint(0x25CF);
        ctx.push_input(chain);

        let result = composed.execute_with(&mut ctx, resolve_skill);

        assert!(!result.result.is_ok());
        assert_eq!(result.steps_completed, 1); // Echo succeeded
        assert_eq!(result.failed_at, Some(1)); // Missing failed
    }

    #[test]
    fn composed_skill_stops_on_insufficient() {
        // No input → first step returns Insufficient
        let composed = ComposedSkill::new(String::from("test"), vec![
            String::from("Echo"),
        ]);

        let mut ctx = ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0);
        // No input_chains

        let result = composed.execute_with(&mut ctx, resolve_skill);

        assert!(matches!(result.result, SkillResult::Insufficient));
        assert_eq!(result.steps_completed, 0);
        assert_eq!(result.failed_at, Some(0));
    }

    #[test]
    fn composed_skill_name() {
        let cs = ComposedSkill::new(String::from("my_pipeline"), vec![]);
        assert_eq!(cs.name(), "my_pipeline");
    }

    // ── SkillPattern tests ───────────────────────────────────────────────────

    #[test]
    fn skill_pattern_observe_success() {
        let mut pat = SkillPattern::new(vec![String::from("A"), String::from("B")], 100);
        assert_eq!(pat.observations, 1);
        assert_eq!(pat.effectiveness, 1.0);

        pat.observe_success(200);
        assert_eq!(pat.observations, 2);
        assert_eq!(pat.effectiveness, 1.0); // 2/2

        pat.observe_failure(300);
        assert_eq!(pat.observations, 3);
        // (1.0 * 2) / 3 ≈ 0.667
        assert!(pat.effectiveness > 0.6 && pat.effectiveness < 0.7);
    }

    #[test]
    fn skill_pattern_to_composed_threshold() {
        let mut pat = SkillPattern::new(vec![String::from("X"), String::from("Y")], 100);
        // 1 observation → not enough
        assert!(pat.to_composed().is_none());

        pat.observe_success(200); // 2 observations
        assert!(pat.to_composed().is_none()); // still < 3

        pat.observe_success(300); // 3 observations, effectiveness=1.0
        let cs = pat.to_composed();
        assert!(cs.is_some());
        let cs = cs.unwrap();
        assert!(cs.name().contains("X→Y"));
        assert_eq!(cs.steps.len(), 2);
    }

    #[test]
    fn skill_pattern_low_effectiveness_no_promote() {
        let mut pat = SkillPattern::new(vec![String::from("A")], 100);
        pat.observe_failure(200);
        pat.observe_failure(300);
        pat.observe_failure(400);
        // 4 observations but low effectiveness
        assert!(pat.effectiveness < 0.6);
        assert!(pat.to_composed().is_none());
    }

    #[test]
    fn skill_pattern_key() {
        let pat = SkillPattern::new(vec![String::from("A"), String::from("B")], 100);
        assert_eq!(pat.key(), "A|B");
    }

    // ── SkillPatternStore tests ──────────────────────────────────────────────

    #[test]
    fn store_record_and_promote() {
        let mut store = SkillPatternStore::new();
        let steps = vec![String::from("Ingest"), String::from("Cluster")];

        // Record 3 successes → auto-promote
        store.record(steps.clone(), true, 100);
        assert_eq!(store.pattern_count(), 1);
        assert_eq!(store.composed_count(), 0); // not enough observations

        store.record(steps.clone(), true, 200);
        assert_eq!(store.composed_count(), 0); // 2 < 3

        store.record(steps.clone(), true, 300);
        assert_eq!(store.composed_count(), 1); // 3 observations, eff=1.0 → promoted!
        assert_eq!(store.composed_skills()[0].steps, steps);
    }

    #[test]
    fn store_no_duplicate_promotion() {
        let mut store = SkillPatternStore::new();
        let steps = vec![String::from("A"), String::from("B")];

        for i in 0..5 {
            store.record(steps.clone(), true, i * 100);
        }
        // Should only promote once
        assert_eq!(store.composed_count(), 1);
    }

    #[test]
    fn store_mixed_success_failure() {
        let mut store = SkillPatternStore::new();
        let steps = vec![String::from("X")];

        store.record(steps.clone(), true, 100);  // eff=1.0
        store.record(steps.clone(), false, 200); // eff=0.5
        store.record(steps.clone(), false, 300); // eff=0.33
        // 3 observations but effectiveness < 0.6 → no promote
        assert_eq!(store.composed_count(), 0);
        assert_eq!(store.promotable().len(), 0);
    }

    #[test]
    fn store_multiple_patterns() {
        let mut store = SkillPatternStore::new();
        let a = vec![String::from("A"), String::from("B")];
        let b = vec![String::from("C"), String::from("D")];

        for i in 0..3 {
            store.record(a.clone(), true, i * 100);
        }
        store.record(b.clone(), true, 400);

        assert_eq!(store.pattern_count(), 2);
        assert_eq!(store.composed_count(), 1); // only pattern A promoted
    }
}
