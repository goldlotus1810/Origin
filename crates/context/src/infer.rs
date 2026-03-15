//! # infer — InferContext
//!
//! Tự suy luận EmotionContext từ văn bản tự nhiên.
//!
//! ```text
//! "tôi vừa mất việc"         → FirstPerson + RealNow  + recency=0.95
//! "hồi nhỏ tôi đã từng..."   → FirstPerson + RealPast + recency=0.25
//! "bạn tôi kể rằng..."       → ThirdPerson + RealOther
//! "trong phim đó..."          → Observer    + Fiction
//! "bài nhạc này..."           → Observer    + Music
//! ```

extern crate alloc;
use crate::context::{EmotionContext, Role, EmotionSource};

// ─────────────────────────────────────────────────────────────────────────────
// Signal table
// ─────────────────────────────────────────────────────────────────────────────

struct CtxSignal {
    kw:      &'static [&'static str],
    role:    Role,
    source:  EmotionSource,
    rec_min: f32,
    rec_max: f32,
}

static CTX_SIGNALS: &[CtxSignal] = &[
    // ── FirstPerson + RealNow ─────────────────────────────────────────────
    CtxSignal {
        kw: &["tôi vừa", "tôi đang", "em vừa", "em đang", "mình vừa", "mình đang",
              "just happened", "right now", "i just", "i am going through",
              "vừa xảy ra", "đang xảy ra"],
        role: Role::FirstPerson, source: EmotionSource::RealNow,
        rec_min: 0.85, rec_max: 1.0,
    },
    // ── FirstPerson + RealPast ────────────────────────────────────────────
    CtxSignal {
        kw: &["hồi đó", "ngày xưa", "hồi nhỏ", "trước đây tôi", "tôi đã từng",
              "khi còn trẻ", "năm ngoái tôi", "back then", "when i was",
              "i used to", "years ago", "hồi xưa"],
        role: Role::FirstPerson, source: EmotionSource::RealPast,
        rec_min: 0.1, rec_max: 0.4,
    },
    // ── FirstPerson + Memory ──────────────────────────────────────────────
    CtxSignal {
        kw: &["tôi nhớ", "tôi nhớ lại", "em nhớ", "mình nhớ",
              "i remember", "thinking back", "looking back", "nhớ về"],
        role: Role::FirstPerson, source: EmotionSource::Memory,
        rec_min: 0.5, rec_max: 0.75,
    },
    // ── ThirdPerson + RealOther ───────────────────────────────────────────
    CtxSignal {
        kw: &["bạn tôi kể", "anh ấy nói", "chị ấy trải qua", "người ta nói",
              "họ kể", "nghe kể rằng", "my friend told", "someone i know",
              "he went through", "she experienced", "they said", "bạn bè kể"],
        role: Role::ThirdPerson, source: EmotionSource::RealOther,
        rec_min: 0.5, rec_max: 0.9,
    },
    // ── SecondPerson ──────────────────────────────────────────────────────
    CtxSignal {
        kw: &["bạn đang trải qua", "bạn vừa", "bạn đã",
              "you are going through", "you just", "you experienced"],
        role: Role::SecondPerson, source: EmotionSource::RealOther,
        rec_min: 0.7, rec_max: 1.0,
    },
    // ── Observer + Fiction ────────────────────────────────────────────────
    CtxSignal {
        kw: &["trong phim", "trong truyện", "nhân vật", "tập phim",
              "cuốn sách", "tiểu thuyết", "in the movie", "in the book",
              "the character", "fictional", "the story", "in the show",
              "đọc sách", "xem phim", "bộ phim"],
        role: Role::Observer, source: EmotionSource::Fiction,
        rec_min: 0.3, rec_max: 0.7,
    },
    // ── Observer + Music ──────────────────────────────────────────────────
    CtxSignal {
        kw: &["bài nhạc", "bài hát", "ca khúc", "nghe nhạc",
              "this song", "the music", "listening to", "melody",
              "bài này", "âm nhạc"],
        role: Role::Observer, source: EmotionSource::Music,
        rec_min: 0.8, rec_max: 1.0,
    },
];

static SHARED_KW: &[&str] = &[
    "cùng nhau", "chúng tôi", "chúng mình", "bọn mình",
    "together", "we all", "our group", "everyone felt",
];

static EXPECTED_KW: &[&str] = &[
    "biết trước", "đã chuẩn bị", "không bất ngờ", "dự đoán",
    "knew it was coming", "expected", "prepared for", "anticipated",
];

// ─────────────────────────────────────────────────────────────────────────────
// infer_context
// ─────────────────────────────────────────────────────────────────────────────

/// Suy luận EmotionContext từ văn bản tự nhiên.
///
/// Trả về `EmotionContext::DEFAULT` nếu không có signal rõ ràng.
pub fn infer_context(text: &str) -> EmotionContext {
    let lo = text.to_lowercase();

    let mut ctx = EmotionContext::DEFAULT;

    // Tìm signal mạnh nhất (first match wins)
    'outer: for sig in CTX_SIGNALS {
        for &kw in sig.kw {
            if lo.contains(kw) {
                ctx.role    = sig.role;
                ctx.source  = sig.source;
                ctx.recency = (sig.rec_min + sig.rec_max) / 2.0;
                break 'outer;
            }
        }
    }

    // Shared
    for &kw in SHARED_KW {
        if lo.contains(kw) { ctx.shared = true; break; }
    }

    // Expected
    for &kw in EXPECTED_KW {
        if lo.contains(kw) { ctx.expected = true; break; }
    }

    ctx
}

/// Shortcut: infer context + apply lên EmotionTag.
pub fn infer_and_apply(text: &str, raw: silk::edge::EmotionTag) -> silk::edge::EmotionTag {
    infer_context(text).apply(raw)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_person_real_now() {
        let ctx = infer_context("tôi vừa mất việc hôm nay");
        assert_eq!(ctx.role,   Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealNow);
        assert!(ctx.recency > 0.8, "Recency phải cao: {}", ctx.recency);
    }

    #[test]
    fn first_person_past() {
        let ctx = infer_context("hồi nhỏ tôi đã từng sợ bóng tối lắm");
        assert_eq!(ctx.role,   Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealPast);
        assert!(ctx.recency < 0.5, "Recency phải thấp: {}", ctx.recency);
    }

    #[test]
    fn memory_context() {
        let ctx = infer_context("tôi nhớ lại ngày đó rất rõ");
        assert_eq!(ctx.role,   Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::Memory);
    }

    #[test]
    fn third_person_other() {
        let ctx = infer_context("bạn tôi kể rằng cô ấy rất buồn");
        assert_eq!(ctx.role,   Role::ThirdPerson);
        assert_eq!(ctx.source, EmotionSource::RealOther);
    }

    #[test]
    fn observer_fiction() {
        let ctx = infer_context("trong phim đó nhân vật chính rất đau");
        assert_eq!(ctx.role,   Role::Observer);
        assert_eq!(ctx.source, EmotionSource::Fiction);
    }

    #[test]
    fn observer_music() {
        let ctx = infer_context("bài nhạc này làm mình buồn ghê");
        assert_eq!(ctx.role,   Role::Observer);
        assert_eq!(ctx.source, EmotionSource::Music);
    }

    #[test]
    fn second_person() {
        let ctx = infer_context("bạn đang trải qua chuyện khó khăn");
        assert_eq!(ctx.role, Role::SecondPerson);
    }

    #[test]
    fn shared_detected() {
        let ctx = infer_context("chúng mình cùng nhau vượt qua");
        assert!(ctx.shared, "Shared phải được detect");
    }

    #[test]
    fn expected_detected() {
        let ctx = infer_context("tôi đã chuẩn bị tinh thần rồi");
        assert!(ctx.expected, "Expected phải được detect");
    }

    #[test]
    fn default_fallback() {
        // Không có signal → DEFAULT
        let ctx = infer_context("hôm nay thời tiết đẹp quá");
        assert_eq!(ctx.role,   Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealNow);
    }

    #[test]
    fn same_signal_different_s() {
        // Cùng EmotionTag, context khác → S khác
        use silk::edge::EmotionTag;
        let raw = EmotionTag { valence: -0.7, arousal: 0.6, dominance: 0.3, intensity: 0.65 };

        let ctx_real    = infer_context("tôi vừa trải qua chuyện đó");
        let ctx_past    = infer_context("hồi xưa tôi đã từng như vậy");
        let ctx_fiction = infer_context("trong phim có cảnh rất xúc động");

        let i_real    = ctx_real.apply(raw).intensity;
        let i_past    = ctx_past.apply(raw).intensity;
        let i_fiction = ctx_fiction.apply(raw).intensity;

        assert!(i_real > i_past,    "RealNow > RealPast: {} > {}", i_real, i_past);
        assert!(i_past > i_fiction, "RealPast > Fiction: {} > {}", i_past, i_fiction);
    }

    #[test]
    fn english_signals_work() {
        let ctx1 = infer_context("i just lost my job");
        assert_eq!(ctx1.role,   Role::FirstPerson);
        assert_eq!(ctx1.source, EmotionSource::RealNow);

        let ctx2 = infer_context("in the movie the main character was very sad");
        assert_eq!(ctx2.role,   Role::Observer);
        assert_eq!(ctx2.source, EmotionSource::Fiction);

        let ctx3 = infer_context("my friend told me she was going through a tough time");
        assert_eq!(ctx3.role,   Role::ThirdPerson);
    }
}
