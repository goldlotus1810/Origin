# agents

> L0 instincts + Agent hierarchy: ContentEncoder, LearningLoop, SecurityGate, 7 Instinct Skills, LeoAI, Chief, Worker, and Skill trait.

## Dependencies
- ucd
- olang
- silk
- context
- isl
- libm

## Files
| File | Purpose |
|------|---------|
| encoder.rs | ContentEncoder: encode Text/Audio/Sensor/Code/Math/System -> EncodedContent (MolecularChain + EmotionTag) |
| learning.rs | LearningLoop: Gate -> Encode -> Context -> STM -> Silk pipeline; ShortTermMemory with LFU eviction; 5-layer text learning |
| gate.rs | SecurityGate (check_text, check_intent), EpistemicFirewall (Fact/Opinion/Hypothesis/Unknown/Deprecated), BlockReason |
| book.rs | BookReader: read(text) -> Vec<SentenceRecord>, top_significant, stats; sentence-level + word-level + topic emotion |
| skill.rs | Skill trait + ExecContext + SkillResult — QT4 rules: stateless, isolated, single-responsibility |
| instinct.rs | 7 superintelligent instincts: Analogy, Abstraction, Causality, Contradiction, Curiosity, Reflection, Honesty + innate_instincts() |
| worker.rs | Worker agent: HomeOS thu nhỏ tại thiết bị. WorkerKind(Sensor/Actuator/Camera/Network/Generic), SensorReading, WorkerReport |
| chief.rs | Chief agent: tủy sống — xử lý/tổng hợp từ Workers. ChiefKind(Home/Vision/Network/General), IngestedReport |
| leo.rs | LeoAI: não — KnowledgeChief + Learning + Dream. LeoState(Listening/Learning/Dreaming/Proposing), LeoPendingProposal |

## Agent Hierarchy
```
AAM [tier 0] — stateless (memory crate)
LeoAI · Chief [tier 1] — orchestrators
Worker [tier 2] — HomeOS at device

✅ AAM ↔ Chief · ✅ Chief ↔ Chief · ✅ Chief ↔ Worker
❌ AAM ↔ Worker · ❌ Worker ↔ Worker
```

## Key API
```rust
pub fn ContentEncoder::encode(&self, input: ContentInput) -> EncodedContent
pub fn LearningLoop::process_one(&mut self, input: ContentInput) -> ProcessResult
pub fn SecurityGate::check_text(&self, text: &str) -> GateVerdict
pub trait Skill { fn execute(&self, ctx: &mut ExecContext) -> SkillResult; }
pub fn innate_instincts() -> [&'static dyn Skill; 7]
pub fn LeoAI::ingest(&mut self, report: IngestedReport, ts: i64)
pub fn LeoAI::run_instincts(&mut self, ctx: &mut ExecContext)
pub fn Chief::receive_frame(&mut self, frame: ISLFrame, ts: i64)
pub fn Worker::process(&mut self, event: WorkerEvent, ts: i64)
```

## Rules
- 14: L0 does not import L1 -- absolute.
- 15: Agent tiers: AAM(tier 0) + Chiefs(tier 1) + Workers(tier 2).
- QT4①-⑤: Skill = stateless, isolated, single-responsibility, communicates via ExecContext.
- SecurityGate runs BEFORE everything else.
- Crisis -> return immediately, do not enter pipeline.
- Worker sends molecular chain -- NEVER raw data.
- All agents: silent by default, wake on ISL, process, sleep.

## Test
```bash
cargo test -p agents
```
