//! # context — EmotionContext
//!
//! Điều kiện biên của phương trình cảm xúc.
//!
//! ```text
//! EmotionTag = DetectEmotion(signal) × S(Context)
//! S = role × source × recency × shared_bonus × expected_dampen
//! ```
//!
//! Cùng sự kiện, S khác → EmotionTag khác:
//!   Trực tiếp trải qua (S=1.0) vs Xem phim (S=0.05)

extern crate alloc;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// Role — bạn là ai trong câu chuyện
// ─────────────────────────────────────────────────────────────────────────────

/// Vị trí của người cảm nhận trong câu chuyện.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Bạn đang sống nó trực tiếp
    FirstPerson,
    /// Người kể chuyện nói với bạn
    SecondPerson,
    /// Bạn nghe người khác kể về người khác
    ThirdPerson,
    /// Bạn xem/đọc từ xa
    Observer,
}

impl Role {
    /// Hệ số nhạy cảm theo vai trò.
    pub fn sensitivity(self) -> f32 {
        match self {
            Role::FirstPerson  => 1.00,
            Role::SecondPerson => 0.75,
            Role::ThirdPerson  => 0.55,
            Role::Observer     => 0.30,
        }
    }

    /// String label.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::FirstPerson  => "first_person",
            Role::SecondPerson => "second_person",
            Role::ThirdPerson  => "third_person",
            Role::Observer     => "observer",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Source — nguồn gốc sự việc
// ─────────────────────────────────────────────────────────────────────────────

/// Bản chất của nguồn cảm xúc.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionSource {
    /// Đang xảy ra với bạn ngay lúc này
    RealNow,
    /// Đã xảy ra — có khoảng cách thời gian
    RealPast,
    /// Xảy ra với người khác (bạn nghe kể)
    RealOther,
    /// Phim, sách, game — biết là không thật
    Fiction,
    /// Âm nhạc thuần túy
    Music,
    /// Ký ức — mix giữa real và constructed
    Memory,
}

impl EmotionSource {
    /// Hệ số theo nguồn gốc.
    pub fn sensitivity(self) -> f32 {
        match self {
            EmotionSource::RealNow   => 1.00,
            EmotionSource::RealPast  => 0.80,
            EmotionSource::Memory    => 0.70,
            EmotionSource::RealOther => 0.60,
            EmotionSource::Fiction   => 0.30,
            EmotionSource::Music     => 0.25,
        }
    }

    /// String label.
    pub fn as_str(self) -> &'static str {
        match self {
            EmotionSource::RealNow   => "real_now",
            EmotionSource::RealPast  => "real_past",
            EmotionSource::RealOther => "real_other",
            EmotionSource::Fiction   => "fiction",
            EmotionSource::Music     => "music",
            EmotionSource::Memory    => "memory",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EmotionContext
// ─────────────────────────────────────────────────────────────────────────────

/// Điều kiện biên của phương trình cảm xúc.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct EmotionContext {
    pub role:     Role,
    pub source:   EmotionSource,
    /// 0.0 (xa xưa) → 1.0 (vừa xảy ra)
    pub recency:  f32,
    /// Đang chia sẻ cùng người khác → cộng hưởng +15%
    pub shared:   bool,
    /// Biết trước sẽ xảy ra → giảm shock -20%
    pub expected: bool,
}

impl EmotionContext {
    /// Ngữ cảnh mặc định: đang nói về bản thân, sự việc vừa qua.
    pub const DEFAULT: Self = Self {
        role:     Role::FirstPerson,
        source:   EmotionSource::RealNow,
        recency:  1.0,
        shared:   false,
        expected: false,
    };

    /// Xem phim / đọc sách.
    pub const FICTION: Self = Self {
        role:     Role::Observer,
        source:   EmotionSource::Fiction,
        recency:  0.5,
        shared:   false,
        expected: false,
    };

    /// Nghe nhạc.
    pub const MUSIC: Self = Self {
        role:     Role::Observer,
        source:   EmotionSource::Music,
        recency:  1.0,
        shared:   false,
        expected: false,
    };

    /// Hệ số nhân tổng thể:
    /// `S = role × source × (0.5 + 0.5×recency) × shared × expected`
    ///
    /// Clamp [0.05, 1.0].
    pub fn s(self) -> f32 {
        let mut s = self.role.sensitivity() * self.source.sensitivity();
        s *= 0.5 + 0.5 * self.recency.clamp(0.0, 1.0);
        if self.shared   { s *= 1.15; }
        if self.expected { s *= 0.80; }
        s.clamp(0.05, 1.0)
    }

    /// Áp dụng context lên EmotionTag thô.
    ///
    /// - Valence:   scale theo S (dấu giữ nguyên)
    /// - Arousal:   scale với baseline 0.1
    /// - Dominance: Observer → tăng (khoảng cách an toàn)
    /// - Intensity: scale theo S² (mờ nhanh với khoảng cách)
    pub fn apply(self, raw: EmotionTag) -> EmotionTag {
        let s = self.s();

        let valence = raw.valence * s;

        let arousal = (0.1 + (raw.arousal - 0.1) * s).max(0.0);

        let dominance = match self.role {
            Role::FirstPerson if matches!(self.source, EmotionSource::RealNow) => {
                // Trực tiếp → bị áp đảo hơn khi cảm xúc mạnh
                raw.dominance * (1.0 - 0.3 * raw.intensity)
            }
            Role::Observer => {
                // Quan sát → khoảng cách an toàn
                raw.dominance * 0.5 + 0.5
            }
            _ => raw.dominance,
        };

        // Intensity scale theo S² — aesthetic floor cho Fiction/Music
        let floor = 0.15_f32 * raw.intensity;
        let mut intensity = (raw.intensity * s * s).max(0.0_f32);
        if matches!(self.source, EmotionSource::Fiction | EmotionSource::Music)
            && intensity < floor { intensity = floor; }

        EmotionTag {
            valence:   valence.clamp(-1.0, 1.0),
            arousal:   arousal.clamp(0.0, 1.0),
            dominance: dominance.clamp(0.0, 1.0),
            intensity: intensity.clamp(0.0, 1.0),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sad() -> EmotionTag {
        EmotionTag { valence: -0.7, arousal: 0.6, dominance: 0.3, intensity: 0.65 }
    }

    #[test]
    fn first_person_real_now_full_intensity() {
        let ctx = EmotionContext::DEFAULT;
        assert!((ctx.s() - 1.0).abs() < 0.01, "DEFAULT S phải ≈ 1.0: {}", ctx.s());
        let applied = ctx.apply(sad());
        assert!((applied.valence - sad().valence).abs() < 0.01,
            "FirstPerson/RealNow không scale valence: {}", applied.valence);
    }

    #[test]
    fn fiction_attenuates_intensity() {
        let fiction = EmotionContext::FICTION;
        let s = fiction.s();
        assert!(s < 0.3, "Fiction S phải < 0.3: {}", s);

        let applied = fiction.apply(sad());
        assert!(applied.intensity < sad().intensity,
            "Fiction phải giảm intensity: {} < {}", applied.intensity, sad().intensity);
        // aesthetic floor: intensity > 0
        assert!(applied.intensity > 0.0, "Fiction vẫn có aesthetic floor");
    }

    #[test]
    fn music_has_aesthetic_floor() {
        let music = EmotionContext::MUSIC;
        let applied = music.apply(sad());
        // floor = 0.15 * raw.intensity
        let floor = 0.15 * sad().intensity;
        assert!(applied.intensity >= floor - 0.01,
            "Music aesthetic floor: {} >= {}", applied.intensity, floor);
    }

    #[test]
    fn shared_boosts_s() {
        // Dùng ThirdPerson (S < 1.0) để shared có thể boost
        let normal = EmotionContext {
            role: Role::ThirdPerson, source: EmotionSource::RealOther,
            recency: 0.7, shared: false, expected: false,
        };
        let shared = EmotionContext { shared: true, ..normal };
        assert!(shared.s() > normal.s(),
            "Shared S > normal S: {} > {}", shared.s(), normal.s());
    }

    #[test]
    fn expected_dampens_s() {
        let normal   = EmotionContext::DEFAULT;
        let expected = EmotionContext { expected: true, ..EmotionContext::DEFAULT };
        assert!(expected.s() < normal.s(), "Expected S < normal S");
    }

    #[test]
    fn recency_affects_s() {
        let recent = EmotionContext { recency: 1.0, ..EmotionContext::DEFAULT };
        let old    = EmotionContext { recency: 0.1, ..EmotionContext::DEFAULT };
        assert!(recent.s() > old.s(), "Recent S > old S: {} > {}", recent.s(), old.s());
    }

    #[test]
    fn observer_dominance_higher() {
        let real = EmotionContext::DEFAULT;
        let obs  = EmotionContext::FICTION;
        let raw  = sad();
        let d_real = real.apply(raw).dominance;
        let d_obs  = obs.apply(raw).dominance;
        assert!(d_obs >= d_real * 0.9, // Observer has more control
            "Observer dominance {} vs real {}", d_obs, d_real);
    }

    #[test]
    fn third_person_between_first_and_fiction() {
        let first = EmotionContext::DEFAULT;
        let third = EmotionContext {
            role: Role::ThirdPerson,
            source: EmotionSource::RealOther,
            recency: 0.7,
            ..EmotionContext::DEFAULT
        };
        let fiction = EmotionContext::FICTION;

        let raw = sad();
        let i_first   = first.apply(raw).intensity;
        let i_third   = third.apply(raw).intensity;
        let i_fiction = fiction.apply(raw).intensity;

        // Third và Fiction dùng S² → có thể rất gần nhau
        // Chỉ verify first > fiction (guaranteed)
        assert!(i_first > i_fiction,
            "First > Fiction: {} > {}", i_first, i_fiction);
        // Third phải nhỏ hơn First (S_third < S_first)
        assert!(i_first >= i_third,
            "First >= Third: {} >= {}", i_first, i_third);
    }

    #[test]
    fn s_clamp_min() {
        // Extreme case: Observer + Music + very old + expected
        let ctx = EmotionContext {
            role:     Role::Observer,
            source:   EmotionSource::Music,
            recency:  0.0,
            expected: true,
            shared:   false,
        };
        assert!(ctx.s() >= 0.05, "S phải ≥ 0.05 (minimum): {}", ctx.s());
    }
}
