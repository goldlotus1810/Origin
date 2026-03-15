//! # encoder — ContentEncoder
//!
//! Bản năng L0: kích hoạt tự động khi có bất kỳ input nào.
//! Mọi ContentInput → MolecularChain — cùng 1 format.
//!
//! Text   → tách câu → cụm từ → từ → ký tự → chain
//! Audio  → freq_hz, amplitude, pitch → chain
//! Sensor → nhiệt/ánh sáng/chuyển động → chain
//! Code   → structure/complexity → chain
//! Math   → operator/operands → chain
//! Image  → SDF description → chain (basic)
//! System → event type → chain

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use olang::molecular::MolecularChain;
use olang::encoder::encode_codepoint;
use olang::lca::lca_many;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// ContentInput — mọi loại đầu vào
// ─────────────────────────────────────────────────────────────────────────────

/// Mọi loại content HomeOS có thể nhận và học.
#[derive(Debug, Clone)]
pub enum ContentInput {
    /// Văn bản tự nhiên
    Text {
        content:   String,
        timestamp: i64,
    },
    /// Âm thanh
    Audio {
        freq_hz:     f32,
        amplitude:   f32,
        duration_ms: u32,
        timestamp:   i64,
    },
    /// Sensor vật lý
    Sensor {
        kind:      SensorKind,
        value:     f32,
        timestamp: i64,
    },
    /// Code / program
    Code {
        content:  String,
        language: CodeLang,
        timestamp: i64,
    },
    /// Công thức toán học
    Math {
        expression: String,
        timestamp:  i64,
    },
    /// Sự kiện hệ thống
    System {
        event:     SystemEvent,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorKind {
    Temperature, // °C
    Humidity,    // %
    Light,       // lux
    Motion,      // boolean
    Sound,       // dB
    Power,       // W
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeLang {
    Rust, Python, JavaScript, Go, Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemEvent {
    Boot, Shutdown, NodeCreated, SilkFormed,
    DreamCycle, Error,
}

impl ContentInput {
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::Text   { timestamp, .. } => *timestamp,
            Self::Audio  { timestamp, .. } => *timestamp,
            Self::Sensor { timestamp, .. } => *timestamp,
            Self::Code   { timestamp, .. } => *timestamp,
            Self::Math   { timestamp, .. } => *timestamp,
            Self::System { timestamp, .. } => *timestamp,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EncodedContent — kết quả encode
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả encode một ContentInput.
#[derive(Debug, Clone)]
pub struct EncodedContent {
    /// Chain chính đại diện cho input
    pub chain:     MolecularChain,
    /// EmotionTag của nội dung
    pub emotion:   EmotionTag,
    /// Timestamp
    pub timestamp: i64,
    /// Source type
    pub source:    SourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceKind {
    Text, Audio, Sensor, Code, Math, System,
}

// ─────────────────────────────────────────────────────────────────────────────
// ContentEncoder
// ─────────────────────────────────────────────────────────────────────────────

/// Encode mọi ContentInput → EncodedContent.
///
/// Đây là bản năng L0 — không cần cấu hình, không cần training.
/// Mọi chain đến từ UCD lookup qua encode_codepoint().
pub struct ContentEncoder;

impl ContentEncoder {
    pub fn new() -> Self { Self }

    /// Encode một ContentInput → EncodedContent.
    pub fn encode(&self, input: ContentInput) -> EncodedContent {
        let _ts = input.timestamp();
        match input {
            ContentInput::Text { content, timestamp } =>
                self.encode_text(&content, timestamp),
            ContentInput::Audio { freq_hz, amplitude, duration_ms, timestamp } =>
                self.encode_audio(freq_hz, amplitude, duration_ms, timestamp),
            ContentInput::Sensor { kind, value, timestamp } =>
                self.encode_sensor(kind, value, timestamp),
            ContentInput::Code { content, language, timestamp } =>
                self.encode_code(&content, language, timestamp),
            ContentInput::Math { expression, timestamp } =>
                self.encode_math(&expression, timestamp),
            ContentInput::System { event, timestamp } =>
                self.encode_system(event, timestamp),
        }
    }

    // ── Text ─────────────────────────────────────────────────────────────────

    /// Text → chain qua LCA của các từ.
    ///
    /// Mỗi từ → lookup Unicode codepoints → chain.
    /// LCA(tất cả chains) → chain đại diện câu.
    fn encode_text(&self, text: &str, ts: i64) -> EncodedContent {
        // Collect chains từ các codepoints trong text
        let chains: Vec<MolecularChain> = text.chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
            .take(64) // Tối đa 64 chars để avoid O(n²) LCA
            .map(|c| encode_codepoint(c as u32))
            .filter(|ch| !ch.is_empty())
            .collect();

        let chain = if chains.is_empty() {
            // Fallback: dùng ○ (origin)
            encode_codepoint(0x25CB)
        } else {
            lca_many(&chains)
        };

        // Emotion từ text pattern
        let emotion = text_emotion(text);

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::Text }
    }

    // ── Audio ─────────────────────────────────────────────────────────────────

    /// Audio → chain từ freq và amplitude.
    ///
    /// freq_hz → Musical note codepoint gần nhất
    /// amplitude → Arousal byte
    fn encode_audio(&self, freq_hz: f32, amplitude: f32, _dur: u32, ts: i64) -> EncodedContent {
        // Map freq → Musical note codepoint
        // Dùng MUSICAL group: note values từ UCD
        let note_cp = freq_to_note_cp(freq_hz);
        let chain = encode_codepoint(note_cp);

        // Emotion từ audio
        let _pitch_norm = (freq_hz / 440.0).min(2.0); // A4=440Hz = neutral
        let valence = if freq_hz < 150.0 { -0.4 } // giọng thấp = lo lắng
                      else if freq_hz > 500.0 { 0.3 } // giọng cao = excited
                      else { 0.0 };
        let emotion = EmotionTag::new(valence, amplitude, 0.5, amplitude);

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::Audio }
    }

    // ── Sensor ────────────────────────────────────────────────────────────────

    /// Sensor → chain từ loại sensor và giá trị.
    fn encode_sensor(&self, kind: SensorKind, value: f32, ts: i64) -> EncodedContent {
        // Map sensor kind → EMOTICON codepoint phù hợp
        let cp = match kind {
            SensorKind::Temperature => {
                if value > 35.0      { 0x1F525 } // 🔥 hot
                else if value < 15.0 { 0x2744  } // ❄ cold
                else                 { 0x1F31E } // 🌞 warm
            }
            SensorKind::Humidity    => 0x1F4A7, // 💧
            SensorKind::Light       => 0x1F4A1, // 💡
            SensorKind::Motion      => 0x1F3C3, // 🏃
            SensorKind::Sound       => 0x1F50A, // 🔊
            SensorKind::Power       => 0x26A1,  // ⚡
        };
        let chain = encode_codepoint(cp);

        // Emotion từ sensor value
        let norm = (value / 100.0).min(1.0).max(0.0);
        let valence = match kind {
            SensorKind::Temperature =>
                if value > 38.0 { -0.3 } else if value < 10.0 { -0.2 } else { 0.1 },
            SensorKind::Motion  => if value > 0.5 { 0.0 } else { 0.0 },
            SensorKind::Sound   => if value > 80.0 { -0.2 } else { 0.0 },
            _ => 0.0,
        };
        let emotion = EmotionTag::new(valence, norm * 0.5, 0.5, norm * 0.4);

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::Sensor }
    }

    // ── Code ──────────────────────────────────────────────────────────────────

    /// Code → chain từ structure.
    fn encode_code(&self, content: &str, lang: CodeLang, ts: i64) -> EncodedContent {
        // Code = Math + Relation (logic)
        // Dùng ∘ (RING OPERATOR) = compose — code là tổ hợp operations
        let cp = match lang {
            CodeLang::Rust       => 0x2218, // ∘ Compose
            CodeLang::Python     => 0x2192, // → Flow
            CodeLang::JavaScript => 0x2194, // ↔ Mirror
            CodeLang::Go         => 0x2200, // ∀ ForAll
            CodeLang::Other      => 0x2218, // ∘ default
        };
        let chain = encode_codepoint(cp);

        // Complexity → arousal
        let lines = content.lines().count();
        let complexity = (lines as f32 / 100.0).min(1.0);
        let emotion = EmotionTag::new(0.1, complexity * 0.6, 0.7, complexity * 0.5);

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::Code }
    }

    // ── Math ──────────────────────────────────────────────────────────────────

    /// Math → chain từ operator.
    fn encode_math(&self, expr: &str, ts: i64) -> EncodedContent {
        // Detect operator chính
        let cp = if expr.contains('∫') || expr.contains("integral") {
            0x222B // ∫
        } else if expr.contains('∑') || expr.contains("sum") {
            0x2211 // ∑
        } else if expr.contains('∂') || expr.contains("partial") {
            0x2202 // ∂
        } else if expr.contains('=') {
            0x2261 // ≡ equiv
        } else {
            0x2200 // ∀ default
        };
        let chain = encode_codepoint(cp);
        let emotion = EmotionTag::new(0.0, 0.3, 0.8, 0.4);

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::Math }
    }

    // ── System ────────────────────────────────────────────────────────────────

    /// System event → chain.
    fn encode_system(&self, event: SystemEvent, ts: i64) -> EncodedContent {
        let cp = match event {
            SystemEvent::Boot        => 0x25CB, // ○ origin
            SystemEvent::Shutdown    => 0x1F6D1, // 🛑 stop
            SystemEvent::NodeCreated => 0x2728,  // ✨ spark
            SystemEvent::SilkFormed  => 0x1F578, // 🕸 spider web
            SystemEvent::DreamCycle  => 0x1F319, // 🌙 moon (dream)
            SystemEvent::Error       => 0x26A0,  // ⚠ warning
        };
        let chain = encode_codepoint(cp);
        let emotion = match event {
            SystemEvent::Error    => EmotionTag::new(-0.3, 0.7, 0.5, 0.6),
            SystemEvent::Boot     => EmotionTag::new(0.3, 0.5, 0.7, 0.5),
            SystemEvent::Shutdown => EmotionTag::new(-0.1, 0.2, 0.5, 0.3),
            _                     => EmotionTag::NEUTRAL,
        };

        EncodedContent { chain, emotion, timestamp: ts, source: SourceKind::System }
    }
}

impl Default for ContentEncoder {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Map freq Hz → Musical note codepoint (1D100..1D1FF).
fn freq_to_note_cp(freq_hz: f32) -> u32 {
    // Musical Symbols: 𝅝=0x1D15D (whole), 𝅗=0x1D158 (half),
    // ♩=0x2669 (quarter), ♪=0x266A (eighth)
    if freq_hz < 100.0      { 0x1D15D } // Whole note — very slow/deep
    else if freq_hz < 250.0 { 0x1D158 } // Half note — slow
    else if freq_hz < 500.0 { 0x2669  } // Quarter — medium (speech range)
    else if freq_hz < 1000.0{ 0x266A  } // Eighth — fast (high voice)
    else                    { 0x266B  } // Beamed — very high/rapid
}

/// Detect emotion từ text dùng UTF-8 native.
fn text_emotion(text: &str) -> EmotionTag {
    use context::emotion::word_affect;
    // Aggregate emotion từ các từ trong text
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() { return EmotionTag::NEUTRAL; }

    let mut tv = 0.0f32; let mut ta = 0.0f32;
    let mut td = 0.0f32; let mut ti = 0.0f32;
    for &w in &words {
        let lower = w.to_lowercase();
        let e = word_affect(&lower);
        tv += e.valence; ta += e.arousal;
        td += e.dominance; ti += e.intensity;
    }
    let n = words.len() as f32;
    EmotionTag::new(
        (tv/n).max(-1.0).min(1.0),
        (ta/n).max(0.0).min(1.0),
        (td/n).max(0.0).min(1.0),
        (ti/n).max(0.0).min(1.0),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn enc() -> ContentEncoder { ContentEncoder::new() }

    fn skip() -> bool { ucd::table_len() == 0 }

    // ── Text ─────────────────────────────────────────────────────────────────

    #[test]
    fn encode_text_not_empty() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Text {
            content: "tôi buồn quá hôm nay".to_string(), timestamp: 1000,
        });
        assert!(!r.chain.is_empty(), "Text → chain không rỗng");
        assert_eq!(r.source, SourceKind::Text);
    }

    #[test]
    fn encode_text_emotion_negative() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Text {
            content: "tôi buồn và mệt".to_string(), timestamp: 1000,
        });
        assert!(r.emotion.valence < 0.0,
            "Câu buồn → emotion âm: {}", r.emotion.valence);
    }

    #[test]
    fn encode_text_emoji() {
        if skip() { return; }
        // Text chứa emoji → chain từ UCD của emoji đó
        let r = enc().encode(ContentInput::Text {
            content: "🔥".to_string(), timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    // ── Audio ─────────────────────────────────────────────────────────────────

    #[test]
    fn encode_audio_low_pitch_negative() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 120.0, amplitude: 0.3,
            duration_ms: 500, timestamp: 2000,
        });
        assert!(r.emotion.valence < 0.0,
            "Pitch thấp 120Hz → valence âm: {}", r.emotion.valence);
        assert_eq!(r.source, SourceKind::Audio);
    }

    #[test]
    fn encode_audio_chain_not_empty() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 440.0, amplitude: 0.6,
            duration_ms: 1000, timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    // ── Sensor ────────────────────────────────────────────────────────────────

    #[test]
    fn encode_sensor_fire_temperature() {
        if skip() { return; }
        // 40°C → 🔥 chain
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Temperature, value: 40.0, timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
        assert_eq!(r.source, SourceKind::Sensor);
        // 40°C nóng → valence hơi âm
        assert!(r.emotion.valence < 0.1,
            "40°C nóng → emotion: {}", r.emotion.valence);
    }

    #[test]
    fn encode_sensor_cold_temperature() {
        if skip() { return; }
        // 5°C → ❄ chain
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Temperature, value: 5.0, timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    #[test]
    fn encode_sensor_motion() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Motion, value: 1.0, timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Sensor);
        assert!(!r.chain.is_empty());
    }

    // ── Code ──────────────────────────────────────────────────────────────────

    #[test]
    fn encode_code_rust() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Code {
            content: "fn main() {\n    println!(\"hello\");\n}".to_string(),
            language: CodeLang::Rust, timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Code);
        assert!(!r.chain.is_empty());
    }

    // ── Math ──────────────────────────────────────────────────────────────────

    #[test]
    fn encode_math_integral() {
        if skip() { return; }
        let r = enc().encode(ContentInput::Math {
            expression: "∫ f(x) dx".to_string(), timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Math);
        assert!(!r.chain.is_empty());
    }

    // ── System ────────────────────────────────────────────────────────────────

    #[test]
    fn encode_system_boot() {
        if skip() { return; }
        let r = enc().encode(ContentInput::System {
            event: SystemEvent::Boot, timestamp: 0,
        });
        assert_eq!(r.source, SourceKind::System);
        assert!(r.emotion.valence >= 0.0, "Boot → positive");
    }

    #[test]
    fn encode_system_error_negative() {
        if skip() { return; }
        let r = enc().encode(ContentInput::System {
            event: SystemEvent::Error, timestamp: 1000,
        });
        assert!(r.emotion.valence < 0.0, "Error → negative valence");
    }

    // ── All sources produce chains ────────────────────────────────────────────

    #[test]
    fn all_sources_produce_chains() {
        if skip() { return; }
        let inputs = alloc::vec![
            ContentInput::Text    { content: "test".to_string(), timestamp: 1 },
            ContentInput::Audio   { freq_hz: 440.0, amplitude: 0.5, duration_ms: 100, timestamp: 2 },
            ContentInput::Sensor  { kind: SensorKind::Light, value: 500.0, timestamp: 3 },
            ContentInput::Code    { content: "x = 1".to_string(), language: CodeLang::Python, timestamp: 4 },
            ContentInput::Math    { expression: "x = 1".to_string(), timestamp: 5 },
            ContentInput::System  { event: SystemEvent::NodeCreated, timestamp: 6 },
        ];

        for input in inputs {
            let r = enc().encode(input);
            assert!(!r.chain.is_empty(), "Source {:?} phải tạo chain", r.source);
        }
    }
}
