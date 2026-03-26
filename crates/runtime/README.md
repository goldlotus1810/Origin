# runtime

> HomeOS runtime engine: processes natural text and `○{}` expressions through a 7-stage emotion pipeline, producing tone-aware responses.

## Dependencies
- ucd
- olang
- silk
- context
- agents
- memory
- libm

## Files
| File | Purpose |
|------|---------|
| lib.rs | Crate root; re-exports `origin`, `parser`, `response_template` modules (`#![no_std]`) |
| origin.rs | `HomeRuntime` struct — main entry point; text/audio/image processing, Dream cycle, Silk walk, serialization |
| parser.rs | `OlangParser` — splits input into `Natural` text vs `○{...}` expressions; tokenizer and expression AST |
| response_template.rs | `render()` — maps (tone, action, valence) to response text; crisis text, empathy, clarify, fallback templates |

## Key API
```rust
// Boot runtime from nothing or from origin.olang bytes
pub fn HomeRuntime::new(session_id: u64) -> Self;
pub fn HomeRuntime::with_file(session_id: u64, file_bytes: Option<&[u8]>) -> Self;

// Process text input (natural or ○{...}) — main entry point
pub fn HomeRuntime::process_text(&mut self, text: &str, ts: i64) -> Response;

// Process audio/image modality inputs
pub fn HomeRuntime::process_audio(&mut self, pitch_hz: f32, energy: f32, tempo_bpm: f32, voice_break: f32, ts: i64) -> Response;
pub fn HomeRuntime::process_image(&mut self, hue: f32, saturation: f32, brightness: f32, motion: f32, face_valence: Option<f32>, ts: i64) -> Response;

// Serialize learned state (Silk edges + STM observations) for append to origin.olang
pub fn HomeRuntime::serialize_learned(&self, ts: i64) -> Vec<u8>;

// Response template rendering
pub fn render(p: &ResponseParams) -> String;
```

## Rules
- `#![no_std]` — uses `alloc` only, no standard library
- Crisis detection overrides all other processing (QT9: do no harm)
- Auto-Dream triggers every 8 turns when STM has >= 3 observations
- Serialization is append-only (QT8) — never overwrites origin.olang
- Response text lives in `response_template`, not hardcoded in logic

## Test
```bash
cargo test -p runtime
```
