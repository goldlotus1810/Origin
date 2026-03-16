//! # seeder — Seed L0 từ UCD
//!
//! Tạo origin.olang với L0 nodes từ Unicode 18.0.
//! KHÔNG có presets. KHÔNG có ISL hardcode.
//! Mọi chain từ encode_codepoint(cp).

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::log::{EventLog, LogEvent};
use olang::qr::QRSigner;
use olang::registry::Registry;
use olang::writer::OlangWriter;

/// L0 Seed Map: (tên, codepoint, aliases)
/// Chain KHÔNG hardcode — đến từ encode_codepoint(cp).
static L0_NODES: &[(&str, u32, &[&str])] = &[
    ("fire", 0x1F525, &["lua", "lửa", "fire", "feu"]),
    ("light", 0x1F4A1, &["anh-sang", "light"]),
    ("spark", 0x2728, &["tia-lua", "spark"]),
    ("bolt", 0x26A1, &["set", "lightning"]),
    ("water", 0x1F4A7, &["nuoc", "nước", "water", "eau"]),
    ("earth", 0x1F30D, &["dat", "earth"]),
    ("wind", 0x1F32C, &["gio", "wind"]),
    ("sound", 0x1F50A, &["am-thanh", "sound"]),
    ("cold", 0x2744, &["lanh", "cold"]),
    ("warm", 0x1F31E, &["am", "warm"]),
    ("sun", 0x2600, &["mat-troi", "sun"]),
    ("pain", 0x1F915, &["dau", "pain"]),
    ("joy", 0x1F60C, &["vui", "joy"]),
    ("hunger", 0x1F374, &["doi", "hunger"]),
    ("fatigue", 0x1F634, &["met", "tired"]),
    ("danger", 0x26A0, &["nguy-hiem", "danger"]),
    ("dark", 0x1F311, &["toi", "dark"]),
    ("alert", 0x1F6A8, &["canh-bao", "alert"]),
    ("shelter", 0x1F3E0, &["nha", "shelter", "home"]),
    ("house", 0x1F3E1, &["nha-o", "house"]),
    ("nature", 0x1F333, &["thien-nhien", "nature"]),
    ("ocean", 0x1F30A, &["bien", "ocean", "sea"]),
    ("mind", 0x1F9E0, &["tam-tri", "tâm trí", "mind", "brain"]),
    ("person", 0x1F464, &["nguoi", "person"]),
    ("eye", 0x1F441, &["mat", "eye"]),
    ("heart", 0x2764, &["tim", "trái tim", "heart"]),
    ("yes", 0x2705, &["co", "yes", "true"]),
    ("no", 0x274C, &["khong", "no", "false"]),
    ("now", 0x23F0, &["bay-gio", "now"]),
    ("all", 0x267E, &["tat-ca", "all"]),
    ("move", 0x1F3C3, &["di-chuyen", "move"]),
    ("stop", 0x1F6D1, &["dung", "stop"]),
    ("open", 0x1F513, &["mo", "open"]),
    ("close", 0x1F512, &["dong", "close"]),
    ("origin", 0x25CB, &["nguon-goc", "origin"]),
];

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

fn main() {
    println!("[seeder] HomeOS Origin Seeder");
    println!("[seeder] Unicode 18.0 — {} L0 nodes", L0_NODES.len());

    if ucd::table_len() == 0 {
        eprintln!("[seeder] ERROR: UCD table empty");
        std::process::exit(1);
    }
    println!("[seeder] UCD: {} entries", ucd::table_len());

    let ts = now_ns();
    let seed = [0x42u8; 32];
    let signer = QRSigner::from_seed(&seed);

    let mut writer = OlangWriter::new(ts);
    let mut registry = Registry::new();
    let mut log = EventLog::new(String::from("origin.olang.log"));

    let mut count = 0usize;
    let mut failed = 0usize;

    for &(name, cp, aliases) in L0_NODES {
        let chain = encode_codepoint(cp);
        let hash = chain.chain_hash();
        let qr = signer.sign_qr(&chain, ts);

        if !signer.verify(&qr) {
            eprintln!("[seeder] WARN: verify failed: {}", name);
            failed += 1;
            continue;
        }

        // QT8: file TRUOC
        let offset = match writer.append_node(&chain, 0, true, ts) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[seeder] write error {}: {:?}", name, e);
                failed += 1;
                continue;
            }
        };
        let _ = writer.append_alias(&format!("_qr_{:016X}", hash), hash, ts);

        // Registry SAU
        registry.insert(&chain, 0, offset, ts, true);
        registry.register_alias(name, hash);
        for &alias in aliases {
            registry.register_alias(alias, hash);
            let _ = writer.append_alias(alias, hash, ts);
        }

        // Log CUOI CUNG
        log.append(LogEvent::NodeCreated {
            chain_hash: hash,
            layer: 0,
            file_offset: offset,
            timestamp: ts,
        });

        let uname = ucd::lookup(cp).map(|e| e.name).unwrap_or("?");
        println!("[seeder] ✓ {} (U+{:05X} {})", name, cp, uname);
        count += 1;
    }

    println!();
    println!("[seeder] Nodes   : {}", count);
    println!("[seeder] Failed  : {}", failed);
    println!("[seeder] Registry: {}", registry.len());
    println!("[seeder] Aliases : {}", registry.alias_count());
    println!("[seeder] File    : {} bytes", writer.size());

    if failed > 0 {
        eprintln!("[seeder] ERRORS — abort");
        std::process::exit(1);
    }

    // Ghi file
    let bytes = writer.as_bytes().to_vec();
    fs::write("origin.olang", &bytes).expect("write origin.olang");
    println!("[seeder] ✓ origin.olang ({} bytes)", bytes.len());

    // Verify roundtrip
    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!(
        "[seeder] ✓ Roundtrip: {} nodes, {} aliases",
        parsed.node_count(),
        parsed.alias_count()
    );

    println!("[seeder] Done ✓  ○(empty)==○");
}
