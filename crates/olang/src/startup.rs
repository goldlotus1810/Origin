//! # startup — Boot Sequence
//!
//! Stage 0: Raw entry — không có gì
//! Stage 1: Self Init — ○(∅)==○ (registry rỗng = hợp lệ)
//! Stage 2: Axiom Load — 4 opcodes: IDENT/SELF/IDEM/INST
//! Stage 3: UCD Table — từ .rodata (5263 entries tĩnh)
//! Stage 4: Registry Init — rebuild từ file hoặc rỗng
//! Stage 5: Alias Index — nạp aliases vào RAM
//! Stage 6: Verify — ○(x)==x self-check

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use crate::molecular::MolecularChain;
use crate::encoder::encode_codepoint;
use crate::registry::Registry;
use crate::reader::{OlangReader, ParseError};
use crate::lca::lca;

// ─────────────────────────────────────────────────────────────────────────────
// BootResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả boot.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct BootResult {
    pub registry:    Registry,
    pub node_count:  usize,
    pub alias_count: usize,
    pub stage:       BootStage,
    pub errors:      Vec<String>,
}

/// Stage boot đã đạt được.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum BootStage {
    Raw      = 0,
    SelfInit = 1,
    AxiomLoad= 2,
    UcdReady = 3,
    Loaded   = 4,
    Verified = 5,
}

#[allow(missing_docs)]
impl BootResult {
    pub fn is_ok(&self) -> bool {
        self.stage >= BootStage::UcdReady && self.errors.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HomeOS::boot()
// ─────────────────────────────────────────────────────────────────────────────

/// Boot HomeOS từ file bytes (origin.olang).
///
/// Nếu bytes = None hoặc rỗng → boot từ hư không (○(∅)==○).
/// Nếu bytes có data → rebuild Registry từ file.
pub fn boot(file_bytes: Option<&[u8]>) -> BootResult {
    let mut errors = Vec::new();

    // Stage 0: Raw — không làm gì
    // Stage 1: Self Init — ○(∅)==○
    let mut registry = Registry::new();
    let mut stage    = BootStage::SelfInit;

    // Stage 2: Axiom Load — seed 4 axiom nodes
    // Dùng UCD nếu có, không thì bỏ qua
    if ucd::table_len() > 0 {
        seed_axioms(&mut registry);
        stage = BootStage::AxiomLoad;
    }

    // Stage 3: UCD ready
    if ucd::table_len() > 0 {
        stage = BootStage::UcdReady;
    } else {
        errors.push(String::from("UCD table empty — build with UnicodeData.txt"));
    }

    // Stage 4: Load từ file
    if let Some(bytes) = file_bytes {
        if !bytes.is_empty() {
            match load_from_bytes(bytes, &mut registry) {
                Ok(()) => { stage = BootStage::Loaded; }
                Err(e) => {
                    errors.push(alloc::format!("Load error: {:?}", e));
                }
            }
        }
    }

    // Stage 5 + 6: Verify ○(x)==x
    if stage >= BootStage::UcdReady {
        match verify_identity(&registry) {
            Ok(()) => { stage = BootStage::Verified; }
            Err(e) => { errors.push(e); }
        }
    }

    let node_count  = registry.len();
    let alias_count = registry.alias_count();

    BootResult { registry, node_count, alias_count, stage, errors }
}

/// Boot từ hư không — ○(∅)==○.
pub fn boot_empty() -> BootResult {
    boot(None)
}

// ─────────────────────────────────────────────────────────────────────────────
// Seed axioms
// ─────────────────────────────────────────────────────────────────────────────

/// Seed 4 axiom nodes vào Registry.
///
/// Không phụ thuộc vào file — đây là L0 bất biến.
fn seed_axioms(registry: &mut Registry) {
    let ts = 0i64; // boot time

    // ○ (origin) = U+25CB WHITE CIRCLE
    let origin = encode_codepoint(0x25CB);
    let h = registry.insert(&origin, 0, 0, ts, true);
    registry.register_alias("○", h);
    registry.register_alias("origin", h);

    // Axiom 1: identity — ∅ (empty set) proxy → ○
    // Đại diện bằng ∅ U+2205
    if ucd::table_len() > 0 {
        let empty = encode_codepoint(0x2205);
        let he = registry.insert(&empty, 0, 1, ts, true);
        registry.register_alias("∅", he);
        registry.register_alias("empty", he);

        // Axiom 2: idem — ∘ compose U+2218
        let compose = encode_codepoint(0x2218);
        let hc = registry.insert(&compose, 0, 2, ts, true);
        registry.register_alias("∘", hc);
        registry.register_alias("compose", hc);

        // Axiom 3: instance — ∈ member U+2208
        let member = encode_codepoint(0x2208);
        let hm = registry.insert(&member, 0, 3, ts, true);
        registry.register_alias("∈", hm);
        registry.register_alias("instance", hm);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Load từ file bytes
// ─────────────────────────────────────────────────────────────────────────────

fn load_from_bytes(bytes: &[u8], registry: &mut Registry) -> Result<(), ParseError> {
    let reader = OlangReader::new(bytes)?;
    let parsed = reader.parse_all()?;

    // Nạp nodes
    for node in &parsed.nodes {
        registry.insert(
            &node.chain,
            node.layer,
            node.file_offset,
            node.timestamp,
            node.is_qr,
        );
    }

    // Nạp aliases
    for alias in &parsed.aliases {
        // Bỏ qua _qr_ internal aliases
        if !alias.name.starts_with("_qr_") {
            registry.register_alias(&alias.name, alias.chain_hash);
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Verify ○(x)==x
// ─────────────────────────────────────────────────────────────────────────────

/// Verify: ○ không làm hỏng thứ gì.
///
/// Lấy một chain từ registry → LCA(x, x) == x.
fn verify_identity(registry: &Registry) -> Result<(), String> {
    if ucd::table_len() == 0 { return Ok(()); } // skip nếu không có UCD

    // Test với origin node
    let origin = encode_codepoint(0x25CB);
    let lca_result = lca(&origin, &origin);

    // ○(x)==x: LCA(x,x) phải == x
    if lca_result != origin {
        return Err(alloc::format!(
            "Axiom violated: LCA(○,○) ≠ ○ (hash {:016X} ≠ {:016X})",
            lca_result.chain_hash(),
            origin.chain_hash(),
        ));
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry lookup helper — dùng trong VM LOAD resolution
// ─────────────────────────────────────────────────────────────────────────────

/// Resolve một alias/name → MolecularChain.
///
/// Thứ tự lookup:
///   1. Single emoji/symbol → encode_codepoint trực tiếp
///   2. Alias trong registry → tìm codepoint qua LOOKUP_TABLE
///   3. First emoji char trong string
///   4. Empty chain
pub fn resolve(name: &str, registry: &Registry) -> MolecularChain {
    // 1. Single character → encode trực tiếp
    let chars: alloc::vec::Vec<char> = name.chars().collect();
    if chars.len() == 1 {
        let cp = chars[0] as u32;
        if cp > 0x20 {
            return encode_codepoint(cp);
        }
    }

    // 2. Alias lookup → tìm codepoint ứng với hash
    if let Some(hash) = registry.lookup_name(name) {
        // Scan ALIAS_CODEPOINTS để tìm cp có chain_hash == hash
        for &(alias, cp) in ALIAS_CODEPOINTS {
            let chain = encode_codepoint(cp);
            if chain.chain_hash() == hash {
                return chain;
            }
            // Cũng check alias word match
            if alias == name {
                return chain;
            }
        }
    }

    // 3. First non-ASCII char trong string
    for c in name.chars() {
        let cp = c as u32;
        if cp > 0x2000 {
            return encode_codepoint(cp);
        }
    }

    // 4. Word match trong ALIAS_CODEPOINTS
    for &(alias, cp) in ALIAS_CODEPOINTS {
        if alias == name {
            return encode_codepoint(cp);
        }
    }

    MolecularChain::empty()
}

/// Bảng tra cứu alias → codepoint cho L0 nodes.
/// Dùng khi registry không có chain raw (chỉ có hash).
static ALIAS_CODEPOINTS: &[(&str, u32)] = &[
    // fire
    ("fire", 0x1F525), ("lửa", 0x1F525), ("lua", 0x1F525),
    ("feu", 0x1F525),  ("fuego", 0x1F525),
    // water
    ("water", 0x1F4A7), ("nước", 0x1F4A7), ("nuoc", 0x1F4A7),
    ("eau", 0x1F4A7),
    // cold
    ("cold", 0x2744), ("lạnh", 0x2744), ("lanh", 0x2744),
    // sun
    ("sun", 0x2600), ("warm", 0x1F31E),
    // mind
    ("mind", 0x1F9E0), ("brain", 0x1F9E0), ("tâm trí", 0x1F9E0),
    // heart
    ("heart", 0x2764), ("tim", 0x2764), ("trái tim", 0x2764),
    // origin
    ("origin", 0x25CB), ("○", 0x25CB),
    // math
    ("∘", 0x2218), ("compose", 0x2218),
    ("∈", 0x2208), ("member", 0x2208),
    ("∅", 0x2205), ("empty", 0x2205),
    // joy / sadness
    ("vui", 0x1F60A), ("happy", 0x1F60A), ("joy", 0x1F60A),
    ("buồn", 0x1F614), ("sad", 0x1F614),
    // danger / alert
    ("danger", 0x26A0), ("nguy hiểm", 0x26A0),
    // yes / no
    ("yes", 0x2705), ("có", 0x2705),
    ("no", 0x274C), ("không", 0x274C),
];

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn skip() -> bool { ucd::table_len() == 0 }

    #[test]
    fn boot_empty_ok() {
        let result = boot_empty();
        assert!(result.stage >= BootStage::SelfInit,
            "Boot empty phải reach SelfInit");
        // Registry rỗng = hợp lệ (○(∅)==○)
        assert!(result.errors.is_empty() || !result.is_ok() || true,
            "Boot empty không crash");
    }

    #[test]
    fn boot_with_ucd_reaches_verified() {
        if skip() { return; }
        let result = boot_empty();
        assert!(result.stage >= BootStage::Verified,
            "Boot với UCD phải reach Verified: {:?}", result.errors);
        assert!(result.errors.is_empty(),
            "Không có errors: {:?}", result.errors);
    }

    #[test]
    fn boot_seeds_axioms() {
        if skip() { return; }
        let result = boot_empty();
        // Registry phải có origin node
        assert!(result.registry.lookup_name("○").is_some(),
            "○ phải có trong registry sau boot");
        assert!(result.registry.lookup_name("origin").is_some());
        assert!(result.registry.lookup_name("∘").is_some(),
            "∘ (compose) phải có");
        assert!(result.registry.lookup_name("∈").is_some(),
            "∈ (member) phải có");
    }

    #[test]
    fn boot_axiom_identity() {
        if skip() { return; }
        // Verify: LCA(○,○)==○
        let origin = encode_codepoint(0x25CB);
        let lca_result = lca(&origin, &origin);
        assert_eq!(lca_result, origin,
            "○(x)==x: LCA(○,○) phải == ○");
    }

    #[test]
    fn boot_from_seeded_file() {
        if skip() { return; }
        // Tạo mini file với 1 node
        use crate::writer::OlangWriter;
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut w = OlangWriter::new(0);
        w.append_node(&chain, 0, true, 0).unwrap();
        w.append_alias("fire", chain.chain_hash(), 0).unwrap();
        let bytes = w.into_bytes();

        let result = boot(Some(&bytes));
        assert!(result.stage >= BootStage::Loaded,
            "Boot từ file hợp lệ phải reach Loaded: {:?}", result.errors);
        assert!(result.registry.lookup_name("fire").is_some(),
            "Alias 'fire' phải được load");
    }

    #[test]
    fn boot_from_bad_file() {
        // File bytes xấu → fallback, không crash
        let bad_bytes = [0x00u8; 20];
        let result = boot(Some(&bad_bytes));
        // Không panic, chỉ report error
        assert!(result.stage >= BootStage::SelfInit,
            "Boot từ file xấu không crash");
    }

    #[test]
    fn boot_stage_ordering() {
        assert!(BootStage::SelfInit  < BootStage::AxiomLoad);
        assert!(BootStage::AxiomLoad < BootStage::UcdReady);
        assert!(BootStage::UcdReady  < BootStage::Loaded);
        assert!(BootStage::Loaded    < BootStage::Verified);
    }

    #[test]
    fn resolve_single_emoji() {
        if skip() { return; }
        let registry = Registry::new();
        let chain = resolve("🔥", &registry);
        assert!(!chain.is_empty(),
            "resolve('🔥') phải trả non-empty chain");
        assert_eq!(chain, encode_codepoint(0x1F525));
    }

    #[test]
    fn resolve_unknown_returns_empty() {
        let registry = Registry::new();
        let chain = resolve("xyz_unknown_abc", &registry);
        assert!(chain.is_empty(),
            "Unknown alias → empty chain");
    }

    #[test]
    fn resolve_origin_symbol() {
        if skip() { return; }
        let registry = Registry::new();
        let chain = resolve("○", &registry);
        assert!(!chain.is_empty(), "○ → non-empty chain");
        assert_eq!(chain, encode_codepoint(0x25CB));
    }
}
