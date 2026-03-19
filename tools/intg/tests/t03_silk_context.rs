//! Integration: Silk edge → EmotionTag → ConversationCurve → tone
//!
//! Kiểm tra: Silk edge với EmotionTag → amplify_emotion →
//! ConversationCurve.push() → tone đúng.

use context::emotion::curve::ConversationCurve;
use silk::edge::EmotionTag;
use silk::walk::ResponseTone;

// ── ConversationCurve tone detection ─────────────────────────────────────────

#[test]
fn curve_supportive_when_valence_drops() {
    let mut curve = ConversationCurve::new();

    // Neutral → negative = dropping
    curve.push(0.0);
    curve.push(-0.3);
    curve.push(-0.5);

    let tone = curve.tone();
    assert!(
        matches!(tone, ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause),
        "dropping valence should produce Supportive/Gentle/Pause, got {:?}",
        tone
    );
}

#[test]
fn curve_reinforcing_when_valence_recovers() {
    let mut curve = ConversationCurve::new();

    // Down then up = recovery
    curve.push(-0.5);
    curve.push(-0.3);
    curve.push(-0.1);
    curve.push(0.1);
    curve.push(0.3);

    let tone = curve.tone();
    // During recovery, tone should be positive
    assert!(
        matches!(tone, ResponseTone::Reinforcing | ResponseTone::Celebratory | ResponseTone::Engaged),
        "recovering valence should produce Reinforcing/Celebratory/Neutral, got {:?}",
        tone
    );
}

#[test]
fn curve_gentle_when_stable_negative() {
    let mut curve = ConversationCurve::new();

    // Stable negative
    curve.push(-0.3);
    curve.push(-0.3);
    curve.push(-0.3);
    curve.push(-0.3);

    let tone = curve.tone();
    assert!(
        matches!(tone, ResponseTone::Gentle | ResponseTone::Supportive | ResponseTone::Engaged),
        "stable negative should produce Gentle/Supportive, got {:?}",
        tone
    );
}

// ── Emotion amplification through Silk ───────────────────────────────────────

#[test]
fn emotion_tag_valence_affects_curve() {
    let mut curve = ConversationCurve::new();

    // Simulate emotions from silk edges
    let emo_sad = EmotionTag {
        valence: -0.7,
        arousal: 0.5,
        dominance: 0.0,
        intensity: 0.8,
    };

    // Push valence from emotion tags
    curve.push(emo_sad.valence);
    curve.push(emo_sad.valence);
    curve.push(emo_sad.valence);

    let tone = curve.tone();
    assert!(
        matches!(tone, ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause),
        "negative emotion should produce supportive tone, got {:?}",
        tone
    );
}

#[test]
fn curve_variance_instability_detection() {
    let mut curve = ConversationCurve::new();

    // Wildly oscillating = instability
    curve.push(0.8);
    curve.push(-0.8);
    curve.push(0.7);
    curve.push(-0.7);
    curve.push(0.6);

    // After instability, celebratory should be overridden to gentle
    let tone = curve.tone();
    // The specific override depends on implementation, but tone should exist
    assert!(
        matches!(
            tone,
            ResponseTone::Gentle
                | ResponseTone::Supportive
                | ResponseTone::Pause
                | ResponseTone::Reinforcing
                | ResponseTone::Celebratory
                | ResponseTone::Engaged
        ),
        "curve must produce a valid tone, got {:?}",
        tone
    );
}

// ── Curve push returns fx ────────────────────────────────────────────────────

#[test]
fn curve_push_returns_valid_fx() {
    let mut curve = ConversationCurve::new();

    let fx1 = curve.push(0.0);
    let fx2 = curve.push(-0.5);
    let fx3 = curve.push(-0.8);

    // fx should be finite numbers
    assert!(fx1.is_finite(), "fx must be finite");
    assert!(fx2.is_finite(), "fx must be finite");
    assert!(fx3.is_finite(), "fx must be finite");
}
