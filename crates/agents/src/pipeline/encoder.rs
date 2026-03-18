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
use alloc::string::String;
use alloc::vec::Vec;

use olang::encoder::encode_codepoint;
use olang::lca::lca_many;
use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// ContentInput — mọi loại đầu vào
// ─────────────────────────────────────────────────────────────────────────────

/// Mọi loại content HomeOS có thể nhận và học.
#[derive(Debug, Clone)]
pub enum ContentInput {
    /// Văn bản tự nhiên
    Text { content: String, timestamp: i64 },
    /// Âm thanh
    Audio {
        freq_hz: f32,
        amplitude: f32,
        duration_ms: u32,
        timestamp: i64,
    },
    /// Sensor vật lý
    Sensor {
        kind: SensorKind,
        value: f32,
        timestamp: i64,
    },
    /// Code / program
    Code {
        content: String,
        language: CodeLang,
        timestamp: i64,
    },
    /// Công thức toán học
    Math { expression: String, timestamp: i64 },
    /// Hình ảnh → SDF description → chain
    Image {
        /// SDF primitive phát hiện được (0=sphere, 1=box, 2=cylinder, 3=plane, 4=mixed)
        sdf_type: u8,
        /// Độ sáng trung bình [0.0, 1.0]
        brightness: f32,
        /// Điểm chuyển động [0.0, 1.0]
        motion_score: f32,
        /// Số vùng phát hiện (regions / objects)
        region_count: u8,
        timestamp: i64,
    },
    /// Sự kiện hệ thống
    System { event: SystemEvent, timestamp: i64 },
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
    Rust,
    Python,
    JavaScript,
    Go,
    Other,
}

/// SDF primitive type detected in image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdfPrimitive {
    Sphere = 0,
    Box = 1,
    Cylinder = 2,
    Plane = 3,
    Mixed = 4,
}

impl SdfPrimitive {
    /// From raw byte.
    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => Self::Sphere,
            1 => Self::Box,
            2 => Self::Cylinder,
            3 => Self::Plane,
            _ => Self::Mixed,
        }
    }
}

/// Audio frequency band classification for spectral features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBand {
    /// < 60 Hz: sub-bass (rumble, earthquakes)
    SubBass,
    /// 60-250 Hz: bass (drums, bass guitar)
    Bass,
    /// 250-500 Hz: low-mid (voice fundamental)
    LowMid,
    /// 500-2000 Hz: mid (voice harmonics, most instruments)
    Mid,
    /// 2000-6000 Hz: high-mid (consonants, presence)
    HighMid,
    /// > 6000 Hz: treble (cymbals, sibilance)
    Treble,
}

impl AudioBand {
    /// Classify frequency into energy band.
    pub fn from_freq(freq_hz: f32) -> Self {
        if freq_hz < 60.0 {
            Self::SubBass
        } else if freq_hz < 250.0 {
            Self::Bass
        } else if freq_hz < 500.0 {
            Self::LowMid
        } else if freq_hz < 2000.0 {
            Self::Mid
        } else if freq_hz < 6000.0 {
            Self::HighMid
        } else {
            Self::Treble
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemEvent {
    Boot,
    Shutdown,
    NodeCreated,
    SilkFormed,
    DreamCycle,
    Error,
}

impl ContentInput {
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::Text { timestamp, .. } => *timestamp,
            Self::Audio { timestamp, .. } => *timestamp,
            Self::Sensor { timestamp, .. } => *timestamp,
            Self::Code { timestamp, .. } => *timestamp,
            Self::Math { timestamp, .. } => *timestamp,
            Self::Image { timestamp, .. } => *timestamp,
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
    pub chain: MolecularChain,
    /// EmotionTag của nội dung
    pub emotion: EmotionTag,
    /// Timestamp
    pub timestamp: i64,
    /// Source type
    pub source: SourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceKind {
    Text,
    Audio,
    Sensor,
    Code,
    Math,
    Image,
    System,
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
    pub fn new() -> Self {
        Self
    }

    /// Encode một ContentInput → EncodedContent.
    pub fn encode(&self, input: ContentInput) -> EncodedContent {
        let _ts = input.timestamp();
        match input {
            ContentInput::Text { content, timestamp } => self.encode_text(&content, timestamp),
            ContentInput::Audio {
                freq_hz,
                amplitude,
                duration_ms,
                timestamp,
            } => self.encode_audio(freq_hz, amplitude, duration_ms, timestamp),
            ContentInput::Sensor {
                kind,
                value,
                timestamp,
            } => self.encode_sensor(kind, value, timestamp),
            ContentInput::Code {
                content,
                language,
                timestamp,
            } => self.encode_code(&content, language, timestamp),
            ContentInput::Math {
                expression,
                timestamp,
            } => self.encode_math(&expression, timestamp),
            ContentInput::Image {
                sdf_type,
                brightness,
                motion_score,
                region_count,
                timestamp,
            } => self.encode_image(sdf_type, brightness, motion_score, region_count, timestamp),
            ContentInput::System { event, timestamp } => self.encode_system(event, timestamp),
        }
    }

    // ── Text ─────────────────────────────────────────────────────────────────

    /// Text → chain qua LCA của các từ.
    ///
    /// Mỗi từ → lookup Unicode codepoints → chain.
    /// LCA(tất cả chains) → chain đại diện câu.
    fn encode_text(&self, text: &str, ts: i64) -> EncodedContent {
        // Collect chains từ các codepoints trong text
        let chains: Vec<MolecularChain> = text
            .chars()
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

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Text,
        }
    }

    // ── Audio ─────────────────────────────────────────────────────────────────

    /// Audio → chain từ freq, amplitude, duration.
    ///
    /// Spectral features extracted:
    ///   1. Fundamental frequency → Musical note codepoint
    ///   2. Energy band classification (sub-bass/bass/mid/treble)
    ///   3. Spectral centroid estimate → brightness perception
    ///   4. Zero-crossing rate estimate → noisiness
    ///   5. Duration → temporal character
    fn encode_audio(&self, freq_hz: f32, amplitude: f32, dur_ms: u32, ts: i64) -> EncodedContent {
        // Feature 1: Fundamental frequency → Musical note codepoint
        let note_cp = freq_to_note_cp(freq_hz);

        // Feature 2: Energy band classification
        let band = AudioBand::from_freq(freq_hz);
        let band_cp = match band {
            AudioBand::SubBass => 0x1D122, // 𝄢 F CLEF (deep bass)
            AudioBand::Bass => 0x1D15D,    // 𝅝 WHOLE NOTE
            AudioBand::LowMid => 0x2669,   // ♩ QUARTER NOTE
            AudioBand::Mid => 0x266A,      // ♪ EIGHTH NOTE
            AudioBand::HighMid => 0x266B,  // ♫ BEAMED EIGHTH NOTES
            AudioBand::Treble => 0x1D11E,  // 𝄞 G CLEF (high)
        };

        // Feature 3: Spectral centroid estimate (brightness)
        // Higher centroid = brighter sound = more treble energy
        let spectral_brightness = (freq_hz / 4000.0).clamp(0.0, 1.0);

        // Feature 4: Noisiness estimate from amplitude variability
        // Low amplitude + high freq → likely noise; high amplitude + mid freq → likely voice
        let noisiness = if amplitude < 0.2 && freq_hz > 2000.0 {
            0.8 // likely noise
        } else if amplitude > 0.5 && (100.0..=1000.0).contains(&freq_hz) {
            0.1 // likely voice/music
        } else {
            0.4 // uncertain
        };

        // Multi-feature chain: combine note + band for richer encoding
        let chain = if amplitude > 0.3 {
            // Strong signal: encode both pitch and band character
            let chains = alloc::vec![
                encode_codepoint(note_cp),
                encode_codepoint(band_cp),
            ];
            lca_many(&chains)
        } else {
            encode_codepoint(note_cp)
        };

        // Feature 5: Duration affects temporal character
        let temporal_weight = if dur_ms > 2000 {
            0.3 // sustained → contemplative
        } else if dur_ms < 100 {
            0.8 // percussive → exciting
        } else {
            0.5
        };

        // Emotion from spectral features
        let valence = if freq_hz < 150.0 {
            -0.4 // deep = somber
        } else if freq_hz > 500.0 {
            0.2 + spectral_brightness * 0.2 // bright = positive
        } else {
            0.0 // neutral speech range
        };
        let arousal = amplitude * (1.0 - noisiness * 0.5) + temporal_weight * 0.2;
        let dominance = 0.5 + (amplitude - 0.5) * 0.3; // loud = dominant
        let intensity = amplitude.max(0.1);
        let emotion = EmotionTag::new(valence, arousal.clamp(0.0, 1.0), dominance, intensity);

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Audio,
        }
    }

    // ── Sensor ────────────────────────────────────────────────────────────────

    /// Sensor → chain từ loại sensor và giá trị.
    fn encode_sensor(&self, kind: SensorKind, value: f32, ts: i64) -> EncodedContent {
        // Map sensor kind → EMOTICON codepoint phù hợp
        let cp = match kind {
            SensorKind::Temperature => {
                if value > 35.0 {
                    0x1F525
                }
                // 🔥 hot
                else if value < 15.0 {
                    0x2744
                }
                // ❄ cold
                else {
                    0x1F31E
                } // 🌞 warm
            }
            SensorKind::Humidity => 0x1F4A7, // 💧
            SensorKind::Light => 0x1F4A1,    // 💡
            SensorKind::Motion => 0x1F3C3,   // 🏃
            SensorKind::Sound => 0x1F50A,    // 🔊
            SensorKind::Power => 0x26A1,     // ⚡
        };
        let chain = encode_codepoint(cp);

        // Emotion từ sensor value
        let norm = (value / 100.0).clamp(0.0, 1.0);
        let valence = match kind {
            SensorKind::Temperature => {
                if value > 38.0 {
                    -0.3
                } else if value < 10.0 {
                    -0.2
                } else {
                    0.1
                }
            }
            SensorKind::Motion => 0.0,
            SensorKind::Sound => {
                if value > 80.0 {
                    -0.2
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        let emotion = EmotionTag::new(valence, norm * 0.5, 0.5, norm * 0.4);

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Sensor,
        }
    }

    // ── Code ──────────────────────────────────────────────────────────────────

    /// Code → chain từ structure.
    fn encode_code(&self, content: &str, lang: CodeLang, ts: i64) -> EncodedContent {
        // Code = Math + Relation (logic)
        // Dùng ∘ (RING OPERATOR) = compose — code là tổ hợp operations
        let cp = match lang {
            CodeLang::Rust => 0x2218,       // ∘ Compose
            CodeLang::Python => 0x2192,     // → Flow
            CodeLang::JavaScript => 0x2194, // ↔ Mirror
            CodeLang::Go => 0x2200,         // ∀ ForAll
            CodeLang::Other => 0x2218,      // ∘ default
        };
        let chain = encode_codepoint(cp);

        // Complexity → arousal
        let lines = content.lines().count();
        let complexity = (lines as f32 / 100.0).min(1.0);
        let emotion = EmotionTag::new(0.1, complexity * 0.6, 0.7, complexity * 0.5);

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Code,
        }
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

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Math,
        }
    }

    // ── Image ─────────────────────────────────────────────────────────────────

    /// Image → chain từ SDF description.
    ///
    /// Pipeline: pixel features → SDF fitting → geometric chain → emotion
    ///
    /// SDF fitting from pixel features:
    ///   1. edge_ratio (horizontal vs vertical edges) → determine shape
    ///   2. circularity (edge uniformity) → sphere vs box
    ///   3. brightness + motion → emotion
    ///   4. region_count → scene complexity → chain depth
    fn encode_image(
        &self,
        sdf_type: u8,
        brightness: f32,
        motion_score: f32,
        region_count: u8,
        ts: i64,
    ) -> EncodedContent {
        // Map SDF primitive → geometric codepoint
        let prim = SdfPrimitive::from_byte(sdf_type);
        let primary_cp = match prim {
            SdfPrimitive::Sphere => 0x25CF,   // ● BLACK CIRCLE
            SdfPrimitive::Box => 0x25A0,      // ■ BLACK SQUARE
            SdfPrimitive::Cylinder => 0x25AD,  // ▭ WHITE RECTANGLE (cylinder projection)
            SdfPrimitive::Plane => 0x25B3,     // △ WHITE UP-POINTING TRIANGLE
            SdfPrimitive::Mixed => 0x25C6,     // ◆ BLACK DIAMOND (composite)
        };

        // Multi-region scenes: encode each region as sub-chain, then LCA
        let chain = if region_count > 1 {
            // Complex scene: encode primary + secondary geometric primitives
            let mut chains = Vec::new();
            chains.push(encode_codepoint(primary_cp));
            // Secondary codepoints for scene complexity
            if region_count >= 2 {
                // Size indicator: small=0x25AA(▪), large=0x25A0(■)
                let size_cp = if brightness > 0.6 { 0x25CB } else { 0x25CF }; // ○ vs ●
                chains.push(encode_codepoint(size_cp));
            }
            if region_count >= 4 {
                // Spatial indicator: distributed objects
                chains.push(encode_codepoint(0x2234)); // ∴ THEREFORE (multiple points)
            }
            lca_many(&chains)
        } else {
            encode_codepoint(primary_cp)
        };

        // Emotion from visual content:
        // - brightness → valence (dark = slightly negative, bright = slightly positive)
        // - motion → arousal (movement = excitement)
        // - region_count → intensity (complex scene = more intense)
        // - SDF type affects dominance (plane=stable, mixed=chaotic)
        let valence = (brightness - 0.5) * 0.4; // [-0.2, +0.2]
        let arousal = motion_score.clamp(0.0, 1.0) * 0.7 + 0.1;
        let intensity = ((region_count as f32) / 10.0).clamp(0.1, 0.8);
        let dominance = match prim {
            SdfPrimitive::Plane => 0.7,    // stable
            SdfPrimitive::Sphere => 0.5,   // neutral
            SdfPrimitive::Box => 0.6,      // structured
            SdfPrimitive::Cylinder => 0.5, // neutral
            SdfPrimitive::Mixed => 0.3,    // chaotic
        };
        let emotion = EmotionTag::new(valence, arousal, dominance, intensity);

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::Image,
        }
    }

    // ── System ────────────────────────────────────────────────────────────────

    /// System event → chain.
    fn encode_system(&self, event: SystemEvent, ts: i64) -> EncodedContent {
        let cp = match event {
            SystemEvent::Boot => 0x25CB,        // ○ origin
            SystemEvent::Shutdown => 0x1F6D1,   // 🛑 stop
            SystemEvent::NodeCreated => 0x2728, // ✨ spark
            SystemEvent::SilkFormed => 0x1F578, // 🕸 spider web
            SystemEvent::DreamCycle => 0x1F319, // 🌙 moon (dream)
            SystemEvent::Error => 0x26A0,       // ⚠ warning
        };
        let chain = encode_codepoint(cp);
        let emotion = match event {
            SystemEvent::Error => EmotionTag::new(-0.3, 0.7, 0.5, 0.6),
            SystemEvent::Boot => EmotionTag::new(0.3, 0.5, 0.7, 0.5),
            SystemEvent::Shutdown => EmotionTag::new(-0.1, 0.2, 0.5, 0.3),
            _ => EmotionTag::NEUTRAL,
        };

        EncodedContent {
            chain,
            emotion,
            timestamp: ts,
            source: SourceKind::System,
        }
    }
}

impl Default for ContentEncoder {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Map freq Hz → Musical note codepoint (1D100..1D1FF).
fn freq_to_note_cp(freq_hz: f32) -> u32 {
    // Musical Symbols: 𝅝=0x1D15D (whole), 𝅗=0x1D158 (half),
    // ♩=0x2669 (quarter), ♪=0x266A (eighth)
    if freq_hz < 100.0 {
        0x1D15D
    }
    // Whole note — very slow/deep
    else if freq_hz < 250.0 {
        0x1D158
    }
    // Half note — slow
    else if freq_hz < 500.0 {
        0x2669
    }
    // Quarter — medium (speech range)
    else if freq_hz < 1000.0 {
        0x266A
    }
    // Eighth — fast (high voice)
    else {
        0x266B
    } // Beamed — very high/rapid
}

/// Detect emotion từ text dùng UTF-8 native.
fn text_emotion(text: &str) -> EmotionTag {
    use context::emotion::word_affect;
    // Aggregate emotion từ các từ trong text
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return EmotionTag::NEUTRAL;
    }

    let mut tv = 0.0f32;
    let mut ta = 0.0f32;
    let mut td = 0.0f32;
    let mut ti = 0.0f32;
    for &w in &words {
        let lower = w.to_lowercase();
        let e = word_affect(&lower);
        tv += e.valence;
        ta += e.arousal;
        td += e.dominance;
        ti += e.intensity;
    }
    let n = words.len() as f32;
    EmotionTag::new(
        (tv / n).clamp(-1.0, 1.0),
        (ta / n).clamp(0.0, 1.0),
        (td / n).clamp(0.0, 1.0),
        (ti / n).clamp(0.0, 1.0),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn enc() -> ContentEncoder {
        ContentEncoder::new()
    }


    // ── Text ─────────────────────────────────────────────────────────────────

    #[test]
    fn encode_text_not_empty() {
        let r = enc().encode(ContentInput::Text {
            content: "tôi buồn quá hôm nay".to_string(),
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty(), "Text → chain không rỗng");
        assert_eq!(r.source, SourceKind::Text);
    }

    #[test]
    fn encode_text_emotion_negative() {
        let r = enc().encode(ContentInput::Text {
            content: "tôi buồn và mệt".to_string(),
            timestamp: 1000,
        });
        assert!(
            r.emotion.valence < 0.0,
            "Câu buồn → emotion âm: {}",
            r.emotion.valence
        );
    }

    #[test]
    fn encode_text_emoji() {
        // Text chứa emoji → chain từ UCD của emoji đó
        let r = enc().encode(ContentInput::Text {
            content: "🔥".to_string(),
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    // ── Audio ─────────────────────────────────────────────────────────────────

    #[test]
    fn encode_audio_low_pitch_negative() {
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 120.0,
            amplitude: 0.3,
            duration_ms: 500,
            timestamp: 2000,
        });
        assert!(
            r.emotion.valence < 0.0,
            "Pitch thấp 120Hz → valence âm: {}",
            r.emotion.valence
        );
        assert_eq!(r.source, SourceKind::Audio);
    }

    #[test]
    fn encode_audio_chain_not_empty() {
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 440.0,
            amplitude: 0.6,
            duration_ms: 1000,
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    // ── Sensor ────────────────────────────────────────────────────────────────

    #[test]
    fn encode_sensor_fire_temperature() {
        // 40°C → 🔥 chain
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Temperature,
            value: 40.0,
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
        assert_eq!(r.source, SourceKind::Sensor);
        // 40°C nóng → valence hơi âm
        assert!(
            r.emotion.valence < 0.1,
            "40°C nóng → emotion: {}",
            r.emotion.valence
        );
    }

    #[test]
    fn encode_sensor_cold_temperature() {
        // 5°C → ❄ chain
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Temperature,
            value: 5.0,
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
    }

    #[test]
    fn encode_sensor_motion() {
        let r = enc().encode(ContentInput::Sensor {
            kind: SensorKind::Motion,
            value: 1.0,
            timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Sensor);
        assert!(!r.chain.is_empty());
    }

    // ── Code ──────────────────────────────────────────────────────────────────

    #[test]
    fn encode_code_rust() {
        let r = enc().encode(ContentInput::Code {
            content: "fn main() {\n    println!(\"hello\");\n}".to_string(),
            language: CodeLang::Rust,
            timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Code);
        assert!(!r.chain.is_empty());
    }

    // ── Math ──────────────────────────────────────────────────────────────────

    #[test]
    fn encode_math_integral() {
        let r = enc().encode(ContentInput::Math {
            expression: "∫ f(x) dx".to_string(),
            timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Math);
        assert!(!r.chain.is_empty());
    }

    // ── System ────────────────────────────────────────────────────────────────

    #[test]
    fn encode_system_boot() {
        let r = enc().encode(ContentInput::System {
            event: SystemEvent::Boot,
            timestamp: 0,
        });
        assert_eq!(r.source, SourceKind::System);
        assert!(r.emotion.valence >= 0.0, "Boot → positive");
    }

    #[test]
    fn encode_system_error_negative() {
        let r = enc().encode(ContentInput::System {
            event: SystemEvent::Error,
            timestamp: 1000,
        });
        assert!(r.emotion.valence < 0.0, "Error → negative valence");
    }

    // ── Image ─────────────────────────────────────────────────────────────────

    #[test]
    fn encode_image_sphere() {
        let r = enc().encode(ContentInput::Image {
            sdf_type: 0, // Sphere
            brightness: 0.7,
            motion_score: 0.0,
            region_count: 1,
            timestamp: 1000,
        });
        assert_eq!(r.source, SourceKind::Image);
        assert!(!r.chain.is_empty());
        // Bright scene → positive valence
        assert!(r.emotion.valence > 0.0, "Bright → positive: {}", r.emotion.valence);
    }

    #[test]
    fn encode_image_dark_negative() {
        let r = enc().encode(ContentInput::Image {
            sdf_type: 1, // Box
            brightness: 0.1,
            motion_score: 0.0,
            region_count: 1,
            timestamp: 1000,
        });
        // Dark scene → slightly negative valence
        assert!(r.emotion.valence < 0.0, "Dark → negative: {}", r.emotion.valence);
    }

    #[test]
    fn encode_image_motion_high_arousal() {
        let r = enc().encode(ContentInput::Image {
            sdf_type: 4, // Mixed
            brightness: 0.5,
            motion_score: 0.9,
            region_count: 5,
            timestamp: 1000,
        });
        assert!(r.emotion.arousal > 0.5, "Motion → high arousal: {}", r.emotion.arousal);
    }

    // ── All sources produce chains ────────────────────────────────────────────

    #[test]
    fn all_sources_produce_chains() {
        let inputs = alloc::vec![
            ContentInput::Text {
                content: "test".to_string(),
                timestamp: 1
            },
            ContentInput::Audio {
                freq_hz: 440.0,
                amplitude: 0.5,
                duration_ms: 100,
                timestamp: 2
            },
            ContentInput::Sensor {
                kind: SensorKind::Light,
                value: 500.0,
                timestamp: 3
            },
            ContentInput::Code {
                content: "x = 1".to_string(),
                language: CodeLang::Python,
                timestamp: 4
            },
            ContentInput::Math {
                expression: "x = 1".to_string(),
                timestamp: 5
            },
            ContentInput::Image {
                sdf_type: 0,
                brightness: 0.5,
                motion_score: 0.2,
                region_count: 2,
                timestamp: 6
            },
            ContentInput::System {
                event: SystemEvent::NodeCreated,
                timestamp: 7
            },
        ];

        for input in inputs {
            let r = enc().encode(input);
            assert!(!r.chain.is_empty(), "Source {:?} phải tạo chain", r.source);
        }
    }

    // ── AudioBand classification ────────────────────────────────────────────

    #[test]
    fn audio_band_sub_bass() {
        assert!(matches!(AudioBand::from_freq(30.0), AudioBand::SubBass));
    }

    #[test]
    fn audio_band_bass() {
        assert!(matches!(AudioBand::from_freq(100.0), AudioBand::Bass));
    }

    #[test]
    fn audio_band_mid() {
        assert!(matches!(AudioBand::from_freq(1000.0), AudioBand::Mid));
    }

    #[test]
    fn audio_band_treble() {
        assert!(matches!(AudioBand::from_freq(8000.0), AudioBand::Treble));
    }

    #[test]
    fn audio_band_boundaries() {
        // Boundary: exactly 60 Hz → Bass (not SubBass)
        assert!(matches!(AudioBand::from_freq(60.0), AudioBand::Bass));
        // Boundary: exactly 250 Hz → LowMid
        assert!(matches!(AudioBand::from_freq(250.0), AudioBand::LowMid));
        // Boundary: exactly 6000 Hz → Treble
        assert!(matches!(AudioBand::from_freq(6000.0), AudioBand::Treble));
    }

    // ── Multi-region image encoding ─────────────────────────────────────────

    #[test]
    fn encode_image_multi_region() {
        let r = enc().encode(ContentInput::Image {
            sdf_type: 4,  // Mixed
            brightness: 0.6,
            motion_score: 0.3,
            region_count: 8, // many regions
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
        // Multiple regions → LCA chain should have multiple molecules
        assert!(!r.chain.0.is_empty(), "Multi-region image encodes");
    }

    // ── Audio spectral features ─────────────────────────────────────────────

    #[test]
    fn encode_audio_loud_high_freq() {
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 8000.0, // treble
            amplitude: 0.9,  // loud
            duration_ms: 500,
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
        assert!(r.emotion.arousal > 0.5, "Loud treble → high arousal: {}", r.emotion.arousal);
    }

    #[test]
    fn encode_audio_quiet_low_freq() {
        let r = enc().encode(ContentInput::Audio {
            freq_hz: 40.0,   // sub-bass
            amplitude: 0.1,  // quiet
            duration_ms: 2000,
            timestamp: 1000,
        });
        assert!(!r.chain.is_empty());
        assert!(r.emotion.arousal < 0.5, "Quiet sub-bass → low arousal: {}", r.emotion.arousal);
    }
}
