# silk

> Hebbian learning graph with emotional edges: co-activation strengthens connections, decay weakens them, walk amplifies composite emotion.

## Dependencies
- ucd
- olang
- libm

## Files
| File | Purpose |
|------|---------|
| edge.rs | EmotionTag (V/A/D/I), EdgeKind (22 types), SilkEdge struct, ModalitySource |
| graph.rs | SilkGraph: connect_structural, co_activate, decay_all, neighbors, cluster_score_partial, promote_candidates |
| hebbian.rs | Hebbian strengthen/decay formulas, Fibonacci threshold, should_promote |
| walk.rs | sentence_affect (walk with amplification), ResponseTone, response_tone, next_curve_step |

## Key API
```rust
pub fn SilkGraph::co_activate(&mut self, from: u64, to: u64, emotion: EmotionTag, reward: f32, ts: i64)
pub fn sentence_affect(graph: &SilkGraph, word_hashes: &[u64], word_emotions: &[EmotionTag], max_depth: usize) -> WalkResult
pub fn hebbian_strengthen(weight: f32, reward: f32) -> f32
pub fn hebbian_decay(weight: f32, elapsed_ns: i64) -> f32
pub fn should_promote(weight: f32, fire_count: u32, depth: usize) -> bool
```

## Rules
- 11: Silk only at Ln-1 -- free between leaves of same layer.
- 12: Cross-layer connection -> via NodeLx representative.
- 13: Silk edges carry EmotionTag of the co-activation moment.
- 17: Fibonacci throughout -- structure, threshold, render.
- NEVER average emotions -- always AMPLIFY via Silk walk.

## Test
```bash
cargo test -p silk
```
