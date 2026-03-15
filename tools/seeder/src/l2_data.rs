//! # l2_data — Seed KnowTree L2 từ multilingual corpus
//!
//! L2 = tầng khái niệm — học từ data thực tế, không phải axioms.
//! Mỗi entry: nhiều từ cùng nghĩa trong nhiều ngôn ngữ → 1 node L2.
//!
//! Feature flag: cargo run -p seeder --bin l2_data

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::qr::QRSigner;
use olang::writer::OlangWriter;
use olang::registry::Registry;

fn now_ns() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64
}

// ─────────────────────────────────────────────────────────────────────────────
// L2 Data: (representative_emoji_cp, &[(language_code, word)])
// Extracted từ multilingual sentiment corpus + manual curation
// ─────────────────────────────────────────────────────────────────────────────
static L2_NODES: &[(u32, &[(&str, &str)])] = &[
    // ── Positive emotions ────────────────────────────────────────────────────
    (0x1F60A, &[  // 😊 joy
        ("vi", "vui"),    ("vi", "hạnh phúc"), ("vi", "vui vẻ"),
        ("en", "happy"),  ("en", "joy"),        ("en", "great"),
        ("fr", "heureux"),("fr", "joie"),       ("fr", "grâce"),
        ("de", "schönes"),("de", "träume"),     ("de", "glückwunsch"),
        ("es", "buena"),  ("es", "genial"),     ("es", "felicidades"),
        ("it", "buon"),   ("it", "bellissimo"), ("it", "auguri"),
        ("zh", "高兴"),   ("zh", "开心"),       ("zh", "快乐"),
        ("ja", "嬉しい"), ("ja", "楽しい"),
        ("pt", "feliz"),  ("pt", "alegre"),
    ]),
    (0x2764, &[  // ❤ love
        ("vi", "yêu"),    ("vi", "tình yêu"),  ("vi", "thương"),
        ("en", "love"),   ("en", "affection"),
        ("fr", "amour"),  ("fr", "aimer"),
        ("de", "liebe"),  ("de", "mögen"),
        ("es", "amor"),   ("es", "querer"),    ("es", "encanta"),
        ("it", "amore"),  ("it", "amare"),
        ("zh", "爱"),     ("zh", "喜欢"),
        ("ja", "愛"),     ("ja", "好き"),
        ("pt", "amor"),
    ]),
    (0x2B50, &[  // ⭐ excellent
        ("vi", "tuyệt vời"), ("vi", "xuất sắc"), ("vi", "hoàn hảo"),
        ("en", "excellent"), ("en", "perfect"),  ("en", "amazing"),
        ("fr", "excellent"), ("fr", "parfait"),  ("fr", "magnifique"),
        ("de", "wunderbar"), ("de", "perfekt"),  ("de", "super"),
        ("es", "excelente"), ("es", "perfecto"), ("es", "guapa"),
        ("it", "ottimo"),    ("it", "perfetto"), ("it", "finalmente"),
        ("zh", "优秀"),      ("zh", "完美"),
        ("ja", "素晴らしい"),("ja", "完璧"),
        ("pt", "excelente"), ("pt", "ótimo"),
    ]),
    // ── Negative emotions ────────────────────────────────────────────────────
    (0x1F614, &[  // 😔 sadness
        ("vi", "buồn"),   ("vi", "buồn bã"),  ("vi", "u sầu"),
        ("en", "sad"),    ("en", "sorrow"),   ("en", "unhappy"),
        ("fr", "triste"), ("fr", "malheureux"),("fr", "disparaître"),
        ("de", "traurig"),("de", "unglücklich"),
        ("es", "triste"), ("es", "sola"),     ("es", "pobre"),
        ("it", "triste"), ("it", "peggio"),
        ("zh", "悲伤"),   ("zh", "难过"),
        ("ja", "悲しい"),
        ("pt", "triste"), ("pt", "infeliz"),
    ]),
    (0x1F4A9, &[  // 💩 terrible (reused for "bad")
        ("vi", "tệ"),     ("vi", "tồi tệ"),   ("vi", "kinh khủng"),
        ("en", "terrible"),("en", "awful"),   ("en", "worst"),  ("en", "sucks"),
        ("fr", "horrible"),("fr", "noire"),
        ("de", "schrecklich"),("de", "schlecht"),
        ("es", "peor"),   ("es", "malo"),
        ("it", "male"),
        ("zh", "糟糕"),   ("zh", "可怕"),
        ("ja", "最悪"),   ("ja", "ひどい"),
        ("pt", "terrível"),("pt", "horrível"),
    ]),
    (0x1F621, &[  // 😡 anger
        ("vi", "giận"),   ("vi", "tức"),      ("vi", "phẫn nộ"),
        ("en", "angry"),  ("en", "furious"),  ("en", "rage"),
        ("fr", "furieux"),("fr", "colère"),
        ("de", "wütend"), ("de", "ärger"),
        ("es", "enojado"),("es", "enfadado"),
        ("it", "arrabbiato"),
        ("zh", "愤怒"),   ("zh", "生气"),
        ("ja", "怒り"),   ("ja", "怒る"),
        ("pt", "raiva"),  ("pt", "furioso"),
    ]),
    (0x1F628, &[  // 😨 fear
        ("vi", "sợ"),     ("vi", "lo sợ"),    ("vi", "hoảng sợ"),
        ("en", "scared"), ("en", "afraid"),   ("en", "fear"),
        ("fr", "peur"),   ("fr", "effrayé"),
        ("de", "angst"),  ("de", "fürchten"),
        ("es", "miedo"),  ("es", "asustado"),
        ("it", "paura"),  ("it", "spavento"),
        ("zh", "恐惧"),   ("zh", "害怕"),
        ("ja", "恐怖"),   ("ja", "怖い"),
        ("pt", "medo"),   ("pt", "assustado"),
    ]),
    // ── Physical states ───────────────────────────────────────────────────────
    (0x1F634, &[  // 😴 tired
        ("vi", "mệt"),    ("vi", "mệt mỏi"),  ("vi", "kiệt sức"),
        ("en", "tired"),  ("en", "exhausted"),("en", "weary"),
        ("fr", "fatigué"),("fr", "épuisé"),
        ("de", "müde"),   ("de", "erschöpft"),
        ("es", "cansado"),("es", "agotado"),
        ("it", "stanco"), ("it", "esausto"),
        ("zh", "疲惫"),   ("zh", "累"),
        ("ja", "疲れた"), ("ja", "くたくた"),
        ("pt", "cansado"),("pt", "exausto"),
    ]),
    (0x1F915, &[  // 🤕 pain
        ("vi", "đau"),    ("vi", "đau đớn"),  ("vi", "đau khổ"),
        ("en", "pain"),   ("en", "hurt"),     ("en", "ache"),
        ("fr", "douleur"),("fr", "souffrir"),
        ("de", "schmerz"),("de", "wehtun"),
        ("es", "dolor"),  ("es", "doler"),
        ("it", "dolore"), ("it", "male"),
        ("zh", "痛"),     ("zh", "疼痛"),
        ("ja", "痛い"),   ("ja", "苦しい"),
        ("pt", "dor"),    ("pt", "doer"),
    ]),
    // ── Social / family ──────────────────────────────────────────────────────
    (0x1F46A, &[  // 👪 family
        ("vi", "gia đình"), ("vi", "nhà"),    ("vi", "bố mẹ"),
        ("en", "family"),   ("en", "home"),   ("en", "parents"),
        ("fr", "famille"),  ("fr", "maison"),
        ("de", "familie"),  ("de", "zuhause"),
        ("es", "familia"),  ("es", "hogar"),
        ("it", "famiglia"), ("it", "casa"),
        ("zh", "家庭"),     ("zh", "家人"),
        ("ja", "家族"),     ("ja", "家"),
        ("pt", "família"),  ("pt", "lar"),
    ]),
    // ── Nature / elements ────────────────────────────────────────────────────
    (0x1F30D, &[  // 🌍 earth / nature
        ("vi", "trái đất"), ("vi", "thiên nhiên"), ("vi", "đất"),
        ("en", "earth"),    ("en", "nature"),      ("en", "world"),
        ("fr", "terre"),    ("fr", "nature"),      ("fr", "monde"),
        ("de", "erde"),     ("de", "natur"),       ("de", "welt"),
        ("es", "tierra"),   ("es", "naturaleza"),  ("es", "mundo"),
        ("it", "terra"),    ("it", "natura"),      ("it", "mondo"),
        ("zh", "地球"),     ("zh", "自然"),
        ("ja", "地球"),     ("ja", "自然"),
        ("pt", "terra"),    ("pt", "natureza"),
    ]),
];

fn main() {
    println!("[l2_data] L2 Multilingual Seeder");
    println!("[l2_data] {} L2 concept nodes", L2_NODES.len());

    if ucd::table_len() == 0 {
        eprintln!("[l2_data] ERROR: UCD empty");
        std::process::exit(1);
    }

    let ts   = now_ns();
    let seed = [0x4Cu8; 32]; // L2 seed
    let signer = QRSigner::from_seed(&seed);

    let mut writer   = OlangWriter::new(ts);
    let mut registry = Registry::new();

    let mut node_count  = 0usize;
    let mut alias_count = 0usize;
    let mut lang_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();

    // Load existing origin.olang nếu có
    let existing = fs::read("origin.olang").unwrap_or_default();
    if !existing.is_empty() {
        if let Ok(reader) = olang::reader::OlangReader::new(&existing) {
            if let Ok(parsed) = reader.parse_all() {
                for node in &parsed.nodes {
                    registry.insert(&node.chain, node.layer, node.file_offset, node.timestamp, node.is_qr);
                    writer.append_node(&node.chain, node.layer, node.is_qr, node.timestamp).ok();
                }
                for alias in &parsed.aliases {
                    registry.register_alias(&alias.name, alias.chain_hash);
                    writer.append_alias(&alias.name, alias.chain_hash, alias.timestamp).ok();
                }
                println!("[l2_data] Loaded origin.olang: {} nodes, {} aliases",
                    parsed.node_count(), parsed.alias_count());
            }
        }
    }

    // Seed L2 nodes
    for &(emoji_cp, translations) in L2_NODES {
        let chain = encode_codepoint(emoji_cp);
        let hash  = chain.chain_hash();
        let qr    = signer.sign_qr(&chain, ts);
        if !signer.verify(&qr) { continue; }

        writer.append_node(&chain, 2, true, ts).ok(); // Layer 2
        registry.insert(&chain, 2, 0, ts, true);

        // Emoji as alias
        let emoji_str: String = char::from_u32(emoji_cp)
            .map(|c| c.to_string()).unwrap_or_default();
        if !emoji_str.is_empty() {
            registry.register_alias(&emoji_str, hash);
            writer.append_alias(&emoji_str, hash, ts).ok();
        }

        // All translations as aliases
        for &(lang, word) in translations {
            registry.register_alias(word, hash);
            writer.append_alias(word, hash, ts).ok();
            *lang_counts.entry(lang).or_insert(0) += 1;
            alias_count += 1;
        }

        println!("[l2_data] ✓ U+{:05X} — {} translations", emoji_cp, translations.len());
        node_count += 1;
    }

    println!();
    println!("[l2_data] Nodes   : {}", node_count);
    println!("[l2_data] Aliases : {}", alias_count);
    println!("[l2_data] Languages:");
    let mut lang_vec: Vec<_> = lang_counts.iter().collect();
    lang_vec.sort_by_key(|&(&k, _)| k);
    for (lang, count) in lang_vec {
        println!("[l2_data]   {}: {} words", lang, count);
    }

    // Write origin.olang
    let bytes = writer.into_bytes();
    fs::write("origin.olang", &bytes).expect("write origin.olang");

    // Verify
    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!();
    println!("[l2_data] ✓ origin.olang: {} nodes, {} aliases, {} bytes",
        parsed.node_count(), parsed.alias_count(), bytes.len());
    println!("[l2_data] Done ✓  ○ L2 loaded");
}
