# agents

> ContentEncoder, LearningLoop, BookReader, and SecurityGate -- the L0 instincts that process every input through the full emotion pipeline.

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
| worker.rs | Worker agent utilities |
| chief.rs | Chief agent orchestration |
| leo.rs | LeoAI agent |

## Key API
```rust
pub fn ContentEncoder::encode(&self, input: ContentInput) -> EncodedContent
pub fn LearningLoop::process_one(&mut self, input: ContentInput) -> ProcessResult
pub fn SecurityGate::check_text(&self, text: &str) -> GateVerdict
pub fn BookReader::read(&self, text: &str) -> Vec<SentenceRecord>
pub fn EpistemicFirewall::wrap(level: EpistemicLevel, content: &str) -> String
```

## Rules
- 14: L0 does not import L1 -- absolute.
- 15: Only 2 Agents (AAM + LeoAI) -- do not add more.
- SecurityGate runs BEFORE everything else.
- Crisis -> return immediately, do not enter pipeline.
- Append-only (Rule 10): no DELETE, no OVERWRITE.
- BlackCurtain (Rule 18): not enough evidence -> silence.

## Test
```bash
cargo test -p agents
```
