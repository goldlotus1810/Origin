//! # word_guide — Dẫn dần cảm xúc qua từ ngữ
//!
//! Port từ Go: word_affect.go + emotion_query.go
//!
//! Thuật toán:
//!   TargetAffect:   ConversationCurve → target EmotionTag (dẫn dần, không nhảy)
//!   SelectWords:    target EmotionTag → N từ gần nhất (Euclidean VAD, V×2)
//!   AffectSentence: tone + words → gợi ý mở đầu (không hardcode strings)
//!   EmotionHistory: append-only edges + profile query

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;

use silk::edge::EmotionTag;
use crate::curve::ConversationCurve;
use silk::walk::ResponseTone;
use crate::context::EmotionContext;

// ─────────────────────────────────────────────────────────────────────────────
// CoreLexicon — mỗi từ có EmotionTag riêng
// ─────────────────────────────────────────────────────────────────────────────

/// Một từ với EmotionTag.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct WordEntry {
    pub word:    &'static str,
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
}

/// Core lexicon — (word, V, A, D).
/// Từ ngữ của HomeOS — không hardcode trong logic response.
static CORE_LEXICON: &[WordEntry] = &[
    // Tích cực nhẹ
    WordEntry { word: "bình yên",    valence:  0.50, arousal: 0.20, dominance: 0.60 },
    WordEntry { word: "nhẹ nhàng",   valence:  0.45, arousal: 0.20, dominance: 0.55 },
    WordEntry { word: "an toàn",     valence:  0.55, arousal: 0.20, dominance: 0.65 },
    WordEntry { word: "ổn",          valence:  0.30, arousal: 0.30, dominance: 0.55 },
    WordEntry { word: "thoải mái",   valence:  0.50, arousal: 0.25, dominance: 0.60 },
    WordEntry { word: "ấm áp",       valence:  0.60, arousal: 0.30, dominance: 0.60 },
    WordEntry { word: "rõ ràng",     valence:  0.20, arousal: 0.35, dominance: 0.60 },
    WordEntry { word: "hay",         valence:  0.50, arousal: 0.45, dominance: 0.60 },
    WordEntry { word: "đúng",        valence:  0.30, arousal: 0.35, dominance: 0.65 },
    // Tích cực mạnh
    WordEntry { word: "vui",         valence:  0.70, arousal: 0.60, dominance: 0.70 },
    WordEntry { word: "hạnh phúc",   valence:  0.80, arousal: 0.60, dominance: 0.70 },
    WordEntry { word: "tuyệt vời",   valence:  0.85, arousal: 0.70, dominance: 0.75 },
    WordEntry { word: "thú vị",      valence:  0.65, arousal: 0.65, dominance: 0.65 },
    WordEntry { word: "phấn khích",  valence:  0.70, arousal: 0.80, dominance: 0.65 },
    WordEntry { word: "tốt",         valence:  0.45, arousal: 0.35, dominance: 0.65 },
    // Tiêu cực nhẹ
    WordEntry { word: "khó",         valence: -0.20, arousal: 0.45, dominance: 0.40 },
    WordEntry { word: "mệt",         valence: -0.30, arousal: 0.25, dominance: 0.35 },
    WordEntry { word: "chán",        valence: -0.40, arousal: 0.20, dominance: 0.30 },
    WordEntry { word: "lo",          valence: -0.35, arousal: 0.55, dominance: 0.30 },
    // Tiêu cực mạnh
    WordEntry { word: "buồn",        valence: -0.60, arousal: 0.30, dominance: 0.25 },
    WordEntry { word: "sợ",          valence: -0.65, arousal: 0.75, dominance: 0.20 },
    WordEntry { word: "đau",         valence: -0.65, arousal: 0.55, dominance: 0.20 },
    WordEntry { word: "mất mát",     valence: -0.70, arousal: 0.40, dominance: 0.20 },
];

// ─────────────────────────────────────────────────────────────────────────────
// TargetAffect — dẫn dần về phía tích cực
// ─────────────────────────────────────────────────────────────────────────────

/// Constants cho TargetAffect — không magic numbers.
const MAX_STEP_PER_TURN: f32 = 0.40; // không nhảy quá 0.40/bước (từ Go)
const PAUSE_STEP:        f32 = 0.15;
const SUPPORTIVE_STEP:   f32 = 0.20;
const SUPPORTIVE_FAST_D1_THRESHOLD: f32 = -0.30; // d1 quá xấu → bước nhỏ hơn
const SUPPORTIVE_SLOW_STEP: f32 = 0.15;
const REINFORCING_STEP:  f32 = 0.25;
const CELEBRATORY_STEP:  f32 = 0.35;
const GENTLE_STEP:       f32 = 0.10;
const DEFAULT_TARGET_V:  f32 = 0.35;
const DEFAULT_TARGET_A:  f32 = 0.45;

/// Tính EmotionTag mục tiêu cho response tiếp theo.
///
/// Logic từ Go TargetAffect():
///   - Không nhảy đột ngột (max MAX_STEP_PER_TURN)
///   - Tốc độ dẫn theo tone và d1
///   - Luôn hướng về phía tích cực
pub fn target_affect(curve: &ConversationCurve) -> EmotionTag {
    if curve.turn_count() == 0 {
        return EmotionTag { valence: DEFAULT_TARGET_V, arousal: DEFAULT_TARGET_A,
                            dominance: 0.60, intensity: 0.22 };
    };

    let tone = curve.tone();
    let d1   = curve.d1_now();
    let v    = curve.current_v();
    let cur_a = 0.45f32; // default arousal — curve only tracks valence

    let (target_v, target_a) = match tone {
        ResponseTone::Pause => {
            // Đột ngột xấu → bước nhỏ, giảm arousal
            (v + PAUSE_STEP, 0.30)
        }
        ResponseTone::Supportive => {
            // Đang giảm → dẫn lên, nhưng chậm nếu d1 rất xấu
            let step = if d1 < SUPPORTIVE_FAST_D1_THRESHOLD {
                SUPPORTIVE_SLOW_STEP
            } else {
                SUPPORTIVE_STEP
            };
            (v + step, cur_a * 0.85)
        }
        ResponseTone::Reinforcing => {
            // Đang hồi phục → tiếp đà
            (v + REINFORCING_STEP, cur_a + 0.10)
        }
        ResponseTone::Celebratory => {
            // Đang vui → đẩy lên thêm
            (v + CELEBRATORY_STEP, cur_a + 0.20)
        }
        ResponseTone::Gentle => {
            // Buồn ổn định → nhẹ nhàng, không ép
            (v + GENTLE_STEP, 0.25)
        }
        ResponseTone::Engaged => {
            // Trung tính → mặc định tích cực
            (DEFAULT_TARGET_V, DEFAULT_TARGET_A)
        }
    };

    EmotionTag {
        valence:   target_v.clamp(-0.95, 0.95),
        arousal:   target_a.clamp(0.10, 0.95),
        dominance: 0.60,
        intensity: v.abs() * 0.85,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SelectWords — chọn từ gần nhất với target
// ─────────────────────────────────────────────────────────────────────────────

/// Một từ với distance score (nhỏ hơn = phù hợp hơn).
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct WordCandidate {
    pub word:  &'static str,
    pub tag:   EmotionTag,
    pub score: f32,
}

/// Chọn N từ có EmotionTag gần target nhất.
///
/// Distance = sqrt(dV²×2 + dA²×0.5 + dD²×0.5)
/// Valence có weight ×2 vì quan trọng nhất.
pub fn select_words(target: EmotionTag, n: usize) -> Vec<WordCandidate> {
    let mut candidates: Vec<WordCandidate> = CORE_LEXICON.iter().map(|e| {
        let dv = e.valence   - target.valence;
        let da = e.arousal   - target.arousal;
        let dd = e.dominance - target.dominance;
        let dist = dv*dv*2.0 + da*da*0.5 + dd*dd*0.5; // squared OK for ranking
        WordCandidate {
            word:  e.word,
            tag:   EmotionTag { valence: e.valence, arousal: e.arousal,
                                dominance: e.dominance, intensity: e.valence.abs() },
            score: dist,
        }
    }).collect();

    candidates.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    candidates.truncate(n);
    candidates
}

// ─────────────────────────────────────────────────────────────────────────────
// AffectSentence — gợi ý từ ngữ mở đầu
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả của AffectSentence — không phải string hoàn chỉnh mà là components.
/// Caller quyết định ghép lại thế nào.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct SentenceComponents {
    /// Từ chính gần target nhất
    pub lead_word:    &'static str,
    /// Từ thứ hai
    pub support_word: &'static str,
    /// Tone hiện tại
    pub tone:         ResponseTone,
    /// Valence của target
    pub target_v:     f32,
}

/// Tính SentenceComponents từ ConversationCurve.
///
/// Caller dùng components này để ghép câu theo ngôn ngữ tự nhiên.
/// Không ghép sẵn ở đây — tránh hardcode string trong logic.
pub fn affect_components(curve: &ConversationCurve) -> SentenceComponents {
    let target = target_affect(curve);
    let words  = select_words(target, 3);

    let lead    = words.first().map(|w| w.word).unwrap_or("ổn");
    let support = words.get(1).map(|w| w.word).unwrap_or("tốt");

    SentenceComponents {
        lead_word:    lead,
        support_word: support,
        tone:         curve.tone(),
        target_v:     target.valence,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EmotionHistory — append-only emotion edges
// ─────────────────────────────────────────────────────────────────────────────

const MAX_HISTORY: usize = 500; // QT2: ∞-1

/// Một sự kiện cảm xúc trong lịch sử.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct EmotionEdge {
    pub timestamp: i64,
    pub tag:       EmotionTag,
    pub ctx:       EmotionContext,
    pub text:      String,
}

/// Lịch sử cảm xúc — append-only (QT8).
#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct EmotionHistory {
    pub edges: Vec<EmotionEdge>,
}

impl EmotionHistory {
    pub fn new() -> Self { Self::default() }

    /// Thêm sự kiện — append-only.
    pub fn add(&mut self, tag: EmotionTag, ctx: EmotionContext, text: &str, ts: i64) {
        self.edges.push(EmotionEdge {
            timestamp: ts, tag, ctx,
            text: text.chars().take(60).collect(),
        });
        // QT2: giới hạn hữu hạn, giữ 500 mới nhất
        if self.edges.len() > MAX_HISTORY {
            let drain = self.edges.len() - MAX_HISTORY;
            self.edges.drain(..drain);
        }
    }

    /// Cảm xúc xuất hiện nhiều nhất.
    pub fn dominant_emotion(&self) -> &'static str {
        if self.edges.is_empty() { return "neutral"; }
        let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
        for e in &self.edges {
            let label = valence_label(e.tag.valence);
            *counts.entry(label).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|(_, c)| *c)
            .map(|(k, _)| k).unwrap_or("neutral")
    }

    /// Trung bình VAD toàn lịch sử.
    pub fn average_vad(&self) -> EmotionTag {
        if self.edges.is_empty() { return EmotionTag::NEUTRAL; }
        let n = self.edges.len() as f32;
        let (mut v, mut a, mut d, mut i) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for e in &self.edges { v += e.tag.valence; a += e.tag.arousal; d += e.tag.dominance; i += e.tag.intensity; }
        EmotionTag { valence: v/n, arousal: a/n, dominance: d/n, intensity: i/n }
    }

    /// Xu hướng: improving / declining / stable.
    pub fn recent_trend(&self) -> &'static str {
        if self.edges.len() < 4 { return "stable"; }
        let mid = self.edges.len() / 2;
        let v1: f32 = self.edges[..mid].iter().map(|e| e.tag.valence).sum::<f32>() / mid as f32;
        let v2: f32 = self.edges[mid..].iter().map(|e| e.tag.valence).sum::<f32>() / (self.edges.len() - mid) as f32;
        let diff = v2 - v1;
        if diff > 0.10 { "improving" }
        else if diff < -0.10 { "declining" }
        else { "stable" }
    }

    /// Độ dao động cảm xúc (std valence).
    /// Độ dao động cảm xúc (variance valence — không dùng sqrt cho no_std).
    pub fn volatility(&self) -> f32 {
        if self.edges.len() < 2 { return 0.0; }
        let avg = self.average_vad().valence;
        self.edges.iter()
            .map(|e| { let d = e.tag.valence - avg; d * d })
            .sum::<f32>() / self.edges.len() as f32
    }

    /// Tổng hợp theo nguồn (role/source).
    pub fn by_source(&self) -> BTreeMap<String, EmotionTag> {
        let mut buckets: BTreeMap<String, Vec<EmotionTag>> = BTreeMap::new();
        for e in &self.edges {
            let key = alloc::format!("{}:{}", e.ctx.role.as_str(), e.ctx.source.as_str());
            buckets.entry(key).or_default().push(e.tag);
        }
        buckets.into_iter().map(|(k, tags)| {
            let n = tags.len() as f32;
            let (v, a, d, i) = tags.iter().fold((0.0f32, 0.0f32, 0.0f32, 0.0f32),
                |(v,a,d,i), t| (v+t.valence, a+t.arousal, d+t.dominance, i+t.intensity));
            (k, EmotionTag { valence: v/n, arousal: a/n, dominance: d/n, intensity: i/n })
        }).collect()
    }
}

fn valence_label(v: f32) -> &'static str {
    if v < -0.50 { "sad" }
    else if v < -0.20 { "uneasy" }
    else if v <  0.20 { "neutral" }
    else if v <  0.50 { "calm" }
    else { "happy" }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{EmotionContext, Role, EmotionSource};

    fn make_curve(vals: &[f32]) -> ConversationCurve {
        let mut c = ConversationCurve::new();
        for &v in vals { c.push(v); }
        c
    }

    // ── TargetAffect ─────────────────────────────────────────────────────────

    #[test]
    fn target_moves_toward_positive() {
        // Đang buồn → target phải cao hơn cur
        let curve = make_curve(&[-0.60, -0.65, -0.70]);
        let tgt   = target_affect(&curve);
        let cur_v = curve.current_v();
        assert!(tgt.valence > cur_v,
            "Target phải cao hơn current: {} > {}", tgt.valence, cur_v);
    }

    #[test]
    fn target_no_big_jump() {
        // Không nhảy quá MAX_STEP_PER_TURN
        let curve = make_curve(&[-0.70, -0.65, -0.60]);
        let tgt   = target_affect(&curve);
        let cur_v = curve.current_v();
        let jump  = (tgt.valence - cur_v).abs();
        assert!(jump <= MAX_STEP_PER_TURN + 0.01,
            "Jump {} > MAX {}", jump, MAX_STEP_PER_TURN);
    }

    #[test]
    fn target_empty_curve_default() {
        let curve = ConversationCurve::new();
        let tgt   = target_affect(&curve);
        assert!(tgt.valence > 0.0, "Empty curve → positive default");
    }

    #[test]
    fn supportive_slower_when_falling_fast() {
        // d1 < -0.30 → bước nhỏ hơn bình thường
        let fast_fall = make_curve(&[-0.0, -0.3, -0.7]);
        let slow_fall = make_curve(&[-0.0, -0.1, -0.2]);
        let t1 = target_affect(&fast_fall);
        let t2 = target_affect(&slow_fall);
        // Both supportive, fast fall có step nhỏ hơn
        let v1 = fast_fall.current_v();
        let v2 = slow_fall.current_v();
        let step1 = t1.valence - v1;
        let step2 = t2.valence - v2;
        assert!(step1 <= step2 + 0.01,
            "Fast fall step {} <= slow fall step {}", step1, step2);
    }

    // ── SelectWords ──────────────────────────────────────────────────────────

    #[test]
    fn select_words_returns_n() {
        let target = EmotionTag { valence: -0.50, arousal: 0.30, dominance: 0.30, intensity: 0.50 };
        let words  = select_words(target, 3);
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn select_words_nearest_first() {
        let target = EmotionTag { valence: 0.75, arousal: 0.65, dominance: 0.70, intensity: 0.60 };
        let words  = select_words(target, 5);
        // Từ đầu tiên phải gần target nhất
        assert!(words[0].score <= words[1].score, "Sorted by distance");
        // Từ đầu tiên phải là positive
        assert!(words[0].tag.valence > 0.0,
            "Nearest to V=0.75 phải positive: {} ({})", words[0].word, words[0].tag.valence);
    }

    #[test]
    fn select_words_sad_target() {
        let target = EmotionTag { valence: -0.55, arousal: 0.30, dominance: 0.25, intensity: 0.50 };
        let words  = select_words(target, 3);
        // Từ gần nhất phải negative
        assert!(words[0].tag.valence < 0.0,
            "Nearest to V=-0.55 phải negative: {}", words[0].word);
    }

    #[test]
    fn select_words_scores_ascending() {
        let target = EmotionTag::NEUTRAL;
        let words  = select_words(target, 10);
        for i in 1..words.len() {
            assert!(words[i].score >= words[i-1].score,
                "Scores phải tăng dần: {} >= {}", words[i].score, words[i-1].score);
        }
    }

    // ── AffectSentence ───────────────────────────────────────────────────────

    #[test]
    fn affect_components_not_empty() {
        let curve      = make_curve(&[-0.40]);
        let components = affect_components(&curve);
        assert!(!components.lead_word.is_empty());
        assert!(!components.support_word.is_empty());
    }

    #[test]
    fn affect_components_positive_curve() {
        let curve      = make_curve(&[0.60, 0.65]);
        let components = affect_components(&curve);
        assert!(components.target_v > 0.0,
            "Positive curve → positive target: {}", components.target_v);
    }

    // ── EmotionHistory ───────────────────────────────────────────────────────

    fn ctx() -> EmotionContext {
        EmotionContext { role: Role::FirstPerson, source: EmotionSource::RealNow,
                         recency: 1.0, shared: false, expected: false }
    }

    #[test]
    fn history_append_only() {
        let mut h = EmotionHistory::new();
        let t = EmotionTag { valence: -0.5, arousal: 0.4, dominance: 0.3, intensity: 0.5 };
        h.add(t, ctx(), "test", 1000);
        h.add(t, ctx(), "test2", 2000);
        assert_eq!(h.edges.len(), 2);
    }

    #[test]
    fn history_max_500() {
        let mut h = EmotionHistory::new();
        let t = EmotionTag::NEUTRAL;
        for i in 0..600 {
            h.add(t, ctx(), "x", i as i64);
        }
        assert_eq!(h.edges.len(), MAX_HISTORY,
            "Max {} entries: {}", MAX_HISTORY, h.edges.len());
    }

    #[test]
    fn history_dominant_sad() {
        let mut h = EmotionHistory::new();
        let sad = EmotionTag { valence: -0.6, ..EmotionTag::NEUTRAL };
        for i in 0..5 { h.add(sad, ctx(), "buồn", i); }
        h.add(EmotionTag { valence: 0.7, ..EmotionTag::NEUTRAL }, ctx(), "vui", 10);
        assert_eq!(h.dominant_emotion(), "sad");
    }

    #[test]
    fn history_trend_improving() {
        let mut h = EmotionHistory::new();
        for i in 0..4 { h.add(EmotionTag { valence: -0.5, ..EmotionTag::NEUTRAL }, ctx(), "x", i); }
        for i in 4..8 { h.add(EmotionTag { valence:  0.5, ..EmotionTag::NEUTRAL }, ctx(), "y", i); }
        assert_eq!(h.recent_trend(), "improving");
    }

    #[test]
    fn history_trend_declining() {
        let mut h = EmotionHistory::new();
        for i in 0..4 { h.add(EmotionTag { valence:  0.5, ..EmotionTag::NEUTRAL }, ctx(), "x", i); }
        for i in 4..8 { h.add(EmotionTag { valence: -0.5, ..EmotionTag::NEUTRAL }, ctx(), "y", i); }
        assert_eq!(h.recent_trend(), "declining");
    }

    #[test]
    fn history_volatility() {
        let mut h = EmotionHistory::new();
        // Lúc vui lúc buồn → volatility cao
        for i in 0..6 {
            let v = if i % 2 == 0 { 0.8 } else { -0.8 };
            h.add(EmotionTag { valence: v, ..EmotionTag::NEUTRAL }, ctx(), "x", i);
        }
        let vol = h.volatility();
        assert!(vol > 0.25, "Volatile history (variance): {}", vol);
    }

    #[test]
    fn history_by_source() {
        let mut h = EmotionHistory::new();
        let ctx_real    = EmotionContext { source: EmotionSource::RealNow, ..ctx() };
        let ctx_fiction = EmotionContext { source: EmotionSource::Fiction, ..ctx() };
        let sad  = EmotionTag { valence: -0.6, ..EmotionTag::NEUTRAL };
        let mild = EmotionTag { valence: -0.1, ..EmotionTag::NEUTRAL };
        h.add(sad,  ctx_real,    "thật", 1);
        h.add(mild, ctx_fiction, "phim", 2);
        let by_src = h.by_source();
        assert!(by_src.len() >= 2, "Ít nhất 2 sources");
    }

    #[test]
    fn history_average_vad() {
        let mut h = EmotionHistory::new();
        for i in 0..4 {
            h.add(EmotionTag { valence: -0.4, arousal: 0.5, dominance: 0.3, intensity: 0.4 },
                  ctx(), "x", i);
        }
        let avg = h.average_vad();
        assert!((avg.valence - (-0.4)).abs() < 0.01);
    }
}
