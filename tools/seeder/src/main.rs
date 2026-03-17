//! # seeder — Seed L0 toàn bộ từ UCD
//!
//! Tạo origin.olang với ~5400 L0 nodes từ Unicode 18.0.
//! KHÔNG hardcode. Mọi chain từ encode_codepoint(cp).
//!
//! Bảng tuần hoàn hoàn chỉnh: mọi hình dạng (SDF), mọi quan hệ (MATH),
//! mọi cảm xúc (EMOTICON), mọi nhịp thời gian (MUSICAL).

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::encoder::encode_codepoint;
use olang::log::{EventLog, LogEvent};
use olang::qr::QRSigner;
use olang::registry::Registry;
use olang::startup::L0_NATURAL_ALIASES;
use olang::writer::OlangWriter;

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

fn main() {
    println!("[seeder] HomeOS Origin Seeder — Full L0");
    println!("[seeder] Unicode 18.0 — bảng tuần hoàn hoàn chỉnh");

    if ucd::table_len() == 0 {
        eprintln!("[seeder] ERROR: UCD table empty");
        std::process::exit(1);
    }

    let table = ucd::table();
    println!("[seeder] UCD: {} entries (5 nhóm × 5 chiều)", table.len());

    let ts = now_ns();
    let seed = [0x42u8; 32];
    let signer = QRSigner::from_seed(&seed);

    let mut writer = OlangWriter::new(ts);
    let mut registry = Registry::new();
    let mut log = EventLog::new(String::from("origin.olang.log"));

    let mut count = 0usize;
    let mut failed = 0usize;

    // ── Phase 1: Seed toàn bộ UCD entries → L0 ──────────────────────────────
    println!();
    println!("[seeder] Phase 1: Seeding {} L0 atoms...", table.len());

    // Group counters
    let mut sdf_count = 0usize;
    let mut math_count = 0usize;
    let mut emo_count = 0usize;
    let mut mus_count = 0usize;

    for entry in table {
        let chain = encode_codepoint(entry.cp);
        let hash = chain.chain_hash();
        let qr = signer.sign_qr(&chain, ts);

        if !signer.verify(&qr) {
            eprintln!("[seeder] WARN: verify failed: U+{:05X}", entry.cp);
            failed += 1;
            continue;
        }

        // QT9: file TRƯỚC
        let offset = match writer.append_node(&chain, 0, true, ts) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[seeder] write error U+{:05X}: {:?}", entry.cp, e);
                failed += 1;
                continue;
            }
        };
        let _ = writer.append_alias(&format!("_qr_{:016X}", hash), hash, ts);

        // Unicode NAME = primary alias (QT②)
        let _ = writer.append_alias(entry.name, hash, ts);

        // Registry SAU
        registry.insert(&chain, 0, offset, ts, true);
        registry.register_alias(entry.name, hash);

        // Log
        log.append(LogEvent::NodeCreated {
            chain_hash: hash,
            layer: 0,
            file_offset: offset,
            timestamp: ts,
        });

        // Count by group
        match entry.group {
            0x01 => sdf_count += 1,
            0x02 => math_count += 1,
            0x03 => emo_count += 1,
            0x04 => mus_count += 1,
            _ => {}
        }

        count += 1;
    }

    println!("[seeder]   SDF (Shape)    : {} atoms", sdf_count);
    println!("[seeder]   MATH (Relation): {} atoms", math_count);
    println!("[seeder]   EMOTICON (V+A) : {} atoms", emo_count);
    println!("[seeder]   MUSICAL (Time) : {} atoms", mus_count);

    // ── Phase 2: Natural language aliases ────────────────────────────────────
    println!();
    println!("[seeder] Phase 2: Natural aliases...");

    let mut alias_count = 0usize;
    for &(alias, cp) in L0_NATURAL_ALIASES {
        let chain = encode_codepoint(cp);
        let hash = chain.chain_hash();
        // Chỉ ghi alias nếu node tồn tại
        if registry.lookup_hash(hash).is_some() {
            let _ = writer.append_alias(alias, hash, ts);
            registry.register_alias(alias, hash);
            alias_count += 1;
        }
    }
    println!("[seeder]   {} natural aliases added", alias_count);

    // ── Phase 3: L1 System DNA (Skills, Agents, VM ops) ─────────────────────
    println!();
    println!("[seeder] Phase 3: L1 System DNA...");

    let mut l1_count = 0usize;
    for entry in olang::startup::L1_SYSTEM_SEED {
        let chain = encode_codepoint(entry.codepoint);
        let hash = chain.chain_hash();
        let qr = signer.sign_qr(&chain, ts);

        if !signer.verify(&qr) {
            failed += 1;
            continue;
        }

        let offset = match writer.append_node(&chain, 1, true, ts) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[seeder] L1 write error {}: {:?}", entry.name, e);
                failed += 1;
                continue;
            }
        };

        let kind = olang::registry::NodeKind::from_byte(entry.kind)
            .unwrap_or(olang::registry::NodeKind::Knowledge);
        writer.append_node_kind(hash, kind as u8, ts);
        let _ = writer.append_alias(entry.name, hash, ts);

        registry.insert(&chain, 1, offset, ts, true);
        registry.register_alias(entry.name, hash);
        for &a in entry.aliases {
            let _ = writer.append_alias(a, hash, ts);
            registry.register_alias(a, hash);
        }

        l1_count += 1;
    }
    println!("[seeder]   {} L1 system nodes", l1_count);

    // ── Summary ──────────────────────────────────────────────────────────────
    println!();
    println!("[seeder] ═══════════════════════════════════════════");
    println!("[seeder] L0 Atoms   : {}", count);
    println!("[seeder] L1 DNA     : {}", l1_count);
    println!("[seeder] Aliases    : {}", registry.alias_count());
    println!("[seeder] Failed     : {}", failed);
    println!("[seeder] Registry   : {} entries", registry.len());
    println!("[seeder] File       : {} bytes", writer.size());
    println!("[seeder] ═══════════════════════════════════════════");

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

    println!("[seeder] Done ✓  ○(∅)==○ — bảng tuần hoàn hoàn chỉnh");
}
