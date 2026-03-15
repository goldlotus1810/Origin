//! # modality — AudioToEmotionTag + ImageToEmotionTag
//!
//! Thuật toán convert audio/image features → EmotionTag.
//!
//! Audio:
//!   pitch (Hz)   → arousal (cao = kích động, thấp = buồn)
//!   energy [0,1] → intensity
//!   tempo (BPM)  → arousal modifier
//!   voice_break  → distress signal → valence âm
//!
//! Image:
//!   hue [0,360]  → valence (đỏ/cam = kích thích, xanh = bình yên)
//!   saturation   → intensity
//!   brightness   → arousal
//!   motion [0,1] → arousal boost
//!
//! Kết quả feed vào fuse() cùng text EmotionTag:
//!   Cross-modal conflict → confidence giảm → quan sát thêm

extern crate alloc;
use silk::edge::EmotionTag;
use crate::fusion::{ModalityInput, ModalityKind};

// ─────────────────────────────────────────────────────────────────────────────
// AudioFeatures
// ─────────────────────────────────────────────────────────────────────────────

/// Đặc trưng âm thanh đã extract.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct AudioFeatures {
    /// Pitch cơ bản (Hz). 0 = không có giọng nói.
    pub pitch_hz:    f32,
    /// Năng lượng tổng [0,1].
    pub energy:      f32,
    /// Tempo (BPM). 0 = không xác định.
    pub tempo_bpm:   f32,
    /// Giọng bị bẻ/run [0,1] — distress signal.
    pub voice_break: f32,
    /// Spectral centroid (brightness of sound) [0,1].
    pub brightness:  f32,
}

impl AudioFeatures {
    /// Giọng bình thường (nói chuyện bình thường).
    pub fn normal_speech() -> Self {
        Self { pitch_hz: 180.0, energy: 0.4, tempo_bpm: 120.0, voice_break: 0.0, brightness: 0.5 }
    }

    /// Convert → EmotionTag.
    ///
    /// Thuật toán:
    ///   pitch:       150-250Hz = neutral, <150 = buồn/lo, >300 = kích động
    ///   energy:      cao = mạnh mẽ/kích động, thấp = mệt mỏi
    ///   tempo:       nhanh = arousal cao, chậm = buồn/bình yên
    ///   voice_break: > 0.3 → distress, valence âm mạnh
    pub fn to_emotion(self) -> EmotionTag {
        // ── Valence từ pitch ──────────────────────────────────────────────
        let valence = pitch_to_valence(self.pitch_hz);

        // Distress override: giọng run → âm mạnh
        let valence = if self.voice_break > 0.3 {
            valence - self.voice_break * 0.6
        } else {
            valence
        };

        // ── Arousal từ energy + tempo ─────────────────────────────────────
        let tempo_norm = if self.tempo_bpm > 0.0 {
            ((self.tempo_bpm - 60.0) / 120.0).clamp(0.0, 1.0) // 60BPM=0, 180BPM=1
        } else { 0.5 };
        let arousal = (self.energy * 0.6 + tempo_norm * 0.4).clamp(0.0, 1.0);

        // ── Dominance: giọng to + pitch thấp = dominance cao ─────────────
        let dominance = (self.energy * 0.7 + (1.0 - self.brightness) * 0.3).clamp(0.1, 0.9);

        // ── Intensity ────────────────────────────────────────────────────
        let intensity = (valence.abs() * 0.5 + arousal * 0.5).clamp(0.0, 1.0);

        EmotionTag {
            valence:   valence.clamp(-1.0, 1.0),
            arousal,
            dominance,
            intensity,
        }
    }

    /// Tạo ModalityInput cho fuse().
    pub fn to_modality_input(self, confidence: f32) -> ModalityInput {
        ModalityInput {
            tag:        self.to_emotion(),
            confidence: confidence.clamp(0.0, 1.0),
            source:     ModalityKind::Audio,
        }
    }
}

/// Map pitch Hz → valence.
///
/// Dựa trên nghiên cứu prosody:
///   <100Hz → rất buồn/đau
///   100-150 → buồn
///   150-250 → neutral (giọng nói bình thường)
///   250-350 → tích cực/hứng khởi
///   >400Hz  → lo lắng/sợ (pitch cao bất thường)
fn pitch_to_valence(pitch: f32) -> f32 {
    if pitch <= 0.0   { return -0.20; } // không có giọng
    if pitch < 100.0  { return -0.65; } // rất thấp → buồn/đau
    if pitch < 150.0  { return -0.30; } // thấp → buồn
    if pitch < 250.0  { return  0.05; } // bình thường
    if pitch < 350.0  { return  0.35; } // cao → tích cực
    if pitch < 450.0  { return  0.15; } // rất cao → hứng khởi
    -0.20                               // quá cao → lo lắng
}

// ─────────────────────────────────────────────────────────────────────────────
// ImageFeatures
// ─────────────────────────────────────────────────────────────────────────────

/// Đặc trưng hình ảnh đã extract.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct ImageFeatures {
    /// Hue trung bình [0, 360]. 0/360=đỏ, 120=xanh lá, 240=xanh dương.
    pub hue:        f32,
    /// Saturation trung bình [0, 1].
    pub saturation: f32,
    /// Brightness trung bình [0, 1].
    pub brightness: f32,
    /// Mức độ chuyển động [0, 1]. 0 = tĩnh, 1 = rất nhiều chuyển động.
    pub motion:     f32,
    /// Face detected: valence từ facial expression [-1, 1].
    /// None = không detect được mặt.
    pub face_valence: Option<f32>,
}

impl ImageFeatures {
    /// Convert → EmotionTag.
    ///
    /// Thuật toán:
    ///   hue:        đỏ/cam (0-60, 300-360) = arousal cao, warm
    ///               xanh dương (200-260) = bình yên, valence dương
    ///               xanh lá (100-150) = trung tính/tươi
    ///               xám/tối (saturation thấp) = buồn
    ///   saturation: cao = intense, thấp = muted/buồn
    ///   brightness: thấp = tối = buồn, cao = vui/rõ ràng
    ///   motion:     nhiều chuyển động = arousal cao
    ///   face:       override nếu có (nhận diện khuôn mặt tin cậy hơn màu sắc)
    pub fn to_emotion(self) -> EmotionTag {
        // Nếu có face detection → ưu tiên
        if let Some(fv) = self.face_valence {
            let arousal = (self.motion * 0.4 + self.saturation * 0.3
                           + self.brightness * 0.3).clamp(0.1, 0.9);
            return EmotionTag {
                valence:   fv.clamp(-1.0, 1.0),
                arousal,
                dominance: 0.5,
                intensity: (fv.abs() * 0.7 + arousal * 0.3).clamp(0.0, 1.0),
            };
        }

        // ── Valence từ hue + brightness ──────────────────────────────────
        let hue_v   = hue_to_valence(self.hue, self.saturation);
        // Tối = buồn, sáng = vui (trọng số nhỏ hơn hue)
        let bright_v = (self.brightness - 0.4) * 0.5;
        // Màu nhạt (saturation thấp) = buồn nhẹ
        let sat_v    = if self.saturation < 0.2 { -0.20 } else { 0.0 };
        let valence  = (hue_v + bright_v * 0.3 + sat_v).clamp(-1.0, 1.0);

        // ── Arousal từ motion + saturation + brightness ───────────────────
        let arousal = (self.motion * 0.5
                       + self.saturation * 0.3
                       + self.brightness * 0.2).clamp(0.1, 0.9);

        // ── Dominance từ brightness (sáng = chủ động) ─────────────────────
        let dominance = (0.3 + self.brightness * 0.4).clamp(0.1, 0.9);

        let intensity = (valence.abs() * 0.6 + arousal * 0.4).clamp(0.0, 1.0);

        EmotionTag { valence, arousal, dominance, intensity }
    }

    /// Tạo ModalityInput cho fuse().
    pub fn to_modality_input(self, confidence: f32) -> ModalityInput {
        ModalityInput {
            tag:        self.to_emotion(),
            confidence: confidence.clamp(0.0, 1.0),
            source:     ModalityKind::Image,
        }
    }
}

/// Map hue → valence component.
///
/// Dựa trên color psychology:
///   Đỏ (0,360):   kích thích, nguy hiểm, tình yêu → V nhẹ âm hoặc dương tùy context
///   Cam (30):     ấm áp, năng lượng → V dương
///   Vàng (60):    vui vẻ, lạc quan → V dương
///   Xanh lá (120): tự nhiên, bình yên → V dương nhẹ
///   Lam (180-240): bình yên, tin tưởng → V dương
///   Xanh dương (240): buồn, lạnh → V âm nhẹ
///   Tím (270-300): bí ẩn → V neutral
fn hue_to_valence(hue: f32, sat: f32) -> f32 {
    // Nếu saturation thấp → hue không có nghĩa nhiều
    let weight = sat.clamp(0.0, 1.0);

    let base = if !(30.0..=330.0).contains(&hue) {
        -0.10 // đỏ: nguy hiểm/kích thích — neutral-negative
    } else if hue < 60.0 {
         0.30 // cam: ấm áp
    } else if hue < 90.0 {
         0.35 // vàng: vui
    } else if hue < 150.0 {
         0.20 // xanh lá: tươi mát
    } else if hue < 200.0 {
         0.25 // lam: bình yên
    } else if hue < 260.0 {
        -0.15 // xanh dương: lạnh/buồn
    } else {
         0.05 // tím: trung tính
    };

    base * weight
}

// ─────────────────────────────────────────────────────────────────────────────
// BioFeatures
// ─────────────────────────────────────────────────────────────────────────────

/// Đặc trưng sinh học (không thể giả — tin cậy nhất).
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct BioFeatures {
    /// Nhịp tim (BPM). Bình thường: 60-100.
    pub heart_rate:  f32,
    /// Biến thiên nhịp tim [0,1]. Cao = thư giãn.
    pub hrv:         f32,
    /// Nhiệt độ da (°C). Bình thường: 32-36.
    pub skin_temp:   f32,
    /// Galvanic skin response [0,1]. Cao = căng thẳng.
    pub gsr:         f32,
}

impl BioFeatures {
    /// Convert → EmotionTag.
    ///
    /// Bio là nguồn đáng tin cậy nhất — khó giả.
    pub fn to_emotion(self) -> EmotionTag {
        // Nhịp tim: >100 = kích động, <60 = bình yên/chậm
        let hr_norm = ((self.heart_rate - 60.0) / 60.0).clamp(-0.5, 1.0);
        let arousal = (hr_norm * 0.5 + self.gsr * 0.5).clamp(0.0, 1.0);

        // HRV cao = thư giãn = valence dương
        // GSR cao = căng thẳng = valence âm
        let valence = (self.hrv * 0.4 - self.gsr * 0.6).clamp(-1.0, 1.0);

        // Nhiệt độ da thấp = lo lắng (vasoconstriction)
        let temp_signal = if self.skin_temp < 31.0 { -0.20 }
                          else if self.skin_temp > 35.0 { 0.10 }
                          else { 0.0 };
        let valence = (valence + temp_signal).clamp(-1.0, 1.0);

        let intensity = (valence.abs() * 0.5 + arousal * 0.5).clamp(0.0, 1.0);

        EmotionTag { valence, arousal, dominance: 0.5, intensity }
    }

    /// Tạo ModalityInput (bio = highest base weight = 0.50).
    pub fn to_modality_input(self, confidence: f32) -> ModalityInput {
        ModalityInput {
            tag:        self.to_emotion(),
            confidence: confidence.clamp(0.0, 1.0),
            source:     ModalityKind::Bio,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience: fuse_all
// ─────────────────────────────────────────────────────────────────────────────

/// Fuse text + audio + image + bio cùng lúc.
///
/// Bất kỳ source nào có thể là None — tự động bỏ qua.
pub fn fuse_all(
    text:  Option<ModalityInput>,
    audio: Option<AudioFeatures>,
    image: Option<ImageFeatures>,
    bio:   Option<BioFeatures>,
) -> crate::fusion::FusedEmotionTag {
    let mut inputs: alloc::vec::Vec<ModalityInput> = alloc::vec::Vec::new();

    if let Some(t) = text  { inputs.push(t); }
    if let Some(a) = audio { inputs.push(a.to_modality_input(0.85)); }
    if let Some(i) = image { inputs.push(i.to_modality_input(0.70)); }
    if let Some(b) = bio   { inputs.push(b.to_modality_input(0.95)); }

    crate::fusion::fuse(&inputs)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::fuse;

    // ── AudioFeatures ─────────────────────────────────────────────────────────

    #[test]
    fn low_pitch_sad() {
        let a = AudioFeatures { pitch_hz: 110.0, energy: 0.3, tempo_bpm: 70.0,
                                voice_break: 0.0, brightness: 0.4 };
        let e = a.to_emotion();
        assert!(e.valence < -0.10, "Giọng thấp → buồn: {}", e.valence);
    }

    #[test]
    fn high_pitch_excited() {
        let a = AudioFeatures { pitch_hz: 300.0, energy: 0.8, tempo_bpm: 150.0,
                                voice_break: 0.0, brightness: 0.7 };
        let e = a.to_emotion();
        assert!(e.arousal > 0.5, "Giọng cao + nhanh → arousal cao: {}", e.arousal);
    }

    #[test]
    fn voice_break_distress() {
        let normal = AudioFeatures { pitch_hz: 180.0, energy: 0.4, tempo_bpm: 120.0,
                                     voice_break: 0.0, brightness: 0.5 };
        let breaking = AudioFeatures { voice_break: 0.8, ..normal };
        assert!(breaking.to_emotion().valence < normal.to_emotion().valence,
            "Giọng run → valence thấp hơn");
    }

    #[test]
    fn fast_tempo_high_arousal() {
        let slow = AudioFeatures { tempo_bpm: 60.0,  ..AudioFeatures::normal_speech() };
        let fast = AudioFeatures { tempo_bpm: 180.0, ..AudioFeatures::normal_speech() };
        assert!(fast.to_emotion().arousal > slow.to_emotion().arousal,
            "Nhịp nhanh → arousal cao hơn");
    }

    // ── ImageFeatures ─────────────────────────────────────────────────────────

    #[test]
    fn yellow_hue_positive() {
        let img = ImageFeatures { hue: 60.0, saturation: 0.8, brightness: 0.8,
                                  motion: 0.1, face_valence: None };
        assert!(img.to_emotion().valence > 0.0, "Màu vàng → positive");
    }

    #[test]
    fn blue_hue_negative() {
        let img = ImageFeatures { hue: 240.0, saturation: 0.7, brightness: 0.4,
                                  motion: 0.0, face_valence: None };
        assert!(img.to_emotion().valence < 0.1, "Xanh dương + tối → neutral/negative");
    }

    #[test]
    fn face_detection_overrides_color() {
        let img = ImageFeatures {
            hue: 240.0, saturation: 0.8, brightness: 0.3, motion: 0.0,
            face_valence: Some(0.7), // mặt đang cười dù màu xanh tối
        };
        assert!(img.to_emotion().valence > 0.5, "Face override màu sắc: {}", img.to_emotion().valence);
    }

    #[test]
    fn motion_increases_arousal() {
        let still  = ImageFeatures { motion: 0.0, hue: 120.0, saturation: 0.5,
                                     brightness: 0.6, face_valence: None };
        let moving = ImageFeatures { motion: 0.9, ..still };
        assert!(moving.to_emotion().arousal > still.to_emotion().arousal,
            "Chuyển động → arousal cao hơn");
    }

    #[test]
    fn dark_low_saturation_sad() {
        let dark = ImageFeatures { hue: 200.0, saturation: 0.1, brightness: 0.2,
                                   motion: 0.0, face_valence: None };
        assert!(dark.to_emotion().valence < 0.0, "Tối + nhạt → buồn");
    }

    // ── BioFeatures ───────────────────────────────────────────────────────────

    #[test]
    fn high_hr_gsr_stressed() {
        let bio = BioFeatures { heart_rate: 110.0, hrv: 0.2, skin_temp: 30.5, gsr: 0.8 };
        let e = bio.to_emotion();
        assert!(e.arousal > 0.5, "HR cao + GSR cao → arousal: {}", e.arousal);
        assert!(e.valence < 0.0, "GSR cao → stress → valence âm: {}", e.valence);
    }

    #[test]
    fn relaxed_bio() {
        let bio = BioFeatures { heart_rate: 65.0, hrv: 0.8, skin_temp: 34.0, gsr: 0.1 };
        let e = bio.to_emotion();
        assert!(e.valence > 0.0, "HRV cao + GSR thấp → thư giãn: {}", e.valence);
    }

    // ── Cross-modal conflict ──────────────────────────────────────────────────

    #[test]
    fn text_says_fine_voice_says_distressed() {
        use crate::fusion::ModalityInput;
        // Text: "tôi bình thường" → V=+0.05
        let text_input = ModalityInput {
            tag:        EmotionTag { valence: 0.05, arousal: 0.4, dominance: 0.6, intensity: 0.1 },
            confidence: 0.7,
            source:     ModalityKind::Text,
        };
        // Audio: giọng run → V=-0.50
        let audio = AudioFeatures { pitch_hz: 180.0, energy: 0.6, tempo_bpm: 120.0,
                                    voice_break: 0.7, brightness: 0.5 };

        let result = fuse(&[text_input, audio.to_modality_input(0.85)]);

        // Conflict: V_text=+0.05 vs V_audio≈-0.47 → chênh lệch > 0.4
        assert!(result.has_conflict, "Conflict khi text vs voice không khớp");
        assert!(result.confidence < 0.7, "Confidence giảm khi conflict: {}", result.confidence);
    }

    #[test]
    fn consistent_sources_high_confidence() {
        let audio = AudioFeatures { pitch_hz: 110.0, energy: 0.3, tempo_bpm: 60.0,
                                    voice_break: 0.0, brightness: 0.3 };
        let image = ImageFeatures { hue: 240.0, saturation: 0.6, brightness: 0.3,
                                    motion: 0.0, face_valence: Some(-0.5) };
        let result = fuse_all(None, Some(audio), Some(image), None);

        // Cả audio và image đều nói "buồn" → consistent → confidence cao
        assert!(!result.has_conflict || result.conflict_level < 0.4,
            "Consistent sources: conflict={}, level={}", result.has_conflict, result.conflict_level);
        assert!(result.tag.valence < 0.0, "Fused valence âm: {}", result.tag.valence);
    }

    #[test]
    fn fuse_all_three_sources() {
        use crate::fusion::ModalityInput;
        let text = ModalityInput {
            tag:        EmotionTag { valence: -0.5, arousal: 0.4, dominance: 0.3, intensity: 0.5 },
            confidence: 0.8,
            source:     ModalityKind::Text,
        };
        let audio = AudioFeatures { pitch_hz: 130.0, energy: 0.3, tempo_bpm: 70.0,
                                    voice_break: 0.2, brightness: 0.4 };
        let bio   = BioFeatures { heart_rate: 90.0, hrv: 0.3, skin_temp: 31.5, gsr: 0.6 };

        let result = fuse_all(Some(text), Some(audio), None, Some(bio));
        assert!(result.tag.valence < 0.0, "Ba sources đồng thuận buồn");
        assert!(result.is_certain(), "Đủ confident: {}", result.confidence);
    }

    #[test]
    fn empty_input_neutral() {
        let result = fuse_all(None, None, None, None);
        assert!(!result.is_certain(), "Không có input → không certain");
    }
}
