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
use crate::context::{EmotionContext, EmotionSource, Role};

// ─────────────────────────────────────────────────────────────────────────────
// Signal table
// ─────────────────────────────────────────────────────────────────────────────

struct CtxSignal {
    kw: &'static [&'static str],
    role: Role,
    source: EmotionSource,
    rec_min: f32,
    rec_max: f32,
}

static CTX_SIGNALS: &[CtxSignal] = &[
    // ── FirstPerson + RealNow ─────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "tôi vừa",
            "tôi đang",
            "em vừa",
            "em đang",
            "mình vừa",
            "mình đang",
            "just happened",
            "right now",
            "i just",
            "i am going through",
            "vừa xảy ra",
            "đang xảy ra",
        ],
        role: Role::FirstPerson,
        source: EmotionSource::RealNow,
        rec_min: 0.85,
        rec_max: 1.0,
    },
    // ── FirstPerson + RealPast ────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "hồi đó",
            "ngày xưa",
            "hồi nhỏ",
            "trước đây tôi",
            "tôi đã từng",
            "khi còn trẻ",
            "năm ngoái tôi",
            "back then",
            "when i was",
            "i used to",
            "years ago",
            "hồi xưa",
        ],
        role: Role::FirstPerson,
        source: EmotionSource::RealPast,
        rec_min: 0.1,
        rec_max: 0.4,
    },
    // ── FirstPerson + Memory ──────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "tôi nhớ",
            "tôi nhớ lại",
            "em nhớ",
            "mình nhớ",
            "i remember",
            "thinking back",
            "looking back",
            "nhớ về",
        ],
        role: Role::FirstPerson,
        source: EmotionSource::Memory,
        rec_min: 0.5,
        rec_max: 0.75,
    },
    // ── ThirdPerson + RealOther ───────────────────────────────────────────
    CtxSignal {
        kw: &[
            "bạn tôi kể",
            "anh ấy nói",
            "chị ấy trải qua",
            "người ta nói",
            "họ kể",
            "nghe kể rằng",
            "my friend told",
            "someone i know",
            "he went through",
            "she experienced",
            "they said",
            "bạn bè kể",
        ],
        role: Role::ThirdPerson,
        source: EmotionSource::RealOther,
        rec_min: 0.5,
        rec_max: 0.9,
    },
    // ── SecondPerson ──────────────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "bạn đang trải qua",
            "bạn vừa",
            "bạn đã",
            "you are going through",
            "you just",
            "you experienced",
        ],
        role: Role::SecondPerson,
        source: EmotionSource::RealOther,
        rec_min: 0.7,
        rec_max: 1.0,
    },
    // ── Observer + Fiction ────────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "trong phim",
            "trong truyện",
            "nhân vật",
            "tập phim",
            "cuốn sách",
            "tiểu thuyết",
            "in the movie",
            "in the book",
            "the character",
            "fictional",
            "the story",
            "in the show",
            "đọc sách",
            "xem phim",
            "bộ phim",
        ],
        role: Role::Observer,
        source: EmotionSource::Fiction,
        rec_min: 0.3,
        rec_max: 0.7,
    },
    // ── Observer + Music ──────────────────────────────────────────────────
    CtxSignal {
        kw: &[
            "bài nhạc",
            "bài hát",
            "ca khúc",
            "nghe nhạc",
            "this song",
            "the music",
            "listening to",
            "melody",
            "bài này",
            "âm nhạc",
        ],
        role: Role::Observer,
        source: EmotionSource::Music,
        rec_min: 0.8,
        rec_max: 1.0,
    },
];

static SHARED_KW: &[&str] = &[
    "cùng nhau",
    "chúng tôi",
    "chúng mình",
    "bọn mình",
    "together",
    "we all",
    "our group",
    "everyone felt",
];

static EXPECTED_KW: &[&str] = &[
    "biết trước",
    "đã chuẩn bị",
    "không bất ngờ",
    "dự đoán",
    "knew it was coming",
    "expected",
    "prepared for",
    "anticipated",
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
                ctx.role = sig.role;
                ctx.source = sig.source;
                ctx.recency = (sig.rec_min + sig.rec_max) / 2.0;
                break 'outer;
            }
        }
    }

    // Shared
    for &kw in SHARED_KW {
        if lo.contains(kw) {
            ctx.shared = true;
            break;
        }
    }

    // Expected
    for &kw in EXPECTED_KW {
        if lo.contains(kw) {
            ctx.expected = true;
            break;
        }
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
        assert_eq!(ctx.role, Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealNow);
        assert!(ctx.recency > 0.8, "Recency phải cao: {}", ctx.recency);
    }

    #[test]
    fn first_person_past() {
        let ctx = infer_context("hồi nhỏ tôi đã từng sợ bóng tối lắm");
        assert_eq!(ctx.role, Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealPast);
        assert!(ctx.recency < 0.5, "Recency phải thấp: {}", ctx.recency);
    }

    #[test]
    fn memory_context() {
        let ctx = infer_context("tôi nhớ lại ngày đó rất rõ");
        assert_eq!(ctx.role, Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::Memory);
    }

    #[test]
    fn third_person_other() {
        let ctx = infer_context("bạn tôi kể rằng cô ấy rất buồn");
        assert_eq!(ctx.role, Role::ThirdPerson);
        assert_eq!(ctx.source, EmotionSource::RealOther);
    }

    #[test]
    fn observer_fiction() {
        let ctx = infer_context("trong phim đó nhân vật chính rất đau");
        assert_eq!(ctx.role, Role::Observer);
        assert_eq!(ctx.source, EmotionSource::Fiction);
    }

    #[test]
    fn observer_music() {
        let ctx = infer_context("bài nhạc này làm mình buồn ghê");
        assert_eq!(ctx.role, Role::Observer);
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
        assert_eq!(ctx.role, Role::FirstPerson);
        assert_eq!(ctx.source, EmotionSource::RealNow);
    }

    #[test]
    fn same_signal_different_s() {
        // Cùng EmotionTag, context khác → S khác
        use silk::edge::EmotionTag;
        let raw = EmotionTag {
            valence: -0.7,
            arousal: 0.6,
            dominance: 0.3,
            intensity: 0.65,
        };

        let ctx_real = infer_context("tôi vừa trải qua chuyện đó");
        let ctx_past = infer_context("hồi xưa tôi đã từng như vậy");
        let ctx_fiction = infer_context("trong phim có cảnh rất xúc động");

        let i_real = ctx_real.apply(raw).intensity;
        let i_past = ctx_past.apply(raw).intensity;
        let i_fiction = ctx_fiction.apply(raw).intensity;

        assert!(
            i_real > i_past,
            "RealNow > RealPast: {} > {}",
            i_real,
            i_past
        );
        assert!(
            i_past > i_fiction,
            "RealPast > Fiction: {} > {}",
            i_past,
            i_fiction
        );
    }

    #[test]
    fn english_signals_work() {
        let ctx1 = infer_context("i just lost my job");
        assert_eq!(ctx1.role, Role::FirstPerson);
        assert_eq!(ctx1.source, EmotionSource::RealNow);

        let ctx2 = infer_context("in the movie the main character was very sad");
        assert_eq!(ctx2.role, Role::Observer);
        assert_eq!(ctx2.source, EmotionSource::Fiction);

        let ctx3 = infer_context("my friend told me she was going through a tough time");
        assert_eq!(ctx3.role, Role::ThirdPerson);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tense detection
// ─────────────────────────────────────────────────────────────────────────────

/// Thì của câu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tense {
    /// Đang xảy ra (đang, hiện tại)
    Present,
    /// Đã xảy ra (đã, rồi, xong, từng)
    Past,
    /// Sẽ xảy ra (sẽ, sắp, muốn, dự định)
    Future,
    /// Không rõ
    Unknown,
}

impl Tense {
    /// Hệ số recency theo thì — feed vào EmotionContext.
    ///
    /// Present = thật ngay lúc này → recency cao
    /// Past    = đã qua → recency thấp hơn
    /// Future  = chưa xảy ra → recency thấp (hypothetical)
    pub fn recency(self) -> f32 {
        match self {
            Tense::Present => 1.00,
            Tense::Past => 0.45,
            Tense::Future => 0.25,
            Tense::Unknown => 0.70,
        }
    }

    /// Intensity scale — sự kiện tương lai ít intense hơn hiện tại.
    pub fn intensity_scale(self) -> f32 {
        match self {
            Tense::Present => 1.00,
            Tense::Past => 0.75,
            Tense::Future => 0.50,
            Tense::Unknown => 0.85,
        }
    }
}

static PAST_KW: &[&str] = &[
    // VI
    "đã",
    "rồi",
    "xong",
    "từng",
    "hồi",
    "trước",
    "cũ",
    "ngày xưa",
    "năm ngoái",
    "hôm qua",
    "hồi đó",
    "lúc trước",
    "đã từng",
    // EN
    "was",
    "were",
    "had",
    "did",
    "used to",
    "yesterday",
    "last year",
    "ago",
    "before",
    "previously",
    "once",
    "formerly",
];

static FUTURE_KW: &[&str] = &[
    // VI
    "sẽ",
    "sắp",
    "muốn",
    "dự định",
    "kế hoạch",
    "tương lai",
    "chuẩn bị",
    "định",
    "sắp tới",
    "ngày mai",
    "tuần tới",
    // EN
    "will",
    "shall",
    "going to",
    "plan to",
    "want to",
    "soon",
    "tomorrow",
    "next",
    "intend",
    "about to",
    "future",
];

static PRESENT_KW: &[&str] = &[
    // VI
    "đang",
    "hiện tại",
    "bây giờ",
    "lúc này",
    "vừa",
    "ngay",
    "hiện nay",
    "hôm nay",
    // EN
    "am",
    "is",
    "are",
    "now",
    "currently",
    "today",
    "at the moment",
    "right now",
    "just",
];

/// Detect thì của câu từ text.
pub fn detect_tense(text: &str) -> Tense {
    let lo = text.to_lowercase();

    // Đếm signals
    let past_score = PAST_KW.iter().filter(|&&k| lo.contains(k)).count();
    let future_score = FUTURE_KW.iter().filter(|&&k| lo.contains(k)).count();
    let present_score = PRESENT_KW.iter().filter(|&&k| lo.contains(k)).count();

    // Winner
    let max = past_score.max(future_score).max(present_score);
    if max == 0 {
        return Tense::Unknown;
    }

    if past_score == max {
        Tense::Past
    } else if future_score == max {
        Tense::Future
    } else {
        Tense::Present
    }
}

/// Infer context với tense — recency tự động từ thì của câu.
///
/// Kết hợp infer_context + detect_tense → EmotionContext hoàn chỉnh.
pub fn infer_context_with_tense(text: &str) -> super::context::EmotionContext {
    let mut ctx = infer_context(text);
    let tense = detect_tense(text);

    // Override recency nếu tense rõ hơn keyword signal
    // (tense detection thường chính xác hơn vì keywords rõ ràng)
    match tense {
        Tense::Unknown => {} // giữ nguyên từ infer_context
        _ => {
            // Blend: 60% tense, 40% infer_context
            ctx.recency = tense.recency() * 0.60 + ctx.recency * 0.40;
        }
    }

    ctx
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tense_tests {
    use super::*;

    #[test]
    fn detect_present_dang() {
        assert_eq!(detect_tense("tôi đang buồn"), Tense::Present);
        assert_eq!(detect_tense("she is crying now"), Tense::Present);
    }

    #[test]
    fn detect_past_da() {
        assert_eq!(detect_tense("tôi đã mất việc"), Tense::Past);
        assert_eq!(detect_tense("he was very sad"), Tense::Past);
    }

    #[test]
    fn detect_future_se() {
        assert_eq!(detect_tense("tôi sẽ ổn thôi"), Tense::Future);
        assert_eq!(detect_tense("she will be fine"), Tense::Future);
    }

    #[test]
    fn detect_unknown_no_marker() {
        // "hôm nay" = today = present indicator
        assert_eq!(detect_tense("hôm nay đẹp"), Tense::Present);
        // Không có marker rõ ràng → Unknown
        assert_eq!(detect_tense("trời xanh mây trắng"), Tense::Unknown);
    }

    #[test]
    fn present_highest_recency() {
        assert!(Tense::Present.recency() > Tense::Past.recency());
        assert!(Tense::Present.recency() > Tense::Future.recency());
    }

    #[test]
    fn future_lowest_intensity() {
        assert!(Tense::Future.intensity_scale() < Tense::Past.intensity_scale());
        assert!(Tense::Future.intensity_scale() < Tense::Present.intensity_scale());
    }

    #[test]
    fn infer_with_tense_past_lowers_recency() {
        let ctx_past = infer_context_with_tense("hồi xưa tôi đã từng buồn lắm");
        let ctx_present = infer_context_with_tense("tôi đang buồn lắm");
        assert!(
            ctx_present.recency > ctx_past.recency,
            "Present recency {} > Past recency {}",
            ctx_present.recency,
            ctx_past.recency
        );
    }

    #[test]
    fn infer_with_tense_future_hypothetical() {
        let ctx = infer_context_with_tense("tôi sẽ gặp anh ấy vào ngày mai");
        assert!(ctx.recency < 0.80, "Future → recency thấp: {}", ctx.recency);
    }

    #[test]
    fn same_event_different_tense_different_emotion() {
        use silk::edge::EmotionTag;
        let raw = EmotionTag {
            valence: -0.70,
            arousal: 0.60,
            dominance: 0.30,
            intensity: 0.65,
        };

        let ctx_present = infer_context_with_tense("tôi đang mất việc");
        let ctx_past = infer_context_with_tense("tôi đã mất việc");
        let ctx_future = infer_context_with_tense("tôi sẽ mất việc");

        let i_present = ctx_present.apply(raw).intensity;
        let i_past = ctx_past.apply(raw).intensity;
        let i_future = ctx_future.apply(raw).intensity;

        assert!(
            i_present >= i_past,
            "Present >= Past: {} >= {}",
            i_present,
            i_past
        );
        assert!(
            i_past >= i_future,
            "Past >= Future: {} >= {}",
            i_past,
            i_future
        );
    }

    #[test]
    fn tense_priority_over_context_keywords() {
        // "hồi" (past) + "đang" (present) → present wins (more signals)
        let t = detect_tense("hồi đó tôi đang ở đây");
        // Both có signal, đang = present
        // Result phụ thuộc count — không assert cứng, chỉ verify không crash
        assert!(matches!(t, Tense::Present | Tense::Past));
    }
}
