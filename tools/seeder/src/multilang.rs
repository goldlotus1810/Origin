//! # multilang_seeder — Seed cross-language aliases
//!
//! Dùng EN-VI parallel pairs từ paper + multilingual sentiment lexicon
//! để tạo alias cross-language trong Registry.
//!
//! "good" ≡ "tốt" ≡ "bien" ≡ "gut" → cùng node trong KnowTree

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::lca::lca;
use olang::qr::QRSigner;
use olang::writer::OlangWriter;
use olang::registry::Registry;
use olang::log::{EventLog, LogEvent};

fn now_ns() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64
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
    ("joy", 0x1F60A, &[
        // Vietnamese
        "vui", "hạnh phúc", "vui vẻ", "vui mừng",
        // English
        "happy", "joy", "joyful", "glad", "pleased", "delighted",
        // French
        "heureux", "joie", "joyeux", "content",
        // German
        "glücklich", "fröhlich", "freudig",
        // Chinese
        "快乐", "高兴", "开心",
        // Japanese
        "嬉しい", "楽しい",
        // Spanish
        "feliz", "alegre", "contento",
    ]),
    ("sadness", 0x1F614, &[
        "buồn", "buồn bã", "đau khổ", "u sầu",
        "sad", "unhappy", "sorrowful", "miserable", "depressed",
        "triste", "malheureux", "mélancolique",
        "traurig", "unglücklich", "melancholisch",
        "悲しい", "悲しむ",
        "悲伤", "难过", "伤心",
        "triste", "infeliz", "melancólico",
    ]),
    ("anger", 0x1F621, &[
        "giận", "tức", "bực bội", "phẫn nộ",
        "angry", "furious", "rage", "mad", "irritated",
        "en colère", "furieux", "irrité",
        "wütend", "verärgert", "zornig",
        "怒り", "激怒",
        "愤怒", "生气", "恼火",
        "enojado", "furioso", "irado",
    ]),
    ("fear", 0x1F628, &[
        "sợ", "lo sợ", "hoảng sợ", "kinh hãi",
        "scared", "afraid", "fearful", "terrified", "anxious",
        "peur", "effrayé", "terrifié", "anxieux",
        "Angst", "fürchten", "erschrocken",
        "恐怖", "怖い", "恐れる",
        "miedo", "asustado", "aterrorizado",
    ]),
    ("surprise", 0x1F632, &[
        "ngạc nhiên", "bất ngờ", "kinh ngạc",
        "surprised", "astonished", "amazed", "shocked",
        "surpris", "étonné", "stupéfait",
        "überrascht", "erstaunt", "verblüfft",
        "驚く", "びっくり",
        "惊讶", "震惊", "诧异",
        "sorprendido", "asombrado",
    ]),
    ("disgust", 0x1F922, &[
        "ghê", "ghê tởm", "kinh tởm",
        "disgusted", "revolted", "repulsed",
        "dégoûté", "révoltant",
        "ekelhaft", "widerlich",
        "嫌悪", "気持ち悪い",
        "恶心", "厌恶",
        "asqueroso", "repugnante",
    ]),
    ("love", 0x2764, &[
        "yêu", "yêu thương", "tình yêu", "thương",
        "love", "affection", "adore", "cherish",
        "amour", "aimer", "chérir",
        "Liebe", "lieben", "mögen",
        "愛", "愛する",
        "爱", "喜欢", "爱情",
        "amor", "querer", "amar",
    ]),
    ("excellent", 0x2B50, &[
        "tuyệt vời", "xuất sắc", "tốt lắm", "hoàn hảo",
        "excellent", "outstanding", "superb", "perfect", "great",
        "excellent", "magnifique", "formidable", "parfait",
        "ausgezeichnet", "hervorragend", "wunderbar", "perfekt",
        "素晴らしい", "完璧",
        "优秀", "完美", "出色",
        "excelente", "perfecto", "magnífico",
    ]),
    ("terrible", 0x1F4A9, &[
        "tệ", "tồi tệ", "kinh khủng", "thảm họa",
        "terrible", "awful", "horrible", "dreadful", "appalling",
        "terrible", "horrible", "affreux", "catastrophique",
        "schrecklich", "furchtbar", "katastrophal",
        "ひどい", "最悪",
        "糟糕", "可怕", "恐怖",
        "terrible", "horrible", "espantoso",
    ]),
    ("tired", 0x1F634, &[
        "mệt", "kiệt sức", "mệt mỏi",
        "tired", "exhausted", "weary", "fatigued",
        "fatigué", "épuisé", "exténué",
        "müde", "erschöpft",
        "疲れた", "くたくた",
        "疲惫", "疲倦",
        "cansado", "agotado",
    ]),
];

fn main() {
    println!("[multilang] Cross-Language Alias Seeder");
    println!("[multilang] {} semantic nodes → multilingual aliases", SENTIMENT_NODES.len());

    if ucd::table_len() == 0 {
        eprintln!("[multilang] ERROR: UCD table empty");
        std::process::exit(1);
    }

    let ts = now_ns();
    let seed = [0x42u8; 32];
    let signer = QRSigner::from_seed(&seed);

    let mut writer   = OlangWriter::new(ts);
    let mut registry = Registry::new();
    let mut log      = EventLog::new(String::from("multilang.olang.log"));

    let mut node_count  = 0usize;
    let mut alias_count = 0usize;

    for &(name, emoji_cp, aliases) in SENTIMENT_NODES {
        // Node = encode từ emoji đại diện (QR — bất biến)
        let chain = encode_codepoint(emoji_cp);
        let hash  = chain.chain_hash();
        let qr    = signer.sign_qr(&chain, ts);

        if !signer.verify(&qr) {
            eprintln!("[multilang] WARN: verify failed: {}", name);
            continue;
        }

        // Ghi file (QT8)
        let offset = match writer.append_node(&chain, 1, true, ts) {
            Ok(o)  => o,
            Err(e) => { eprintln!("write error: {:?}", e); continue; }
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
            chain_hash: hash, layer: 1,
            file_offset: offset, timestamp: ts,
        });

        println!("[multilang] ✓ {} (U+{:05X}) — {} aliases", name, emoji_cp, aliases.len());
        node_count += 1;
    }

    println!();
    println!("[multilang] Nodes   : {}", node_count);
    println!("[multilang] Aliases : {}", alias_count);
    println!("[multilang] Registry: {}", registry.len());
    println!("[multilang] File    : {} bytes", writer.size());

    // Ghi file
    let bytes = writer.as_bytes().to_vec();
    fs::write("multilang.olang", &bytes).expect("write multilang.olang");

    // Verify
    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!("[multilang] ✓ Roundtrip: {} nodes, {} aliases", parsed.node_count(), parsed.alias_count());
    println!("[multilang] Done ✓");
    println!("[multilang] ○ vui ≡ happy ≡ heureux ≡ glücklich ≡ 快乐");
}
