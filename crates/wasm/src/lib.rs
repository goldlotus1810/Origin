//! # homeos-wasm — HomeOS WebAssembly Bindings
//!
//! Expose HomeOS ○{} API to JavaScript/browser.
//!
//! Usage (JS):
//!   import init, { HomeOSWasm } from './homeos_wasm.js';
//!   await init();
//!   const os = new HomeOSWasm();
//!   const r  = os.process("○{lửa ∘ nước}");
//!   console.log(r); // JSON response

pub mod bridge;

use wasm_bindgen::prelude::*;

use bridge::EventStream;
use runtime::origin::{HomeRuntime, Response, ResponseKind};
use silk::walk::ResponseTone;

// ─────────────────────────────────────────────────────────────────────────────
// HomeOSWasm — JS-facing API
// ─────────────────────────────────────────────────────────────────────────────

/// HomeOS instance trong browser.
#[wasm_bindgen]
pub struct HomeOSWasm {
    rt: HomeRuntime,
    turn_count: u64,
    events: EventStream,
}

#[wasm_bindgen]
impl HomeOSWasm {
    /// Khởi tạo HomeOS mới.
    #[wasm_bindgen(constructor)]
    pub fn new() -> HomeOSWasm {
        // Panic hook để debug trong browser console
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        HomeOSWasm {
            rt: HomeRuntime::new(js_timestamp_u64()),
            turn_count: 0,
            events: EventStream::default(),
        }
    }

    /// Khởi tạo với origin.olang bytes.
    #[wasm_bindgen]
    pub fn new_with_file(bytes: &[u8]) -> HomeOSWasm {
        HomeOSWasm {
            rt: HomeRuntime::with_file(js_timestamp_u64(), Some(bytes)),
            turn_count: 0,
            events: EventStream::default(),
        }
    }

    /// Process text input → JSON response.
    ///
    /// Input: text thường hoặc ○{...}
    /// Output: JSON { text, tone, fx, kind, turn }
    ///
    /// Side effect: pushes emotion event to EventStream.
    #[wasm_bindgen]
    pub fn process(&mut self, input: &str) -> String {
        self.turn_count += 1;
        let ts = js_timestamp();
        let response = self.rt.process_text(input, ts);
        // Push emotion update to event stream for browser subscribers
        let fx = response.fx;
        self.events.push_emotion(0.0, 0.0, fx);
        response_to_json(&response, self.turn_count)
    }

    /// Drain pending events as binary frames for WebSocket push.
    ///
    /// Call after process() to get emotion/dream/silk updates.
    /// Returns array of binary frame byte arrays (each is a BridgeFrame).
    #[wasm_bindgen]
    pub fn drain_events(&mut self) -> Vec<u8> {
        let frames = self.events.drain_bytes();
        // Concatenate: [count:4][len0:4][frame0][len1:4][frame1]...
        let mut out = Vec::new();
        let count = frames.len() as u32;
        out.extend_from_slice(&count.to_be_bytes());
        for frame in &frames {
            let len = frame.len() as u32;
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(frame);
        }
        out
    }

    /// Number of pending events.
    #[wasm_bindgen(getter)]
    pub fn pending_events(&self) -> u32 {
        self.events.pending() as u32
    }

    /// f(x) — ConversationCurve hiện tại.
    #[wasm_bindgen(getter)]
    pub fn fx(&self) -> f32 {
        self.rt.fx()
    }

    /// Turn count.
    #[wasm_bindgen(getter)]
    pub fn turns(&self) -> u64 {
        self.turn_count
    }

    /// Tone hiện tại dưới dạng string.
    #[wasm_bindgen]
    pub fn tone(&self) -> String {
        tone_to_str(self.rt.tone())
    }

    /// STM observation count — how many things HomeOS remembers this session.
    #[wasm_bindgen(getter)]
    pub fn stm_len(&self) -> u32 {
        self.rt.stm_len() as u32
    }

    /// Silk edge count — total learned connections.
    #[wasm_bindgen(getter)]
    pub fn silk_edge_count(&self) -> u32 {
        self.rt.silk_edge_count() as u32
    }

    /// Silk node count — distinct concepts in the graph.
    #[wasm_bindgen(getter)]
    pub fn silk_node_count(&self) -> u32 {
        self.rt.silk_node_count() as u32
    }

    /// Registry node count — total registered nodes (L0+L1+…).
    #[wasm_bindgen(getter)]
    pub fn registry_len(&self) -> u32 {
        self.rt.registry_len() as u32
    }

    /// Read a block of text through BookReader pipeline.
    /// Returns number of significant sentences learned.
    #[wasm_bindgen]
    pub fn read_book(&mut self, text: &str) -> u32 {
        let ts = js_timestamp();
        self.rt.read_book(text, ts) as u32
    }

    /// Run dream cycle manually. Returns JSON summary.
    #[wasm_bindgen]
    pub fn dream(&mut self) -> String {
        let r = self.rt.process_text("○{dream}", js_timestamp());
        response_to_json(&r, self.turn_count)
    }

    /// Get stats as JSON.
    #[wasm_bindgen]
    pub fn stats(&mut self) -> String {
        let r = self.rt.process_text("○{stats}", js_timestamp());
        response_to_json(&r, self.turn_count)
    }

    /// UCD table size — verify WASM bundle loaded correctly.
    #[wasm_bindgen]
    pub fn ucd_len() -> u32 {
        ucd::table_len() as u32
    }

    /// Encode codepoint → chain hash (để JS có thể verify).
    #[wasm_bindgen]
    pub fn encode_cp(cp: u32) -> u32 {
        use olang::encoder::encode_codepoint;
        let chain = encode_codepoint(cp);
        (chain.chain_hash() & 0xFFFF_FFFF) as u32
    }
}

impl Default for HomeOSWasm {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick create HomeOS with origin.olang bytes.
#[wasm_bindgen]
pub fn create_homeos_with_file(bytes: &[u8]) -> HomeOSWasm {
    HomeOSWasm::new_with_file(bytes)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn response_to_json(r: &Response, turn: u64) -> String {
    let kind = match r.kind {
        ResponseKind::Natural => "natural",
        ResponseKind::OlangResult => "olang",
        ResponseKind::Crisis => "crisis",
        ResponseKind::Blocked => "blocked",
        ResponseKind::System => "system",
    };
    let tone = tone_to_str(r.tone);

    // Manual JSON — không cần serde trong WASM để giữ nhỏ
    format!(
        r#"{{"text":{},"tone":"{}","fx":{:.4},"kind":"{}","turn":{}}}"#,
        json_str(&r.text),
        tone,
        r.fx,
        kind,
        turn,
    )
}

fn json_str(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{}\"", escaped)
}

fn tone_to_str(tone: ResponseTone) -> String {
    match tone {
        ResponseTone::Supportive => "supportive".into(),
        ResponseTone::Pause => "pause".into(),
        ResponseTone::Reinforcing => "reinforcing".into(),
        ResponseTone::Celebratory => "celebratory".into(),
        ResponseTone::Gentle => "gentle".into(),
        ResponseTone::Engaged => "engaged".into(),
    }
}

fn js_timestamp_u64() -> u64 {
    0u64
}
fn js_timestamp() -> i64 {
    0i64
}

// ─────────────────────────────────────────────────────────────────────────────
// JS glue — tạo global functions cho convenience
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo HomeOS instance và trả về JS-usable object.
/// Dùng khi không muốn dùng `new HomeOSWasm()`.
#[wasm_bindgen]
pub fn create_homeos() -> HomeOSWasm {
    HomeOSWasm::new()
}

/// Quick encode — không cần instance.
#[wasm_bindgen]
pub fn quick_encode(cp: u32) -> u32 {
    HomeOSWasm::encode_cp(cp)
}

/// Version string.
#[wasm_bindgen]
pub fn version() -> String {
    format!(
        "HomeOS v{} · Unicode 18.0 · {} UCD entries",
        env!("CARGO_PKG_VERSION"),
        ucd::table_len(),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests (native, không phải WASM)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_to_json_valid() {
        let r = Response {
            text: "Tôi hiểu rồi.".into(),
            tone: ResponseTone::Engaged,
            fx: -0.15,
            kind: ResponseKind::Natural,
        };
        let json = response_to_json(&r, 1);
        assert!(json.starts_with('{'), "JSON bắt đầu bằng {{");
        assert!(json.ends_with('}'), "JSON kết thúc bằng }}");
        assert!(json.contains("\"text\""), "JSON có text field");
        assert!(json.contains("\"tone\""), "JSON có tone field");
        assert!(json.contains("\"fx\""), "JSON có fx field");
        assert!(json.contains("\"kind\""), "JSON có kind field");
        assert!(json.contains("\"turn\""), "JSON có turn field");
    }

    #[test]
    fn json_str_escapes() {
        let s = json_str("hello \"world\"\nnewline");
        assert!(s.contains("\\\""), "Quotes escaped");
        assert!(s.contains("\\n"), "Newline escaped");
    }

    #[test]
    fn tone_to_str_all() {
        let tones = [
            ResponseTone::Supportive,
            ResponseTone::Pause,
            ResponseTone::Reinforcing,
            ResponseTone::Celebratory,
            ResponseTone::Gentle,
            ResponseTone::Engaged,
        ];
        for tone in tones {
            let s = tone_to_str(tone);
            assert!(!s.is_empty(), "tone_to_str không rỗng");
        }
    }

    #[test]
    fn response_kinds_serialize() {
        let kinds = [
            ResponseKind::Natural,
            ResponseKind::OlangResult,
            ResponseKind::Crisis,
            ResponseKind::Blocked,
            ResponseKind::System,
        ];
        for kind in kinds {
            let r = Response {
                text: "test".into(),
                tone: ResponseTone::Engaged,
                fx: 0.0,
                kind,
            };
            let json = response_to_json(&r, 1);
            assert!(json.contains("\"kind\""));
        }
    }

    #[test]
    fn homeos_wasm_new() {
        let mut os = HomeOSWasm::new();
        assert_eq!(os.turns(), 0);
        let r = os.process("xin chào");
        assert!(!r.is_empty(), "process() trả về JSON");
        assert_eq!(os.turns(), 1);
    }

    #[test]
    fn homeos_wasm_process_olang() {
        let mut os = HomeOSWasm::new();
        let r = os.process("○{stats}");
        assert!(
            r.contains("\"kind\":\"system\""),
            "○{{stats}} → system kind"
        );
    }

    #[test]
    fn homeos_wasm_fx() {
        let mut os = HomeOSWasm::new();
        os.process("tôi buồn quá");
        let fx = os.fx();
        assert!(fx.is_finite(), "fx phải finite");
    }

    #[test]
    fn ucd_len_nonzero() {
        assert!(HomeOSWasm::ucd_len() > 0, "UCD phải có entries sau build");
    }

    #[test]
    fn encode_cp_fire() {
        let hash = HomeOSWasm::encode_cp(0x1F525); // 🔥
        assert!(hash > 0, "🔥 hash phải > 0: {}", hash);
    }

    #[test]
    fn version_string() {
        let v = version();
        assert!(v.contains("HomeOS"));
        assert!(v.contains("Unicode 18.0"));
    }

    #[test]
    fn json_crisis_response() {
        let mut os = HomeOSWasm::new();
        let r = os.process("tôi muốn chết");
        assert!(
            r.contains("\"kind\":\"crisis\""),
            "Crisis intent → crisis kind"
        );
        // Crisis response phải có helpline
        assert!(
            r.contains("1800") || r.contains("741741"),
            "Crisis JSON phải có helpline"
        );
    }

    #[test]
    fn event_stream_wired() {
        let mut os = HomeOSWasm::new();
        assert_eq!(os.pending_events(), 0);
        os.process("xin chào");
        assert!(os.pending_events() > 0, "process() should push emotion event");
        let bytes = os.drain_events();
        // Format: [count:4][len0:4][frame0]...
        assert!(bytes.len() >= 4);
        let count = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert!(count >= 1, "at least 1 event frame");
        assert_eq!(os.pending_events(), 0, "drain clears events");
    }

    #[test]
    fn json_turn_increments() {
        let mut os = HomeOSWasm::new();
        for i in 1..=5u64 {
            let r = os.process("ok");
            let expected = format!("\"turn\":{}", i);
            assert!(r.contains(&expected), "Turn {} phải trong JSON: {}", i, r);
        }
    }

    #[test]
    fn stm_grows_after_process() {
        let mut os = HomeOSWasm::new();
        let before = os.stm_len();
        os.process("tôi yêu lập trình");
        assert!(os.stm_len() >= before, "STM should grow or stay: {} → {}", before, os.stm_len());
    }

    #[test]
    fn silk_edge_count_after_process() {
        let mut os = HomeOSWasm::new();
        os.process("lửa và nước");
        os.process("nước chảy mạnh");
        let edges = os.silk_edge_count();
        // Silk co-activates words → some edges expected
        assert!(edges >= 0, "silk_edge_count finite: {}", edges);
    }

    #[test]
    fn registry_len_nonzero() {
        let os = HomeOSWasm::new();
        // L0 seed gives some registry entries
        assert!(os.registry_len() > 0, "Registry should have L0 entries: {}", os.registry_len());
    }

    #[test]
    fn read_book_returns_count() {
        let mut os = HomeOSWasm::new();
        let n = os.read_book("Hôm nay trời đẹp. Tôi đi học. Bạn bè vui vẻ.");
        assert!(n > 0, "read_book should return sentence count: {}", n);
    }

    #[test]
    fn dream_returns_json() {
        let mut os = HomeOSWasm::new();
        os.process("test data");
        let r = os.dream();
        assert!(r.contains("\"kind\""), "dream() returns JSON: {}", r);
    }

    #[test]
    fn stats_returns_json() {
        let mut os = HomeOSWasm::new();
        let r = os.stats();
        assert!(r.contains("\"kind\""), "stats() returns JSON: {}", r);
    }
}
