# context

> Emotion pipeline: word affect lexicon, intent estimation, ConversationCurve (f(x) = 0.6*f_conv + 0.4*f_dn), and cross-modal fusion.

## Dependencies
- ucd
- olang
- silk
- libm

## Files
| File | Purpose |
|------|---------|
| emotion.rs | IntentKind, IntentModifier, word_affect() lexicon (700+ entries, multilingual), sentence_affect, blend_with_audio, bootstrap_affect |
| curve.rs | ConversationCurve: push(valence), derivatives d1/d2, f_conv formula, update_dn, tone() |
| intent.rs | estimate_intent() with scoring buckets, IntentEstimate, IntentAction, decide_action, crisis_text_for |
| fusion.rs | ModalityFusion: fuse() multiple ModalityInputs, conflict detection, BlackCurtain threshold |
| infer.rs | Context inference (tense detection, context scaling) |
| engine.rs | ContextEngine orchestrator |
| context.rs | Context data structures |
| snapshot.rs | RawInput snapshot for pipeline |
| phrase.rs | Phrase-level analysis |
| modality.rs | Modality types |
| word_guide.rs | Word guidance utilities |

## Key API
```rust
pub fn word_affect(word: &str) -> EmotionTag
pub fn estimate_intent(text: &str, cur_v: f32, cur_a: f32) -> IntentEstimate
pub fn ConversationCurve::push(&mut self, valence: f32) -> f32
pub fn fuse(inputs: &[ModalityInput]) -> FusedEmotionTag
pub fn decide_action(est: &IntentEstimate, cur_v: f32) -> IntentAction
```

## Rules
- 18: Not enough evidence -> silence -- NEVER fabricate (BlackCurtain).
- 17: Fibonacci throughout.
- Modality weights: Bio=0.50 > Audio=0.40 > Text=0.30 > Image=0.25.
- Conflict (text happy + voice trembling) -> Audio wins valence, confidence drops.
- Crisis intent has absolute priority -- stops pipeline immediately.

## Test
```bash
cargo test -p context
```
