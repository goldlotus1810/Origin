//! # engine — ContextEngine
//!
//! Entry point: on_activate(raw_input) → ActivationResult
//!
//! Thứ tự:
//!   1. Parse text → PhraseMatch list
//!   2. Compute EmotionTag (+ cross-modal nếu có audio)
//!   3. Detect intent + modifier
//!   4. Build ContextSnapshot
//!   5. Push vào ConversationCurve → f(x)
//!   6. Trả về ActivationResult

extern crate alloc;
use alloc::vec::Vec;

use silk::edge::EmotionTag;
use silk::walk::ResponseTone;

use crate::emotion::{
    IntentKind, IntentModifier, blend_with_audio, sentence_base_affect,
};
use crate::phrase::PhraseDict;
use crate::curve::ConversationCurve;
use crate::snapshot::{ContextSnapshot, RawInput};

// ─────────────────────────────────────────────────────────────────────────────
// ActivationResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả của on_activate().
#[derive(Debug, Clone)]
pub struct ActivationResult {
    /// f(x) sau khi ingest turn này
    pub fx:       f32,
    /// Tone phản hồi khuyến nghị
    pub tone:     ResponseTone,
    /// EmotionTag của turn này
    pub affect:   EmotionTag,
    /// Intent
    pub intent:   IntentKind,
    /// Turn index
    pub turn:     u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// ContextEngine
// ─────────────────────────────────────────────────────────────────────────────

/// Orchestrator của context.
///
/// Một engine per session (per user).
pub struct ContextEngine {
    /// Phrase parser
    dict:      PhraseDict,
    /// ConversationCurve theo dõi f(x)
    curve:     ConversationCurve,
    /// Tất cả snapshots của session
    snapshots: Vec<ContextSnapshot>,
    /// Session ID
    session_id: u64,
}

impl ContextEngine {
    /// Tạo engine mới cho session.
    pub fn new(session_id: u64) -> Self {
        Self {
            dict:       PhraseDict::new(),
            curve:      ConversationCurve::new(),
            snapshots:  Vec::new(),
            session_id,
        }
    }

    /// Entry point — gọi khi có bất kỳ input nào.
    ///
    /// Thứ tự: parse → affect → intent → snapshot → curve → result
    pub fn on_activate(&mut self, raw: RawInput) -> ActivationResult {
        let turn_index = self.snapshots.len() as u32;

        // ── 1. Parse phrases ─────────────────────────────────────────────────
        let phrase_matches = if let Some(ref text) = raw.text {
            self.dict.parse(text)
        } else {
            Vec::new()
        };

        // phrase_hashes
        let phrase_hashes: Vec<u64> = phrase_matches.iter()
            .filter_map(|m| m.chain_hash)
            .collect();

        // ── 2. Compute EmotionTag ────────────────────────────────────────────
        // Base affect từ phrases
        let base_affect = if !phrase_matches.is_empty() {
            PhraseDict::aggregate_emotion(&phrase_matches)
        } else if let Some(ref text) = raw.text {
            let words: Vec<&str> = text.split_whitespace().collect();
            let word_refs: Vec<&str> = words.iter().map(|s| &s[..]).collect();
            sentence_base_affect(&word_refs)
        } else {
            EmotionTag::NEUTRAL
        };

        // Cross-modal fusion nếu có audio
        let affect = if let (Some(pitch), Some(energy)) = (raw.audio_pitch, raw.audio_energy) {
            // Audio valence từ pitch và energy
            let audio_valence = if pitch < 150.0 { -0.4 } else { energy * 0.3 - 0.1 };
            blend_with_audio(base_affect, audio_valence, energy, pitch)
        } else {
            base_affect
        };

        // ── 3. Detect intent + modifier ──────────────────────────────────────
        let text_str = raw.text.as_ref().map(|s| s.as_ref()).unwrap_or("");
        let intent   = IntentKind::detect(text_str);
        let modifier = IntentModifier::detect(text_str);
        let affect   = modifier.apply(affect);

        // ── 4. Build ContextSnapshot ─────────────────────────────────────────
        let snap = ContextSnapshot {
            turn_index,
            raw_text:     raw.text.clone(),
            phrase_hashes,
            affect,
            intent,
            modifier,
            modality:     raw.modality,
            audio_pitch:  raw.audio_pitch,
            audio_energy: raw.audio_energy,
            timestamp:    raw.timestamp,
            session_id:   self.session_id,
        };

        // ── 5. Push vào ConversationCurve ────────────────────────────────────
        let fx = self.curve.push(affect.valence);

        // ── 6. Lưu snapshot ──────────────────────────────────────────────────
        self.snapshots.push(snap);

        // ── 7. Trả về kết quả ───────────────────────────────────────────────
        ActivationResult {
            fx,
            tone:   self.curve.tone(),
            affect,
            intent,
            turn:   turn_index,
        }
    }

    /// f(x) hiện tại.
    pub fn fx(&self) -> f32 { self.curve.fx() }

    /// Số turns.
    pub fn turn_count(&self) -> usize { self.snapshots.len() }

    /// Tone hiện tại.
    pub fn tone(&self) -> ResponseTone { self.curve.tone() }

    /// Valence hiện tại.
    pub fn current_v(&self) -> f32 { self.curve.current_v() }

    /// EmotionTag từ ConversationCurve hiện tại.
    /// Dùng để feed vào Silk khi VM events xảy ra.
    pub fn last_emotion(&self) -> EmotionTag {
        let v = self.curve.current_v();
        let d1 = self.curve.d1_now();
        // valence → emotion byte, arousal từ d1 (momentum)
        let _v_byte = ((v + 1.0) * 127.5) as u8;
        let a_byte = (d1.abs() * 255.0).min(255.0) as u8;
        EmotionTag::new(v, a_byte as f32 / 255.0, 0.5, 0.5)
    }

    /// ConversationCurve (read-only).
    pub fn curve(&self) -> &ConversationCurve { &self.curve }

    /// Snapshots (read-only).
    pub fn snapshots(&self) -> &[ContextSnapshot] { &self.snapshots }

    /// Cập nhật f_dn từ ĐN node mới (gọi khi Memory tạo node).
    pub fn update_dn(&mut self, dn_valence: f32) {
        self.curve.update_dn(dn_valence);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ContextEngine { ContextEngine::new(0xDEAD_BEEF) }

    #[test]
    fn activate_single_turn() {
        let mut e = engine();
        let r = e.on_activate(RawInput::text("hôm nay trời đẹp", 1000));
        assert_eq!(r.turn, 0);
        assert_eq!(e.turn_count(), 1);
    }

    #[test]
    fn activate_crisis_detected() {
        let mut e = engine();
        let r = e.on_activate(RawInput::text("tôi muốn chết", 1000));
        assert_eq!(r.intent, IntentKind::Crisis,
            "Crisis phải detect được");
    }

    #[test]
    fn activate_command_detected() {
        let mut e = engine();
        let r = e.on_activate(RawInput::text("tắt đèn phòng khách", 1000));
        assert_eq!(r.intent, IntentKind::Command);
    }

    #[test]
    fn activate_sad_text_negative_affect() {
        let mut e = engine();
        let r = e.on_activate(RawInput::text("tôi buồn quá hôm nay", 1000));
        assert!(r.affect.valence < 0.0,
            "Câu buồn → affect âm: {}", r.affect.valence);
    }

    #[test]
    fn activate_three_turns_accumulate() {
        let mut e = engine();
        e.on_activate(RawInput::text("ok", 1000));
        e.on_activate(RawInput::text("tôi mệt rồi", 2000));
        e.on_activate(RawInput::text("buồn quá", 3000));

        assert_eq!(e.turn_count(), 3);
        assert!(e.current_v() < 0.0, "Sau 3 turns buồn → V âm");
    }

    #[test]
    fn activate_falling_curve_supportive() {
        let mut e = engine();
        e.on_activate(RawInput::text("ok binh thuong", 1000));
        e.on_activate(RawInput::text("tôi mệt rồi", 2000));
        let r = e.on_activate(RawInput::text("buồn quá khó chịu", 3000));

        // Curve đang giảm → Supportive
        assert!(
            matches!(r.tone, ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause),
            "Curve giảm → Supportive/Gentle/Pause, got {:?}", r.tone
        );
    }

    #[test]
    fn activate_audio_override_text() {
        let mut e = engine();
        // Text vui nhưng giọng run thấp
        let raw = RawInput::text_with_audio("bình thường thôi", 120.0, 0.2, 1000);
        let r = e.on_activate(raw);
        // Pitch 120Hz rất thấp → audio override → V âm
        assert!(r.affect.valence < 0.1,
            "Pitch thấp → audio override: val={}", r.affect.valence);
    }

    #[test]
    fn activate_fx_uses_alpha_beta() {
        let mut e = engine();
        e.on_activate(RawInput::text("ok", 1000));
        let fx = e.fx();
        // f(x) = 0.6 × f_conv + 0.4 × f_dn (f_dn = 0 ban đầu)
        // → f(x) ≈ 0.6 × f_conv
        let f_conv = e.curve().fx_conv;
        let expected = 0.6 * f_conv;
        assert!((fx - expected).abs() < 0.01,
            "f(x) = 0.6×f_conv: {} ≈ {}", fx, expected);
    }

    #[test]
    fn update_dn_affects_fx() {
        let mut e = engine();
        e.on_activate(RawInput::text("ok", 1000));
        let fx_before = e.fx();
        e.update_dn(-0.8); // ĐN âm
        let fx_after = e.fx();
        assert!(fx_after < fx_before,
            "ĐN âm → f(x) giảm: {} < {}", fx_after, fx_before);
    }

    #[test]
    fn session_id_preserved() {
        let e = ContextEngine::new(0x1234_5678);
        assert_eq!(e.session_id, 0x1234_5678);
    }

    #[test]
    fn snapshots_grow_append_only() {
        let mut e = engine();
        for i in 0u32..5 {
            e.on_activate(RawInput::text("ok", i as i64 * 1000));
        }
        assert_eq!(e.snapshots().len(), 5);
        // Verify turn_index tăng dần
        for (i, snap) in e.snapshots().iter().enumerate() {
            assert_eq!(snap.turn_index, i as u32);
        }
    }
}
