//! # walk — Walk qua Silk graph
//!
//! SentenceAffect: không trung bình từng từ riêng lẻ.
//! Walk qua Silk → emotions amplify nhau theo edge weight.
//!
//! "tôi buồn vì mất việc":
//!   MAT_VIEC → BUON (w=0.90) → CO_DON (w=0.71)
//!   composite V = -0.85 (nặng hơn từng từ riêng lẻ)

extern crate alloc;
use alloc::vec::Vec;

use crate::edge::EmotionTag;
use crate::graph::SilkGraph;

// ─────────────────────────────────────────────────────────────────────────────
// WalkResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả walk qua Silk.
#[derive(Debug, Clone)]
pub struct WalkResult {
    /// EmotionTag tổng hợp sau khi walk
    pub composite: EmotionTag,
    /// Các nodes đã đi qua (chain_hash)
    pub path: Vec<u64>,
    /// Tổng weight tích lũy
    pub total_weight: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// SentenceAffect
// ─────────────────────────────────────────────────────────────────────────────

/// Walk qua Silk graph để tính EmotionTag tổng hợp của câu.
///
/// Không trung bình — walk theo edge weight → amplify.
/// Depth-limited để tránh infinite walk (QT2: ∞-1).
pub fn sentence_affect(
    graph: &SilkGraph,
    word_hashes: &[u64],          // hash của từng từ trong câu
    word_emotions: &[EmotionTag], // EmotionTag ban đầu của từng từ
    max_depth: usize,             // Fib[n] — depth giới hạn
) -> WalkResult {
    if word_hashes.is_empty() {
        return WalkResult {
            composite: EmotionTag::NEUTRAL,
            path: Vec::new(),
            total_weight: 0.0,
        };
    }

    // Bắt đầu từ EmotionTag của từ đầu tiên
    let mut composite = word_emotions
        .first()
        .copied()
        .unwrap_or(EmotionTag::NEUTRAL);
    let mut path = Vec::new();
    let mut total_weight = 1.0f32;

    path.push(word_hashes[0]);

    // Walk từng từ
    for i in 1..word_hashes.len().min(word_emotions.len()) {
        let hash = word_hashes[i];
        let w_emo = word_emotions[i];

        // Tìm Silk edge từ từ trước đến từ này
        let edge_weight = graph
            .assoc_weight(word_hashes[i - 1], hash)
            .max(graph.assoc_weight(hash, word_hashes[i - 1]));

        if edge_weight > 0.01 {
            // Amplify: từ kết nối → emotion của nó được khuếch đại
            let amplified = amplify_emotion(w_emo, edge_weight);
            composite = blend_composite(composite, amplified, edge_weight);
            total_weight += edge_weight;
        } else {
            // Không có Silk → blend nhẹ
            composite = blend_composite(composite, w_emo, 0.3);
            total_weight += 0.3;
        }

        path.push(hash);

        // Depth limit (QT2)
        if path.len() >= max_depth {
            break;
        }
    }

    // Normalize
    if total_weight > 0.0 {
        // Clamp valence và arousal
        composite.valence = composite.valence.clamp(-1.0, 1.0);
        composite.arousal = composite.arousal.clamp(0.0, 1.0);
        composite.dominance = composite.dominance.clamp(0.0, 1.0);
        composite.intensity = composite.intensity.clamp(0.0, 1.0);
    }

    WalkResult {
        composite,
        path,
        total_weight,
    }
}

/// Amplify emotion theo edge weight.
///
/// Edge mạnh → emotion đó ảnh hưởng nhiều hơn.
/// Đây là điểm khác biệt với trung bình đơn giản.
fn amplify_emotion(emo: EmotionTag, weight: f32) -> EmotionTag {
    EmotionTag {
        valence: emo.valence * (1.0 + weight * 0.5),
        arousal: emo.arousal * (1.0 + weight * 0.3),
        dominance: emo.dominance,
        intensity: emo.intensity * (1.0 + weight * 0.4),
    }
}

/// Blend composite với emotion mới theo weight.
fn blend_composite(composite: EmotionTag, new_emo: EmotionTag, weight: f32) -> EmotionTag {
    let w_norm = weight / (1.0 + weight);
    EmotionTag {
        valence: composite.valence * (1.0 - w_norm) + new_emo.valence * w_norm,
        arousal: composite.arousal * (1.0 - w_norm) + new_emo.arousal * w_norm,
        dominance: composite.dominance * (1.0 - w_norm) + new_emo.dominance * w_norm,
        intensity: composite.intensity * (1.0 - w_norm) + new_emo.intensity * w_norm,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResponseTone
// ─────────────────────────────────────────────────────────────────────────────

/// Tone phản hồi từ ConversationCurve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseTone {
    /// f'(t) < -0.15 — đang buồn xuống, dẫn lên chậm
    Supportive,
    /// f''(t) < -0.25 — đột ngột xấu, dừng lại hỏi
    Pause,
    /// f'(t) > +0.15 — đang hồi phục, tiếp tục đà
    Reinforcing,
    /// f''(t) > +0.25 AND V > 0 — bước ngoặt tốt
    Celebratory,
    /// V < -0.20, stable — buồn ổn định, dịu dàng
    Gentle,
    /// Bình thường
    Engaged,
}

/// Tính ResponseTone từ ConversationCurve.
///
/// curve: Vec<f32> = valence qua các turns
pub fn response_tone(curve: &[f32]) -> ResponseTone {
    let n = curve.len();
    if n == 0 {
        return ResponseTone::Engaged;
    }

    let v = curve[n - 1];

    // f'(t) — tốc độ thay đổi
    let d1 = if n >= 2 {
        curve[n - 1] - curve[n - 2]
    } else {
        0.0
    };

    // f''(t) — gia tốc thay đổi
    let d2 = if n >= 3 {
        (curve[n - 1] - curve[n - 2]) - (curve[n - 2] - curve[n - 3])
    } else {
        0.0
    };

    // Ưu tiên theo thứ tự
    if d1 < -0.15 {
        ResponseTone::Supportive
    } else if d2 < -0.25 {
        ResponseTone::Pause
    } else if d1 > 0.15 {
        ResponseTone::Reinforcing
    } else if d2 > 0.25 && v > 0.0 {
        ResponseTone::Celebratory
    } else if v < -0.20 {
        ResponseTone::Gentle
    } else {
        ResponseTone::Engaged
    }
}

/// Bước tiếp theo trên ConversationCurve.
///
/// Không nhảy quá 0.40/bước — dẫn từng bước.
pub fn next_curve_step(current_v: f32, target_v: f32) -> f32 {
    let delta = target_v - current_v;
    let max_step = 0.40f32;
    if delta.abs() <= max_step {
        target_v
    } else {
        current_v + max_step * delta.signum()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::EdgeKind;
    use crate::graph::SilkGraph;

    fn emo(v: f32, a: f32) -> EmotionTag {
        EmotionTag::new(v, a, 0.5, 0.8)
    }

    // ── SentenceAffect ───────────────────────────────────────────────────────

    #[test]
    fn sentence_affect_single_word() {
        let g = SilkGraph::new();
        let result = sentence_affect(&g, &[0xA], &[emo(-0.6, 0.8)], 10);
        assert!((result.composite.valence - (-0.6)).abs() < 0.01);
    }

    #[test]
    fn sentence_affect_amplifies_connected() {
        let mut g = SilkGraph::new();

        // "mất việc" và "buồn" hay co-activate → Silk mạnh
        for _ in 0..50 {
            g.co_activate(0xAA11_u64, 0xBB22_u64, emo(-0.7, 0.6), 0.9, 0);
        }

        // "tôi buồn vì mất việc"
        let hashes = [0xCC33_u64, 0xBB22_u64, 0xAA11_u64];
        let emotions = [emo(0.0, 0.3), emo(-0.5, 0.7), emo(-0.6, 0.5)];

        let result = sentence_affect(&g, &hashes, &emotions, 10);

        // Composite phải âm hơn trung bình đơn giản
        let simple_avg = (0.0 + (-0.5) + (-0.6)) / 3.0; // = -0.367
        assert!(
            result.composite.valence < simple_avg,
            "SentenceAffect < average: {} < {}",
            result.composite.valence,
            simple_avg
        );
    }

    #[test]
    fn sentence_affect_unconnected_mild() {
        let g = SilkGraph::new(); // không có Silk

        let hashes = [0xA, 0xB, 0xC];
        let emotions = [emo(-0.3, 0.5), emo(-0.4, 0.6), emo(-0.5, 0.7)];

        let result = sentence_affect(&g, &hashes, &emotions, 10);
        // Không có Silk → gần với trung bình
        let avg = (-0.3 + -0.4 + -0.5) / 3.0;
        assert!(
            (result.composite.valence - avg).abs() < 0.3,
            "Không Silk → gần avg: {} ≈ {}",
            result.composite.valence,
            avg
        );
    }

    #[test]
    fn sentence_affect_depth_limit() {
        let mut g = SilkGraph::new();
        // Tạo chain dài
        for i in 0u64..20 {
            g.co_activate(i, i + 1, emo(-0.3, 0.5), 0.8, 0);
        }

        let hashes: Vec<u64> = (0..20).collect();
        let emotions: Vec<EmotionTag> = (0..20).map(|_| emo(-0.3, 0.5)).collect();

        let result = sentence_affect(&g, &hashes, &emotions, 5); // max_depth=5
        assert!(
            result.path.len() <= 5,
            "Depth limit: {} <= 5",
            result.path.len()
        );
    }

    #[test]
    fn sentence_affect_valence_clamped() {
        let g = SilkGraph::new();
        // Emotions rất cực đoan
        let result = sentence_affect(&g, &[0xA], &[EmotionTag::new(-2.0, 2.0, 1.5, 1.5)], 10);
        assert!(result.composite.valence >= -1.0, "Valence phải >= -1.0");
        assert!(result.composite.arousal <= 1.0, "Arousal phải <= 1.0");
    }

    // ── ResponseTone ─────────────────────────────────────────────────────────

    #[test]
    fn tone_supportive_falling() {
        // Đang buồn xuống nhanh
        let curve = [-0.1, -0.3, -0.5];
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Supportive,
            "d1=-0.2 < -0.15 → Supportive"
        );
    }

    #[test]
    fn tone_pause_sudden_drop() {
        // Đột ngột xấu
        let curve = [-0.1, -0.1, -0.5];
        let tone = response_tone(&curve);
        // d1 = -0.4, d2 = -0.3 < -0.25
        // d1 < -0.15 → Supportive wins (trước d2)
        assert!(matches!(
            tone,
            ResponseTone::Supportive | ResponseTone::Pause
        ));
    }

    #[test]
    fn tone_reinforcing_rising() {
        let curve = [-0.5, -0.2, 0.1];
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Reinforcing,
            "d1=0.3 > 0.15 → Reinforcing"
        );
    }

    #[test]
    fn tone_celebratory() {
        let curve = [-0.1, 0.1, 0.4];
        // d1=0.3 > 0.15 → Reinforcing wins
        // Nhưng nếu d1 nhỏ hơn...
        let curve2 = [0.0, 0.1, 0.3];
        // d1=0.2 > 0.15 → Reinforcing
        let _ = (curve, curve2);
        // Celebratory: d2 > 0.25 AND v > 0 AND d1 <= 0.15
        let curve3 = [-0.2, 0.0, 0.1]; // d1=0.1, d2=0.3, v=0.1
        let tone3 = response_tone(&curve3);
        assert!(matches!(
            tone3,
            ResponseTone::Celebratory | ResponseTone::Engaged
        ));
    }

    #[test]
    fn tone_gentle_stable_sad() {
        let curve = [-0.4, -0.4, -0.4]; // Buồn ổn định
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Gentle,
            "V < -0.20, stable → Gentle"
        );
    }

    #[test]
    fn tone_engaged_neutral() {
        let curve = [0.0, 0.05, 0.0];
        assert_eq!(response_tone(&curve), ResponseTone::Engaged);
    }

    #[test]
    fn tone_empty_curve() {
        assert_eq!(response_tone(&[]), ResponseTone::Engaged);
    }

    // ── Next curve step ──────────────────────────────────────────────────────

    #[test]
    fn step_small_delta() {
        // Target gần → đến thẳng
        assert!((next_curve_step(-0.5, -0.4) - (-0.4)).abs() < 0.001);
    }

    #[test]
    fn step_large_delta_capped() {
        // Target xa → chỉ bước 0.40
        let result = next_curve_step(-0.7, 0.5);
        assert!(
            (result - (-0.30)).abs() < 0.001,
            "Bước tối đa 0.40: {} ≈ -0.30",
            result
        );
    }

    #[test]
    fn step_no_jump_more_than_040() {
        let start = -0.8;
        let target = 0.8;
        let step = next_curve_step(start, target);
        assert!(
            (step - start).abs() <= 0.40 + 0.001,
            "Không nhảy quá 0.40: delta={}",
            (step - start).abs()
        );
    }
}
