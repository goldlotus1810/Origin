//! # multilang_seeder — Seed cross-language aliases
//!
//! Dùng EN-VI parallel pairs từ paper + multilingual sentiment lexicon
//! để tạo alias cross-language trong Registry.
//!
//! "good" ≡ "tốt" ≡ "bien" ≡ "gut" → cùng node trong KnowTree

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::log::{EventLog, LogEvent};
use olang::qr::QRSigner;
use olang::registry::Registry;
use olang::writer::OlangWriter;

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

// ─────────────────────────────────────────────────────────────────────────────
// Sentiment Lexicon: (translations..., valence, arousal)
// ─────────────────────────────────────────────────────────────────────────────
//
// Mỗi entry = nhiều từ CÙNG NGHĨA trong nhiều ngôn ngữ
// → tất cả trỏ về cùng 1 node (alias của nhau)
// → node được tạo từ LCA(encode_codepoint(emoji đại diện))

static SENTIMENT_NODES: &[(&str, u32, &[&str])] = &[
    // (name, representative_emoji_cp, aliases_in_multiple_languages)
    (
        "joy",
        0x1F60A,
        &[
            // Vietnamese
            "vui",
            "hạnh phúc",
            "vui vẻ",
            "vui mừng",
            // English
            "happy",
            "joy",
            "joyful",
            "glad",
            "pleased",
            "delighted",
            // French
            "heureux",
            "joie",
            "joyeux",
            "content",
            // German
            "glücklich",
            "fröhlich",
            "freudig",
            // Chinese
            "快乐",
            "高兴",
            "开心",
            // Japanese
            "嬉しい",
            "楽しい",
            // Spanish
            "feliz",
            "alegre",
            "contento",
        ],
    ),
    (
        "sadness",
        0x1F614,
        &[
            "buồn",
            "buồn bã",
            "đau khổ",
            "u sầu",
            "sad",
            "unhappy",
            "sorrowful",
            "miserable",
            "depressed",
            "triste",
            "malheureux",
            "mélancolique",
            "traurig",
            "unglücklich",
            "melancholisch",
            "悲しい",
            "悲しむ",
            "悲伤",
            "难过",
            "伤心",
            "triste",
            "infeliz",
            "melancólico",
        ],
    ),
    (
        "anger",
        0x1F621,
        &[
            "giận",
            "tức",
            "bực bội",
            "phẫn nộ",
            "angry",
            "furious",
            "rage",
            "mad",
            "irritated",
            "en colère",
            "furieux",
            "irrité",
            "wütend",
            "verärgert",
            "zornig",
            "怒り",
            "激怒",
            "愤怒",
            "生气",
            "恼火",
            "enojado",
            "furioso",
            "irado",
        ],
    ),
    (
        "fear",
        0x1F628,
        &[
            "sợ",
            "lo sợ",
            "hoảng sợ",
            "kinh hãi",
            "scared",
            "afraid",
            "fearful",
            "terrified",
            "anxious",
            "peur",
            "effrayé",
            "terrifié",
            "anxieux",
            "Angst",
            "fürchten",
            "erschrocken",
            "恐怖",
            "怖い",
            "恐れる",
            "miedo",
            "asustado",
            "aterrorizado",
        ],
    ),
    (
        "surprise",
        0x1F632,
        &[
            "ngạc nhiên",
            "bất ngờ",
            "kinh ngạc",
            "surprised",
            "astonished",
            "amazed",
            "shocked",
            "surpris",
            "étonné",
            "stupéfait",
            "überrascht",
            "erstaunt",
            "verblüfft",
            "驚く",
            "びっくり",
            "惊讶",
            "震惊",
            "诧异",
            "sorprendido",
            "asombrado",
        ],
    ),
    (
        "disgust",
        0x1F922,
        &[
            "ghê",
            "ghê tởm",
            "kinh tởm",
            "disgusted",
            "revolted",
            "repulsed",
            "dégoûté",
            "révoltant",
            "ekelhaft",
            "widerlich",
            "嫌悪",
            "気持ち悪い",
            "恶心",
            "厌恶",
            "asqueroso",
            "repugnante",
        ],
    ),
    (
        "love",
        0x2764,
        &[
            "yêu",
            "yêu thương",
            "tình yêu",
            "thương",
            "love",
            "affection",
            "adore",
            "cherish",
            "amour",
            "aimer",
            "chérir",
            "Liebe",
            "lieben",
            "mögen",
            "愛",
            "愛する",
            "爱",
            "喜欢",
            "爱情",
            "amor",
            "querer",
            "amar",
        ],
    ),
    (
        "excellent",
        0x2B50,
        &[
            "tuyệt vời",
            "xuất sắc",
            "tốt lắm",
            "hoàn hảo",
            "excellent",
            "outstanding",
            "superb",
            "perfect",
            "great",
            "excellent",
            "magnifique",
            "formidable",
            "parfait",
            "ausgezeichnet",
            "hervorragend",
            "wunderbar",
            "perfekt",
            "素晴らしい",
            "完璧",
            "优秀",
            "完美",
            "出色",
            "excelente",
            "perfecto",
            "magnífico",
        ],
    ),
    (
        "terrible",
        0x1F4A9,
        &[
            "tệ",
            "tồi tệ",
            "kinh khủng",
            "thảm họa",
            "terrible",
            "awful",
            "horrible",
            "dreadful",
            "appalling",
            "terrible",
            "horrible",
            "affreux",
            "catastrophique",
            "schrecklich",
            "furchtbar",
            "katastrophal",
            "ひどい",
            "最悪",
            "糟糕",
            "可怕",
            "恐怖",
            "terrible",
            "horrible",
            "espantoso",
        ],
    ),
    (
        "tired",
        0x1F634,
        &[
            "mệt",
            "kiệt sức",
            "mệt mỏi",
            "tired",
            "exhausted",
            "weary",
            "fatigued",
            "fatigué",
            "épuisé",
            "exténué",
            "müde",
            "erschöpft",
            "疲れた",
            "くたくた",
            "疲惫",
            "疲倦",
            "cansado",
            "agotado",
        ],
    ),
];

/// Basic concept nodes — common cross-language concepts mapped to Unicode codepoints.
static CONCEPT_NODES: &[(&str, u32, &[&str])] = &[
    (
        "water",
        0x1F4A7, // 💧
        &[
            "nước", "nước uống",
            "water", "aqua",
            "eau", "l'eau",
            "Wasser",
            "水", "みず",
            "agua",
        ],
    ),
    (
        "fire",
        0x1F525, // 🔥
        &[
            "lửa", "ngọn lửa",
            "fire", "flame", "blaze",
            "feu", "flamme",
            "Feuer", "Flamme",
            "火", "炎",
            "fuego", "llama",
        ],
    ),
    (
        "sun",
        0x2600, // ☀
        &[
            "mặt trời", "nắng", "ánh sáng",
            "sun", "sunlight", "sunshine",
            "soleil", "lumière",
            "Sonne", "Sonnenlicht",
            "太陽", "日", "たいよう",
            "sol", "luz solar",
        ],
    ),
    (
        "moon",
        0x1F319, // 🌙
        &[
            "trăng", "mặt trăng", "ánh trăng",
            "moon", "moonlight", "lunar",
            "lune", "clair de lune",
            "Mond", "Mondlicht",
            "月", "つき",
            "luna", "luz de luna",
        ],
    ),
    (
        "earth",
        0x1F30D, // 🌍
        &[
            "trái đất", "địa cầu", "đất",
            "earth", "world", "globe",
            "terre", "monde",
            "Erde", "Welt",
            "地球", "世界", "ちきゅう",
            "tierra", "mundo",
        ],
    ),
    (
        "wind",
        0x1F4A8, // 💨
        &[
            "gió", "cơn gió",
            "wind", "breeze", "gust",
            "vent", "brise",
            "Wind", "Brise",
            "風", "かぜ",
            "viento", "brisa",
        ],
    ),
    (
        "tree",
        0x1F333, // 🌳
        &[
            "cây", "cây cối",
            "tree", "plant",
            "arbre", "plante",
            "Baum", "Pflanze",
            "木", "樹", "き",
            "árbol", "planta",
        ],
    ),
    (
        "mountain",
        0x26F0, // ⛰
        &[
            "núi", "ngọn núi",
            "mountain", "hill", "peak",
            "montagne", "colline",
            "Berg", "Gipfel",
            "山", "やま",
            "montaña", "cerro",
        ],
    ),
    (
        "ocean",
        0x1F30A, // 🌊
        &[
            "biển", "đại dương", "sóng",
            "ocean", "sea", "wave",
            "océan", "mer", "vague",
            "Ozean", "Meer", "Welle",
            "海", "波", "うみ",
            "océano", "mar", "ola",
        ],
    ),
    (
        "star",
        0x2B50, // ⭐ (shared with "excellent" — same node, different aliases)
        &[
            "ngôi sao", "sao",
            "star", "stellar",
            "étoile", "stellaire",
            "Stern", "stellar",
            "星", "ほし",
            "estrella", "estelar",
        ],
    ),
    (
        "heart",
        0x2764, // ❤ (shared with "love" — same node, different aliases)
        &[
            "trái tim", "tim",
            "heart",
            "cœur",
            "Herz",
            "心", "こころ",
            "corazón",
        ],
    ),
    (
        "home",
        0x1F3E0, // 🏠
        &[
            "nhà", "căn nhà", "gia đình",
            "home", "house", "family",
            "maison", "foyer", "famille",
            "Haus", "Heim", "Familie",
            "家", "いえ", "家族",
            "casa", "hogar", "familia",
        ],
    ),
    (
        "food",
        0x1F35E, // 🍞
        &[
            "thức ăn", "đồ ăn", "thực phẩm",
            "food", "meal", "nourishment",
            "nourriture", "repas", "aliment",
            "Essen", "Nahrung", "Mahlzeit",
            "食べ物", "食事", "たべもの",
            "comida", "alimento",
        ],
    ),
    (
        "music",
        0x1F3B5, // 🎵
        &[
            "nhạc", "âm nhạc", "bài hát",
            "music", "song", "melody",
            "musique", "chanson", "mélodie",
            "Musik", "Lied", "Melodie",
            "音楽", "歌", "おんがく",
            "música", "canción", "melodía",
        ],
    ),
    (
        "time",
        0x23F0, // ⏰
        &[
            "thời gian", "giờ",
            "time", "hour", "moment",
            "temps", "heure", "moment",
            "Zeit", "Stunde", "Moment",
            "時間", "時", "じかん",
            "tiempo", "hora", "momento",
        ],
    ),
];

fn main() {
    println!("[multilang] Cross-Language Alias Seeder");
    println!(
        "[multilang] {} sentiment + {} concept nodes → multilingual aliases",
        SENTIMENT_NODES.len(),
        CONCEPT_NODES.len(),
    );

    let ts = now_ns();
    let seed = [0x42u8; 32];
    let signer = QRSigner::from_seed(&seed);

    // Append to existing origin.olang
    let mut writer = if let Ok(existing) = fs::read("origin.olang") {
        println!(
            "[multilang] Loading existing origin.olang ({} bytes)",
            existing.len()
        );
        OlangWriter::from_existing(existing)
    } else {
        println!("[multilang] No existing file — creating new origin.olang");
        OlangWriter::new(ts)
    };

    let mut registry = Registry::new();
    let mut log = EventLog::new(String::from("origin.olang.log"));

    let mut node_count = 0usize;
    let mut alias_count = 0usize;

    // Seed both sentiment and concept nodes
    let all_nodes: Vec<(&str, u32, &[&str])> = SENTIMENT_NODES
        .iter()
        .chain(CONCEPT_NODES.iter())
        .copied()
        .collect();

    for &(name, emoji_cp, aliases) in &all_nodes {
        // Node = encode từ emoji đại diện (QR — bất biến)
        let chain = encode_codepoint(emoji_cp);
        let hash = chain.chain_hash();
        let qr = signer.sign_qr(&chain, ts);

        if !signer.verify(&qr) {
            eprintln!("[multilang] WARN: verify failed: {}", name);
            continue;
        }

        // Ghi file (QT8)
        let offset = match writer.append_node(&chain, 1, true, ts) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("write error: {:?}", e);
                continue;
            }
        };

        // Registry
        registry.insert(&chain, 1, offset, ts, true);
        registry.register_alias(name, hash);
        writer.append_alias(name, hash, ts).ok();

        // Tất cả aliases → cùng node
        for &alias in aliases {
            registry.register_alias(alias, hash);
            writer.append_alias(alias, hash, ts).ok();
            alias_count += 1;
        }

        log.append(LogEvent::NodeCreated {
            chain_hash: hash,
            layer: 1,
            file_offset: offset,
            timestamp: ts,
        });

        println!(
            "[multilang] ✓ {} (U+{:05X}) — {} aliases",
            name,
            emoji_cp,
            aliases.len()
        );
        node_count += 1;
    }

    println!();
    println!("[multilang] Nodes   : {}", node_count);
    println!("[multilang] Aliases : {}", alias_count);
    println!("[multilang] Registry: {}", registry.len());
    println!("[multilang] File    : {} bytes", writer.size());

    // Ghi file — append vào origin.olang
    let bytes = writer.as_bytes().to_vec();
    fs::write("origin.olang", &bytes).expect("write origin.olang");

    // Verify
    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!(
        "[multilang] ✓ Roundtrip: {} nodes, {} aliases",
        parsed.node_count(),
        parsed.alias_count()
    );
    println!("[multilang] Done ✓");
    println!("[multilang] ○ vui ≡ happy ≡ heureux ≡ glücklich ≡ 快乐 ≡ 嬉しい");
}
