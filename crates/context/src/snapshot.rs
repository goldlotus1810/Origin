//! # snapshot — ContextSnapshot
//!
//! Cái nhìn tức thời khi kích hoạt (chat/lệnh/audio/image).
//! Tạo NGAY khi có input — không chờ Dream.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use silk::edge::EmotionTag;
use crate::emotion::{IntentKind, IntentModifier};

// ─────────────────────────────────────────────────────────────────────────────
// ModalitySource — nguồn input
// ─────────────────────────────────────────────────────────────────────────────

/// Nguồn input — từ đâu đến.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalitySource {
    Text,
    Audio,
    Image,
    Sensor,
    Code,
    Math,
    System,
    Internal,
}

// ─────────────────────────────────────────────────────────────────────────────
// RawInput
// ─────────────────────────────────────────────────────────────────────────────

/// Input thô từ người dùng hoặc thiết bị.
#[derive(Debug, Clone)]
pub struct RawInput {
    pub text:         Option<String>,
    pub modality:     ModalitySource,
    pub audio_pitch:  Option<f32>,   // Hz
    pub audio_energy: Option<f32>,   // [0,1]
    pub image_affect: Option<EmotionTag>,
    pub sensor_value: Option<f32>,
    pub timestamp:    i64,
}

impl RawInput {
    pub fn text(s: &str, ts: i64) -> Self {
        Self {
            text:         Some(String::from(s)),
            modality:     ModalitySource::Text,
            audio_pitch:  None,
            audio_energy: None,
            image_affect: None,
            sensor_value: None,
            timestamp:    ts,
        }
    }

    pub fn audio(pitch: f32, energy: f32, ts: i64) -> Self {
        Self {
            text:         None,
            modality:     ModalitySource::Audio,
            audio_pitch:  Some(pitch),
            audio_energy: Some(energy),
            image_affect: None,
            sensor_value: None,
            timestamp:    ts,
        }
    }

    pub fn text_with_audio(s: &str, pitch: f32, energy: f32, ts: i64) -> Self {
        Self {
            text:         Some(String::from(s)),
            modality:     ModalitySource::Audio,
            audio_pitch:  Some(pitch),
            audio_energy: Some(energy),
            image_affect: None,
            sensor_value: None,
            timestamp:    ts,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ContextSnapshot
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot của context tại một thời điểm.
///
/// Tạo ngay khi on_activate() được gọi.
/// Lưu vào ConversationContext để track history.
#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    /// Turn index trong session
    pub turn_index:   u32,
    /// Text thô (nếu có)
    pub raw_text:     Option<String>,
    /// phrase_hashes — chain hashes của các phrases
    pub phrase_hashes: Vec<u64>,
    /// EmotionTag tổng hợp
    pub affect:       EmotionTag,
    /// Intent
    pub intent:       IntentKind,
    /// Modifier
    pub modifier:     IntentModifier,
    /// Nguồn input
    pub modality:     ModalitySource,
    /// Audio pitch nếu có
    pub audio_pitch:  Option<f32>,
    /// Audio energy nếu có
    pub audio_energy: Option<f32>,
    /// Timestamp (ns)
    pub timestamp:    i64,
    /// Session ID
    pub session_id:   u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_input_text() {
        let r = RawInput::text("hom nay troi dep", 1000);
        assert_eq!(r.modality, ModalitySource::Text);
        assert!(r.text.is_some());
        assert!(r.audio_pitch.is_none());
    }

    #[test]
    fn raw_input_audio() {
        let r = RawInput::audio(180.0, 0.3, 2000);
        assert_eq!(r.modality, ModalitySource::Audio);
        assert_eq!(r.audio_pitch, Some(180.0));
    }

    #[test]
    fn raw_input_text_with_audio() {
        let r = RawInput::text_with_audio("toi met qua", 150.0, 0.2, 3000);
        assert!(r.text.is_some());
        assert_eq!(r.audio_pitch, Some(150.0));
        assert_eq!(r.modality, ModalitySource::Audio);
    }
}
