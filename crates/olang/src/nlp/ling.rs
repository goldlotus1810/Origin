//! # ling — Linguistic Modifier Parser
//!
//! Nhận diện các modifier cú pháp ảnh hưởng đến EmotionTag:
//!
//! NEGATION:    không · chẳng · chưa · never · not · ne...pas
//!              → đảo valence: V → -V * 0.7
//!
//! AMPLIFIER:   rất · quá · lắm · cực · vô cùng · so · very · très
//!              → khuếch đại: V → V * 1.4 (capped ±1.0)
//!
//! DIMINISHER:  hơi · một chút · khá · tương đối · rather · un peu
//!              → giảm: V → V * 0.5
//!
//! CONTRAST:    nhưng · mà · tuy nhiên · but · mais · aber · pero
//!              → split sentence, weight second part more
//!
//! POS (parts of speech):
//!   Noun:      người · cái · con · việc · -tion · -ness · -heit
//!   Verb:      là · có · làm · thấy · feel · être · sein · ser
//!   Adjective: buồn · vui · đẹp · sad · happy · triste · schön
//!   Adverb:    nhanh · chậm · mạnh · quickly · lentement

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// ModifierKind
// ─────────────────────────────────────────────────────────────────────────────

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierKind {
    Negation,   // không · not · ne · non
    Amplifier,  // rất · very · très · sehr
    Diminisher, // hơi · rather · un peu · etwas
    Contrast,   // nhưng · but · mais · aber
}

// ─────────────────────────────────────────────────────────────────────────────
// PosTag — Parts of Speech
// ─────────────────────────────────────────────────────────────────────────────

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PosTag {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Modifier(ModifierKind),
    Other,
}

// ─────────────────────────────────────────────────────────────────────────────
// TaggedToken
// ─────────────────────────────────────────────────────────────────────────────

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct TaggedToken<'a> {
    pub word: &'a str,
    pub pos: PosTag,
}

// ─────────────────────────────────────────────────────────────────────────────
// POS data tables
// ─────────────────────────────────────────────────────────────────────────────

static NEGATIONS: &[&str] = &[
    // Vietnamese
    "không", "chẳng", "chưa", "đừng", "chả", // English
    "not", "never", "no", "neither", "nor", "without", // French
    "ne", "pas", "jamais", "rien", "ni", "sans", // German
    "nicht", "kein", "keine", "niemals", "nie", "ohne", // Spanish/Portuguese
    "no", "nunca", "jamás", "sin", // Chinese/Japanese
    "不", "没", "没有", "ない", "ず",
];

static AMPLIFIERS: &[&str] = &[
    // Vietnamese
    "rất",
    "quá",
    "lắm",
    "cực",
    "vô cùng",
    "thực sự",
    "hết sức",
    "cực kỳ",
    "siêu",
    "tuyệt đối",
    // English
    "very",
    "so",
    "extremely",
    "absolutely",
    "totally",
    "really",
    "incredibly",
    "super",
    "utterly",
    "deeply",
    // French
    "très",
    "tellement",
    "vraiment",
    "absolument",
    "extrêmement",
    "trop",
    "fort",
    "terriblement",
    // German
    "sehr",
    "so",
    "wirklich",
    "absolut",
    "total",
    "unglaublich",
    "extrem",
    "furchtbar",
    "schrecklich",
    // Spanish/Portuguese
    "muy",
    "tan",
    "realmente",
    "absolutamente",
    "totalmente",
    "bastante",
    "demasiado",
    // Chinese/Japanese
    "非常",
    "太",
    "很",
    "真的",
    "とても",
    "すごく",
];

static DIMINISHERS: &[&str] = &[
    // Vietnamese
    "hơi",
    "một chút",
    "khá",
    "tương đối",
    "ít",
    "đôi chút",
    "nhẹ",
    "vừa vừa",
    "cũng được",
    // English
    "rather",
    "somewhat",
    "slightly",
    "a bit",
    "a little",
    "fairly",
    "kind of",
    "sort of",
    "not very",
    // French
    "assez",
    "un peu",
    "plutôt",
    "légèrement",
    "relativement",
    "quelque peu",
    // German
    "etwas",
    "ziemlich",
    "ein bisschen",
    "recht",
    "relativ",
    // Spanish/Portuguese
    "bastante",
    "algo",
    "un poco",
    "relativamente",
    // Chinese/Japanese
    "有点",
    "稍微",
    "ちょっと",
    "少し",
];

static CONTRASTS: &[&str] = &[
    // Vietnamese
    "nhưng",
    "mà",
    "tuy nhiên",
    "thế nhưng",
    "song",
    "dù",
    "dẫu",
    "mặc dù",
    "dù vậy",
    "tuy vậy",
    // English
    "but",
    "however",
    "although",
    "though",
    "yet",
    "still",
    "nevertheless",
    "nonetheless",
    "despite",
    // French
    "mais",
    "cependant",
    "pourtant",
    "néanmoins",
    "quand même",
    "bien que",
    "malgré",
    // German
    "aber",
    "jedoch",
    "obwohl",
    "trotzdem",
    "dennoch",
    "doch",
    "allerdings",
    // Spanish/Portuguese
    "pero",
    "sin embargo",
    "aunque",
    "no obstante",
    "a pesar",
];

// ─────────────────────────────────────────────────────────────────────────────
// LingParser
// ─────────────────────────────────────────────────────────────────────────────

/// Parse text → tagged tokens + modifier context.
pub struct LingParser;

#[allow(missing_docs)]
impl LingParser {
    pub fn new() -> Self {
        Self
    }

    /// Tag tokens trong một sentence.
    pub fn tag<'a>(&self, tokens: &'a [&'a str]) -> Vec<TaggedToken<'a>> {
        tokens
            .iter()
            .map(|&word| {
                let pos = self.tag_word(word);
                TaggedToken { word, pos }
            })
            .collect()
    }

    /// Tag một word.
    pub fn tag_word(&self, word: &str) -> PosTag {
        let lower = word.to_lowercase();
        let s = lower.as_str();

        if self.is_negation(s) {
            return PosTag::Modifier(ModifierKind::Negation);
        }
        if self.is_amplifier(s) {
            return PosTag::Modifier(ModifierKind::Amplifier);
        }
        if self.is_diminisher(s) {
            return PosTag::Modifier(ModifierKind::Diminisher);
        }
        if self.is_contrast(s) {
            return PosTag::Modifier(ModifierKind::Contrast);
        }
        if self.is_adjective(s) {
            return PosTag::Adjective;
        }
        if self.is_verb(s) {
            return PosTag::Verb;
        }
        if self.is_noun(s) {
            return PosTag::Noun;
        }
        if self.is_adverb(s) {
            return PosTag::Adverb;
        }

        PosTag::Other
    }

    fn is_negation(&self, w: &str) -> bool {
        NEGATIONS.contains(&w)
    }
    fn is_amplifier(&self, w: &str) -> bool {
        AMPLIFIERS.contains(&w)
    }
    fn is_diminisher(&self, w: &str) -> bool {
        DIMINISHERS.contains(&w)
    }
    fn is_contrast(&self, w: &str) -> bool {
        CONTRASTS.contains(&w)
    }

    fn is_adjective(&self, w: &str) -> bool {
        // Vietnamese adjectives — thường đứng sau noun
        matches!(w, "buồn"|"vui"|"đẹp"|"xấu"|"mệt"|"khỏe"|"đau"|"sợ"|
                    "giận"|"tức"|"yêu"|"ghét"|"tốt"|"tệ"|"hay"|"dở"|
                    "nhanh"|"chậm"|"to"|"nhỏ"|"cao"|"thấp"|"dài"|"ngắn")
        // EN adjectives (có trong word_affect)
        || matches!(w, "sad"|"happy"|"angry"|"scared"|"tired"|"great"|
                       "terrible"|"wonderful"|"awful"|"beautiful"|"ugly")
        // FR/DE/ES adjectives
        || matches!(w, "triste"|"heureux"|"heureuse"|"traurig"|"glücklich"|
                       "feliz")
        // Suffix-based (crude but effective for EN)
        || w.ends_with("ful") || w.ends_with("less") || w.ends_with("ous")
        || w.ends_with("ive") || w.ends_with("ible") || w.ends_with("al")
    }

    fn is_verb(&self, w: &str) -> bool {
        matches!(w, "là"|"có"|"làm"|"thấy"|"cảm"|"muốn"|"nghĩ"|"biết"|
                    "nhớ"|"quên"|"thích"|"ghét"|"yêu"|"sợ"|"lo"|"chạy"|
                    "đi"|"đến"|"về"|"ngủ"|"ăn"|"uống"|"nói"|"hỏi"|"trả lời")
        || matches!(w, "feel"|"feels"|"felt"|"think"|"know"|"want"|"need"|
                       "love"|"hate"|"like"|"enjoy"|"suffer"|"hope"|"fear")
        || matches!(w, "être"|"avoir"|"faire"|"voir"|"vouloir"|"pouvoir"|
                       "sentir"|"aimer"|"détester")
        || matches!(w, "sein"|"haben"|"machen"|"sehen"|"wollen"|"können"|
                       "fühlen"|"lieben"|"hassen")
        || matches!(w, "ser"|"estar"|"tener"|"hacer"|"ver"|"querer"|
                       "sentir"|"amar"|"odiar")
        // EN verb suffixes
        || w.ends_with("ing") || w.ends_with("ed")
    }

    fn is_noun(&self, w: &str) -> bool {
        // Basic person/thing nouns
        matches!(w, "tôi"|"mình"|"em"|"anh"|"chị"|"bạn"|"họ"|"người"|
                    "nhà"|"gia đình"|"bố"|"mẹ"|"con"|"việc"|"chuyện"|"ngày")
        || matches!(w, "i"|"me"|"you"|"he"|"she"|"we"|"they"|"it"|
                       "thing"|"day"|"life"|"work"|"home"|"family")
        // Suffixes
        || w.ends_with("tion") || w.ends_with("ness") || w.ends_with("ment")
        || w.ends_with("heit") || w.ends_with("keit") || w.ends_with("ung")
        || w.ends_with("ción") || w.ends_with("dad")  || w.ends_with("tad")
    }

    fn is_adverb(&self, w: &str) -> bool {
        matches!(w, "mãi"|"vẫn"|"đã"|"sẽ"|"còn"|"nữa"|"cũng"|"đều"|
                    "luôn"|"thường"|"đôi khi"|"hiếm khi"|"lúc nào")
        || matches!(w, "always"|"never"|"often"|"sometimes"|"usually"|
                       "already"|"still"|"yet"|"again"|"soon"|"now")
        || matches!(w, "toujours"|"jamais"|"souvent"|"parfois"|"déjà")
        || matches!(w, "immer"|"niemals"|"oft"|"manchmal"|"bereits")
        // Suffix
        || w.ends_with("ly") || w.ends_with("ment")
    }
}

impl Default for LingParser {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// apply_modifiers — transform EmotionTag based on modifier context
// ─────────────────────────────────────────────────────────────────────────────

/// Apply linguistic modifiers lên một sequence EmotionTag.
///
/// Input: tagged tokens + their emotion values
/// Output: modified EmotionTag cho toàn sentence
pub fn apply_modifiers(tokens: &[&str], base_v: f32, base_a: f32) -> (f32, f32) {
    let mut v = base_v;
    let mut a = base_a;

    let lower: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();
    let ls: Vec<&str> = lower.iter().map(|s| s.as_str()).collect();

    // Detect modifiers và áp dụng theo thứ tự
    let mut neg_count = 0u32;
    let mut amp_found = false;
    let mut dim_found = false;
    let mut contrast_idx: Option<usize> = None;

    for (i, &w) in ls.iter().enumerate() {
        if NEGATIONS.contains(&w) {
            neg_count += 1;
        }
        if AMPLIFIERS.contains(&w) {
            amp_found = true;
        }
        if DIMINISHERS.contains(&w) {
            dim_found = true;
        }
        if CONTRASTS.contains(&w) && contrast_idx.is_none() {
            contrast_idx = Some(i);
        }
    }

    // Contrast: tách sentence, phần sau contrast có weight cao hơn
    if let Some(ci) = contrast_idx {
        // Tính emotion riêng cho 2 phần
        let before = &ls[..ci];
        let after = &ls[ci + 1..];
        let v_before = sub_valence(before, base_v);
        let v_after = sub_valence(after, base_v);
        // Weight: after=0.65, before=0.35
        v = v_before * 0.35 + v_after * 0.65;
        return (v.clamp(-1.0, 1.0), a);
    }

    // Negation: odd count → negate, even count → double-neg (weaker positive)
    if neg_count > 0 {
        if neg_count % 2 == 1 {
            // "không buồn" → -V * 0.7 (không hoàn toàn trung tính)
            v = -v * 0.70;
            a *= 0.80;
        } else {
            // "không phải không vui" → nhẹ positive
            v = v.abs() * 0.40;
        }
    }

    // Amplifier: khuếch đại
    if amp_found {
        v = (v * 1.45).clamp(-1.0, 1.0);
        a = (a * 1.30).clamp(0.0, 1.0);
    }

    // Diminisher: giảm
    if dim_found && !amp_found {
        v *= 0.50;
        a *= 0.70;
    }

    (v, a)
}

/// Tính valence proxy cho một sub-sequence tokens.
fn sub_valence(tokens: &[&str], fallback: f32) -> f32 {
    // Scan tokens cho emotion words
    let mut v = 0.0f32;
    let mut count = 0u32;
    for &w in tokens {
        // Simple: check nếu word có trong amplifiers/negations/emotion
        if NEGATIONS.contains(&w) {
            v -= 0.3;
            count += 1;
        } else if AMPLIFIERS.contains(&w) { /* skip */
        } else {
            // Crude: assume word carries base_v if unknown
            count += 1;
        }
    }
    if count == 0 {
        fallback
    } else {
        v
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> LingParser {
        LingParser::new()
    }

    // ── POS tagging ───────────────────────────────────────────────────────────

    #[test]
    fn tag_negation_vi() {
        assert_eq!(
            parser().tag_word("không"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("chẳng"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("chưa"),
            PosTag::Modifier(ModifierKind::Negation)
        );
    }

    #[test]
    fn tag_negation_multilang() {
        assert_eq!(
            parser().tag_word("not"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("pas"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("nicht"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("no"),
            PosTag::Modifier(ModifierKind::Negation)
        );
        assert_eq!(
            parser().tag_word("不"),
            PosTag::Modifier(ModifierKind::Negation)
        );
    }

    #[test]
    fn tag_amplifier_vi() {
        assert_eq!(
            parser().tag_word("rất"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("quá"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("lắm"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("cực"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
    }

    #[test]
    fn tag_amplifier_multilang() {
        assert_eq!(
            parser().tag_word("very"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("très"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("sehr"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("muy"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
        assert_eq!(
            parser().tag_word("非常"),
            PosTag::Modifier(ModifierKind::Amplifier)
        );
    }

    #[test]
    fn tag_diminisher() {
        assert_eq!(
            parser().tag_word("hơi"),
            PosTag::Modifier(ModifierKind::Diminisher)
        );
        assert_eq!(
            parser().tag_word("rather"),
            PosTag::Modifier(ModifierKind::Diminisher)
        );
        assert_eq!(
            parser().tag_word("etwas"),
            PosTag::Modifier(ModifierKind::Diminisher)
        );
    }

    #[test]
    fn tag_contrast() {
        assert_eq!(
            parser().tag_word("nhưng"),
            PosTag::Modifier(ModifierKind::Contrast)
        );
        assert_eq!(
            parser().tag_word("but"),
            PosTag::Modifier(ModifierKind::Contrast)
        );
        assert_eq!(
            parser().tag_word("aber"),
            PosTag::Modifier(ModifierKind::Contrast)
        );
        assert_eq!(
            parser().tag_word("mais"),
            PosTag::Modifier(ModifierKind::Contrast)
        );
        assert_eq!(
            parser().tag_word("pero"),
            PosTag::Modifier(ModifierKind::Contrast)
        );
    }

    #[test]
    fn tag_adjective_vi() {
        assert_eq!(parser().tag_word("buồn"), PosTag::Adjective);
        assert_eq!(parser().tag_word("vui"), PosTag::Adjective);
        assert_eq!(parser().tag_word("mệt"), PosTag::Adjective);
    }

    #[test]
    fn tag_verb_vi() {
        assert_eq!(parser().tag_word("là"), PosTag::Verb);
        assert_eq!(parser().tag_word("có"), PosTag::Verb);
        assert_eq!(parser().tag_word("thấy"), PosTag::Verb);
    }

    // ── apply_modifiers ───────────────────────────────────────────────────────

    #[test]
    fn negation_inverts_valence() {
        let base_v = -0.7; // buồn
        let (v, _) = apply_modifiers(&["tôi", "không", "buồn"], base_v, 0.5);
        assert!(v > 0.0, "không buồn → positive: {}", v);
        assert!(v < 0.7, "không buồn → không hoàn toàn positive: {}", v);
    }

    #[test]
    fn amplifier_strengthens() {
        let base_v = -0.7; // buồn
        let (v, _) = apply_modifiers(&["tôi", "rất", "buồn"], base_v, 0.5);
        assert!(v < -0.7, "rất buồn → mạnh hơn: {}", v);
        assert!(v >= -1.0, "capped tại -1.0: {}", v);
    }

    #[test]
    fn diminisher_weakens() {
        let base_v = -0.7;
        let (v, _) = apply_modifiers(&["hơi", "buồn"], base_v, 0.5);
        assert!(v > -0.7, "hơi buồn → nhẹ hơn: {}", v);
        assert!(v < 0.0, "vẫn negative: {}", v);
    }

    #[test]
    fn contrast_weights_second_part() {
        // "buồn nhưng vui" — phần sau contrast "vui" nên dominate
        let (v1, _) = apply_modifiers(&["buồn"], -0.7, 0.5);
        let (v2, _) = apply_modifiers(&["buồn", "nhưng", "vui"], -0.7, 0.5);
        // v2 phải gần 0 hoặc positive hơn v1
        assert!(v2 > v1, "contrast: vui sau nhưng dominate: {} > {}", v2, v1);
    }

    #[test]
    fn double_negation_positive() {
        let (v, _) = apply_modifiers(&["không", "phải", "không", "vui"], 0.8, 0.5);
        // "không phải không vui" → mild positive
        assert!(v > 0.0, "double neg → mild positive: {}", v);
        assert!(v < 0.8, "weaker than original: {}", v);
    }

    #[test]
    fn no_modifier_unchanged() {
        let (v, a) = apply_modifiers(&["tôi", "buồn"], -0.7, 0.5);
        // No modifier → same as base (slight variance OK)
        assert!((v - (-0.7)).abs() < 0.01, "no modifier: {}", v);
        assert!((a - 0.5).abs() < 0.01);
    }

    #[test]
    fn multilang_negation_works() {
        // French: "je ne suis pas triste"
        let (v, _) = apply_modifiers(&["je", "ne", "suis", "pas", "triste"], -0.7, 0.5);
        assert!(v > 0.0, "FR negation: {}", v);

        // German: "ich bin nicht traurig"
        let (v2, _) = apply_modifiers(&["ich", "bin", "nicht", "traurig"], -0.7, 0.5);
        assert!(v2 > 0.0, "DE negation: {}", v2);
    }

    #[test]
    fn multilang_amplifier_works() {
        // German: "ich bin sehr glücklich" → very happy
        let (v, _) = apply_modifiers(&["ich", "bin", "sehr", "glücklich"], 0.8, 0.65);
        assert!(v > 0.8, "DE amplifier: {}", v);

        // French: "je suis très heureux"
        let (v2, _) = apply_modifiers(&["je", "suis", "très", "heureux"], 0.8, 0.65);
        assert!(v2 > 0.8, "FR amplifier: {}", v2);
    }
}
