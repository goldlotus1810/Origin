//! # l3_topics — Seed KnowTree L3 Topic Clusters
//!
//! L3 = tầng topic/domain — "éolienne" ∈ cluster "renewable_energy" → positive
//!
//! Cách hoạt động:
//!   Mỗi topic cluster = 1 L3 node với valence/arousal prototype
//!   Tất cả từ trong cluster → alias của node đó
//!   word_affect lookup: "éolienne" → L3_node → valence=+0.6

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::qr::QRSigner;
use olang::registry::Registry;
use olang::writer::OlangWriter;

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

/// L3 topic clusters: (emoji_cp, valence, arousal, &[words...])
/// Extracted từ multilingual corpus analysis + domain knowledge
static L3_CLUSTERS: &[(u32, f32, f32, &[&str])] = &[
    // ── Renewable energy / environment (POSITIVE) ─────────────────────────
    (
        0x2600,
        0.60,
        0.45,
        &[
            // ☀ solar/renewable
            // French (from corpus: solaire 21x positive)
            "solaire",
            "éolienne",
            "éoliennes",
            "photovoltaïque",
            "renouvelable",
            "renouvelables",
            "hydroélectrique",
            "écologique",
            "écologiques",
            "durable",
            "durables",
            "stocke",
            "énergies",
            // German
            "solar",
            "erneuerbar",
            "nachhaltig",
            "windenergie",
            "ökologisch",
            "umweltfreundlich",
            // Spanish
            "solar",
            "renovable",
            "renovables",
            "ecológico",
            "sostenible",
            "energía",
            "limpia",
            // English
            "solar",
            "renewable",
            "sustainable",
            "ecological",
            "green",
            "windmill",
            "photovoltaic",
            // Vietnamese
            "năng lượng mặt trời",
            "tái tạo",
            "sinh thái",
            "bền vững",
        ],
    ),
    // ── Technology / innovation (POSITIVE) ────────────────────────────────
    (
        0x1F4BB,
        0.55,
        0.60,
        &[
            // 💻 tech
            // French (crowdfunding 8x positive)
            "crowdfunding",
            "startup",
            "innovation",
            "technologie",
            "numérique",
            "digital",
            "algorithme",
            "données",
            "cellule",
            "prototype",
            "développement",
            // German
            "technologie",
            "innovation",
            "digitalisierung",
            "startup",
            "entwicklung",
            "software",
            "plattform",
            // Spanish
            "tecnología",
            "innovación",
            "digital",
            "startup",
            "desarrollo",
            "plataforma",
            "aplicación",
            // English
            "technology",
            "innovation",
            "digital",
            "software",
            "platform",
            "algorithm",
            "development",
            // Vietnamese
            "công nghệ",
            "đổi mới",
            "kỹ thuật số",
            "phần mềm",
        ],
    ),
    // ── Finance / investment (NEUTRAL-POSITIVE) ────────────────────────────
    (
        0x1F4B0,
        0.30,
        0.50,
        &[
            // 💰 money
            // French (banques 10x positive in corpus)
            "banques",
            "banque",
            "financement",
            "investissement",
            "économique",
            "financier",
            "marché",
            "capital",
            "budget",
            "fonds",
            "euros",
            "subvention",
            // German
            "bank",
            "finanzierung",
            "investition",
            "wirtschaft",
            "markt",
            "kapital",
            "haushalt",
            // Spanish
            "banco",
            "financiación",
            "inversión",
            "economía",
            "mercado",
            "capital",
            "presupuesto",
            // English
            "bank",
            "finance",
            "investment",
            "economy",
            "market",
            "capital",
            "budget",
            "fund",
        ],
    ),
    // ── Crisis / threat (NEGATIVE) ────────────────────────────────────────
    (
        0x26A0,
        -0.65,
        0.75,
        &[
            // ⚠ danger
            // French (menacées 11x neg, océans 11x neg)
            "menacées",
            "menacés",
            "crise",
            "catastrophe",
            "désastre",
            "polémique",
            "scandale",
            "rejet",
            "rejette",
            "extinction",
            "disparaître",
            "dégradation",
            "pollution",
            "contamination",
            // German
            "krise",
            "katastrophe",
            "gefahr",
            "bedrohung",
            "skandal",
            "ablehnung",
            "aussterben",
            // Spanish
            "crisis",
            "catástrofe",
            "amenaza",
            "escándalo",
            "rechazo",
            "extinción",
            "contaminación",
            // English
            "crisis",
            "catastrophe",
            "threat",
            "scandal",
            "rejection",
            "extinction",
            "contamination",
            // Vietnamese
            "khủng hoảng",
            "thảm họa",
            "đe dọa",
            "ô nhiễm",
        ],
    ),
    // ── Politics / conflict (NEGATIVE) ────────────────────────────────────
    (
        0x1F3DB,
        -0.40,
        0.65,
        &[
            // 🏛 politics
            // French (communistes 9x neg, canada 12x neg in context)
            "communistes",
            "polémique",
            "opposition",
            "conflit",
            "guerre",
            "attaque",
            "protestation",
            "manifestation",
            "censure",
            "corruption",
            "fraude",
            // German
            "konflikt",
            "krieg",
            "angriff",
            "protest",
            "zensur",
            "korruption",
            "betrug",
            // Spanish
            "conflicto",
            "guerra",
            "ataque",
            "protesta",
            "censura",
            "corrupción",
            "fraude",
            // English
            "conflict",
            "war",
            "attack",
            "protest",
            "censorship",
            "corruption",
            "fraud",
            // Vietnamese
            "xung đột",
            "chiến tranh",
            "tham nhũng",
            "gian lận",
        ],
    ),
    // ── Health / medical (NEUTRAL) ────────────────────────────────────────
    (
        0x1F3E5,
        0.10,
        0.55,
        &[
            // 🏥 health
            "hôpital",
            "médecin",
            "santé",
            "traitement",
            "maladie",
            "vaccin",
            "médicament",
            "chirurgie",
            "diagnostic",
            "krankenhaus",
            "arzt",
            "gesundheit",
            "behandlung",
            "krankheit",
            "impfung",
            "medikament",
            "hospital",
            "médico",
            "salud",
            "tratamiento",
            "enfermedad",
            "vacuna",
            "medicamento",
            "hospital",
            "doctor",
            "health",
            "treatment",
            "disease",
            "vaccine",
            "medicine",
            "bệnh viện",
            "bác sĩ",
            "sức khỏe",
            "điều trị",
        ],
    ),
    // ── Sports / competition (POSITIVE) ───────────────────────────────────
    (
        0x1F3C6,
        0.65,
        0.80,
        &[
            // 🏆 trophy
            "sport",
            "victoire",
            "champion",
            "compétition",
            "tournoi",
            "équipe",
            "match",
            "gagner",
            "gagné",
            "sport",
            "sieg",
            "meister",
            "wettbewerb",
            "turnier",
            "mannschaft",
            "gewonnen",
            "deporte",
            "victoria",
            "campeón",
            "competición",
            "torneo",
            "equipo",
            "ganar",
            "sport",
            "victory",
            "champion",
            "competition",
            "tournament",
            "team",
            "win",
            "thể thao",
            "chiến thắng",
            "vô địch",
        ],
    ),
    // ── Education / science (POSITIVE) ────────────────────────────────────
    (
        0x1F393,
        0.60,
        0.50,
        &[
            // 🎓 graduation
            "université",
            "recherche",
            "scientifique",
            "étude",
            "découverte",
            "publication",
            "conférence",
            "formation",
            "universität",
            "forschung",
            "wissenschaft",
            "studie",
            "entdeckung",
            "publikation",
            "ausbildung",
            "universidad",
            "investigación",
            "científico",
            "estudio",
            "descubrimiento",
            "publicación",
            "formación",
            "university",
            "research",
            "science",
            "study",
            "discovery",
            "publication",
            "education",
            "đại học",
            "nghiên cứu",
            "khoa học",
            "giáo dục",
        ],
    ),
];

fn main() {
    println!("[l3_topics] L3 Topic Cluster Seeder");
    println!("[l3_topics] {} topic clusters", L3_CLUSTERS.len());

    if ucd::table_len() == 0 {
        eprintln!("[l3_topics] ERROR: UCD empty");
        std::process::exit(1);
    }

    let ts = now_ns();
    let seed = [0x4Cu8; 32];
    let signer = QRSigner::from_seed(&seed);

    let mut writer = OlangWriter::new(ts);
    let mut registry = Registry::new();

    // Load existing origin.olang
    let existing = fs::read("origin.olang").unwrap_or_default();
    if !existing.is_empty() {
        if let Ok(reader) = olang::reader::OlangReader::new(&existing) {
            if let Ok(parsed) = reader.parse_all() {
                for node in &parsed.nodes {
                    registry.insert(
                        &node.chain,
                        node.layer,
                        node.file_offset,
                        node.timestamp,
                        node.is_qr,
                    );
                    writer
                        .append_node(&node.chain, node.layer, node.is_qr, node.timestamp)
                        .ok();
                }
                for alias in &parsed.aliases {
                    registry.register_alias(&alias.name, alias.chain_hash);
                    writer
                        .append_alias(&alias.name, alias.chain_hash, alias.timestamp)
                        .ok();
                }
                println!(
                    "[l3_topics] Loaded: {} nodes, {} aliases",
                    parsed.node_count(),
                    parsed.alias_count()
                );
            }
        }
    }

    let mut node_count = 0usize;
    let mut alias_count = 0usize;

    for &(emoji_cp, _valence, _arousal, words) in L3_CLUSTERS {
        let chain = encode_codepoint(emoji_cp);
        let hash = chain.chain_hash();
        let qr = signer.sign_qr(&chain, ts);
        if !signer.verify(&qr) {
            continue;
        }

        writer.append_node(&chain, 3, true, ts).ok(); // Layer 3, QR
        registry.insert(&chain, 3, 0, ts, true);

        // Emoji alias
        if let Some(c) = char::from_u32(emoji_cp) {
            let emoji_s = c.to_string();
            registry.register_alias(&emoji_s, hash);
            writer.append_alias(&emoji_s, hash, ts).ok();
        }

        // All topic words → aliases
        for &word in words {
            registry.register_alias(word, hash);
            writer.append_alias(word, hash, ts).ok();
            alias_count += 1;
        }

        println!("[l3_topics] ✓ U+{:05X} — {} words", emoji_cp, words.len());
        node_count += 1;
    }

    println!();
    println!("[l3_topics] Nodes   : {}", node_count);
    println!("[l3_topics] Aliases : {}", alias_count);

    let bytes = writer.into_bytes();
    fs::write("origin.olang", &bytes).expect("write");

    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!(
        "[l3_topics] ✓ origin.olang: {} nodes, {} aliases, {} bytes",
        parsed.node_count(),
        parsed.alias_count(),
        bytes.len()
    );
    println!("[l3_topics] Done ✓");
}
