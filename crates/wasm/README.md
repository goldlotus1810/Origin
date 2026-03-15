# homeos-wasm

> WebAssembly bindings exposing the HomeOS runtime to JavaScript/browser via `wasm-bindgen`.

## Dependencies
- ucd
- olang
- silk
- context
- agents
- memory
- runtime
- wasm-bindgen

## Files
| File | Purpose |
|------|---------|
| lib.rs | `HomeOSWasm` struct with `#[wasm_bindgen]` methods; JSON response serialization; helper free functions (`create_homeos`, `quick_encode`, `version`) |

## Key API
```rust
// Constructor — creates HomeOS instance in browser
#[wasm_bindgen(constructor)]
pub fn HomeOSWasm::new() -> HomeOSWasm;

// Constructor with origin.olang bytes
pub fn HomeOSWasm::new_with_file(bytes: &[u8]) -> HomeOSWasm;

// Process text or ○{...} input, returns JSON { text, tone, fx, kind, turn }
pub fn HomeOSWasm::process(&mut self, input: &str) -> String;

// Getters
pub fn HomeOSWasm::fx(&self) -> f32;       // ConversationCurve value
pub fn HomeOSWasm::turns(&self) -> u64;    // turn count
pub fn HomeOSWasm::tone(&self) -> String;  // current tone as string

// Static utilities
pub fn HomeOSWasm::ucd_len() -> u32;
pub fn HomeOSWasm::encode_cp(cp: u32) -> u32;

// Free functions
pub fn create_homeos() -> HomeOSWasm;
pub fn quick_encode(cp: u32) -> u32;
pub fn version() -> String;
```

## Rules
- Crate type is `cdylib` + `rlib` for WASM compilation
- JSON is hand-serialized (no serde) to keep WASM bundle small
- Response JSON format: `{"text":"...","tone":"...","fx":0.0,"kind":"...","turn":N}`
- `console_error_panic_hook` feature enables browser console panic messages
- `fx` and `turns` are exposed as `#[wasm_bindgen(getter)]` properties

## Test
```bash
cargo test -p homeos-wasm
```
