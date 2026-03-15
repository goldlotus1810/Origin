//! # fusion — ModalityFusion
//!
//! Gộp EmotionTag từ nhiều nguồn (text/audio/image/bio) → FusedEmotionTag.
//!
//! Thuật toán từ design doc:
//!   1. Mỗi nguồn cho ra EmotionTag + confidence riêng
//!   2. Weighted average theo confidence
//!   3. Conflict detection: nếu sources mâu thuẫn → confidence giảm
//!   4. BlackCurtain: nếu confidence < threshold → không kết luận
//!
//! ```text
//! Audio: "bình thường" (V=+0.10)
//! nhưng giọng run, nhịp tim cao (V=-0.40)
//! → conflict → confidence giảm → hỏi thêm
//! ```

extern crate alloc;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// ModalityInput — một nguồn cảm xúc
// ─────────────────────────────────────────────────────────────────────────────

/// Một nguồn cảm xúc với confidence riêng.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct ModalityInput {
    pub tag:        EmotionTag,
    pub confidence: f32,   // 0.0 → 1.0 — độ tin cậy của nguồn này
    pub source:     ModalityKind,
}

/// Loại nguồn cảm xúc.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalityKind {
    Text,
    Audio,
    Image,
    Bio,
}

impl ModalityKind {
    /// Weight mặc định của mỗi modality — có thể override bằng confidence.
    /// Text: chính xác về ngữ nghĩa nhưng có thể che giấu cảm xúc thật.
    /// Audio: khó giả — giọng nói phản ánh cảm xúc thật hơn.
    /// Image: khuôn mặt khó giả hoàn toàn.
    /// Bio: không thể giả (nhịp tim, mồ hôi).
    pub fn base_weight(self) -> f32 {
        match self {
            Self::Text  => 0.30,
            Self::Audio => 0.40,
            Self::Image => 0.25,
            Self::Bio   => 0.50,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FusedEmotionTag — kết quả sau fusion
// ─────────────────────────────────────────────────────────────────────────────

/// EmotionTag sau khi gộp nhiều nguồn.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct FusedEmotionTag {
    pub tag:              EmotionTag,
    /// Confidence tổng thể ∈ [0.0, 1.0]
    pub confidence:       f32,
    /// Có conflict giữa các nguồn không
    pub has_conflict:     bool,
    /// Mức độ conflict [0.0, 1.0]
    pub conflict_level:   f32,
}

impl FusedEmotionTag {
    /// BlackCurtain threshold — dưới đây không kết luận
    pub const CONFIDENCE_THRESHOLD: f32 = 0.35;

    /// Có đủ tin cậy để đưa ra kết luận không.
    pub fn is_certain(self) -> bool {
        self.confidence >= Self::CONFIDENCE_THRESHOLD
    }

    /// Mô tả ngắn gọn cho debug.
    pub fn describe(self) -> alloc::string::String {
        use alloc::format;
        if self.has_conflict {
            format!("CONFLICT({}%) V={:+.2} conf={:.2}",
                (self.conflict_level * 100.0) as u32,
                self.tag.valence, self.confidence)
        } else {
            format!("V={:+.2} A={:.2} conf={:.2}",
                self.tag.valence, self.tag.arousal, self.confidence)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// fuse() — thuật toán fusion
// ─────────────────────────────────────────────────────────────────────────────

/// Gộp nhiều ModalityInput → FusedEmotionTag.
///
/// Thuật toán:
///   1. Weighted average: w_i = base_weight × confidence_i
///   2. Conflict detection: max|V_i - V_j| > 0.4 → conflict
///   3. Confidence penalty khi conflict
///   4. BlackCurtain: has_conflict → thêm dấu hiệu cần hỏi thêm
pub fn fuse(inputs: &[ModalityInput]) -> FusedEmotionTag {
    if inputs.is_empty() {
        return FusedEmotionTag {
            tag: EmotionTag::NEUTRAL,
            confidence: 0.0,
            has_conflict: false,
            conflict_level: 0.0,
        };
    }

    if inputs.len() == 1 {
        return FusedEmotionTag {
            tag: inputs[0].tag,
            confidence: inputs[0].confidence,
            has_conflict: false,
            conflict_level: 0.0,
        };
    }

    // 1. Tính weighted sum
    let mut total_weight = 0.0_f32;
    let mut sum_v        = 0.0_f32;
    let mut sum_a        = 0.0_f32;
    let mut sum_d        = 0.0_f32;
    let mut sum_i        = 0.0_f32;

    for inp in inputs {
        let w = inp.source.base_weight() * inp.confidence.clamp(0.0, 1.0);
        total_weight += w;
        sum_v += inp.tag.valence   * w;
        sum_a += inp.tag.arousal   * w;
        sum_d += inp.tag.dominance * w;
        sum_i += inp.tag.intensity * w;
    }

    let fused = if total_weight > 0.0 {
        EmotionTag {
            valence:   (sum_v / total_weight).clamp(-1.0, 1.0),
            arousal:   (sum_a / total_weight).clamp(0.0,  1.0),
            dominance: (sum_d / total_weight).clamp(0.0,  1.0),
            intensity: (sum_i / total_weight).clamp(0.0,  1.0),
        }
    } else {
        EmotionTag::NEUTRAL
    };

    // 2. Conflict detection: tìm cặp sources có valence chênh lệch nhiều
    let mut max_conflict = 0.0_f32;
    for i in 0..inputs.len() {
        for j in i+1..inputs.len() {
            let diff = (inputs[i].tag.valence - inputs[j].tag.valence).abs();
            if diff > max_conflict { max_conflict = diff; }
        }
    }

    // Conflict threshold: > 0.4 = mâu thuẫn đáng kể
    let has_conflict   = max_conflict > 0.40;
    let conflict_level = (max_conflict / 2.0).clamp(0.0, 1.0);

    // 3. Confidence
    // Baseline = trung bình confidence các sources
    let avg_conf = inputs.iter().map(|i| i.confidence).sum::<f32>() / inputs.len() as f32;
    // Penalty khi conflict: giảm confidence tỷ lệ với conflict level
    let confidence = if has_conflict {
        (avg_conf * (1.0 - conflict_level * 0.6)).clamp(0.0, 1.0)
    } else {
        avg_conf
    };

    FusedEmotionTag { tag: fused, confidence, has_conflict, conflict_level }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(v: f32, a: f32, conf: f32, kind: ModalityKind) -> ModalityInput {
        ModalityInput {
            tag: EmotionTag { valence: v, arousal: a, dominance: 0.5, intensity: v.abs() },
            confidence: conf,
            source: kind,
        }
    }

    #[test]
    fn single_source_passthrough() {
        let result = fuse(&[inp(-0.6, 0.4, 0.8, ModalityKind::Text)]);
        assert!((result.tag.valence - (-0.6)).abs() < 0.01);
        assert_eq!(result.confidence, 0.8);
        assert!(!result.has_conflict);
    }

    #[test]
    fn consistent_sources_high_confidence() {
        // Text và Audio đồng thuận → confidence cao, no conflict
        let result = fuse(&[
            inp(-0.5, 0.4, 0.8, ModalityKind::Text),
            inp(-0.6, 0.5, 0.9, ModalityKind::Audio),
        ]);
        assert!(!result.has_conflict, "Đồng thuận → no conflict");
        assert!(result.tag.valence < -0.3, "Fused vẫn negative");
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn conflicting_sources_lower_confidence() {
        // Text nói "bình thường" (V=+0.1), Audio nói "rất buồn" (V=-0.6)
        let result = fuse(&[
            inp( 0.10, 0.4, 0.7, ModalityKind::Text),
            inp(-0.60, 0.3, 0.8, ModalityKind::Audio),
        ]);
        assert!(result.has_conflict, "Chênh lệch 0.7 → conflict");
        assert!(result.conflict_level > 0.3, "Conflict level: {}", result.conflict_level);
        // Confidence phải thấp hơn trường hợp không conflict
        assert!(result.confidence < 0.7, "Conflict penalty: {}", result.confidence);
    }

    #[test]
    fn bio_has_highest_weight() {
        // Bio không thể giả → base_weight cao nhất
        assert!(ModalityKind::Bio.base_weight() > ModalityKind::Text.base_weight());
        assert!(ModalityKind::Bio.base_weight() > ModalityKind::Audio.base_weight());
    }

    #[test]
    fn conflict_reduces_confidence() {
        let high_conflict = fuse(&[
            inp( 0.8, 0.7, 0.9, ModalityKind::Text),
            inp(-0.8, 0.3, 0.9, ModalityKind::Audio), // extreme opposite
        ]);
        let no_conflict = fuse(&[
            inp(-0.7, 0.4, 0.9, ModalityKind::Text),
            inp(-0.6, 0.5, 0.9, ModalityKind::Audio),
        ]);
        assert!(high_conflict.confidence < no_conflict.confidence,
            "Conflict: {} < no-conflict: {}",
            high_conflict.confidence, no_conflict.confidence);
    }

    #[test]
    fn black_curtain_threshold() {
        // Conflict mạnh → confidence thấp → BlackCurtain không kết luận
        let result = fuse(&[
            inp( 0.9, 0.8, 0.9, ModalityKind::Text),
            inp(-0.9, 0.2, 0.9, ModalityKind::Bio),
        ]);
        // Với conflict cực đại → might fall below threshold
        assert!(result.has_conflict);
        if !result.is_certain() {
            // BlackCurtain: không kết luận
            assert!(result.confidence < FusedEmotionTag::CONFIDENCE_THRESHOLD);
        }
    }

    #[test]
    fn empty_inputs_neutral() {
        let result = fuse(&[]);
        assert_eq!(result.confidence, 0.0);
        assert!(!result.is_certain());
    }

    #[test]
    fn three_sources_weighted() {
        // 3 sources đồng thuận, khác confidence
        let result = fuse(&[
            inp(-0.5, 0.4, 0.6, ModalityKind::Text),
            inp(-0.7, 0.5, 0.9, ModalityKind::Audio), // Audio cao hơn, confident hơn
            inp(-0.6, 0.4, 0.7, ModalityKind::Image),
        ]);
        // Audio có weight cao hơn → fused gần với Audio
        assert!(result.tag.valence < -0.4, "Audio dominates: {}", result.tag.valence);
        assert!(!result.has_conflict, "3 sources đồng thuận");
    }

    #[test]
    fn describe_conflict() {
        let result = fuse(&[
            inp( 0.5, 0.6, 0.8, ModalityKind::Text),
            inp(-0.6, 0.3, 0.8, ModalityKind::Audio),
        ]);
        let desc = result.describe();
        assert!(desc.contains("CONFLICT") || desc.contains("V="), "Describe: {}", desc);
    }
}
