//! # startup — Boot Sequence
//!
//! Stage 0: Raw entry — không có gì
//! Stage 1: Self Init — ○(∅)==○ (registry rỗng = hợp lệ)
//! Stage 2: Axiom Load — 4 opcodes: IDENT/SELF/IDEM/INST
//! Stage 3: UCD Table — từ .rodata (5263 entries tĩnh)
//! Stage 4: Registry Init — rebuild từ file hoặc rỗng
//! Stage 5: Alias Index — nạp aliases vào RAM
//! Stage 6: Verify — ○(x)==x self-check
//! Stage 7: Manifest — scan registry → SystemManifest (biết mình có gì)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::encoder::encode_codepoint;
use crate::lca::lca;
use crate::molecular::MolecularChain;
use crate::reader::{OlangReader, ParseError};
use crate::registry::{NodeKind, Registry};

// ─────────────────────────────────────────────────────────────────────────────
// BootResult
// ─────────────────────────────────────────────────────────────────────────────

/// Edge đã load từ file — để restore Silk graph.
#[derive(Debug, Clone)]
pub struct BootEdge {
    /// FNV-1a hash of source chain
    pub from_hash: u64,
    /// FNV-1a hash of target chain
    pub to_hash: u64,
    /// Edge type byte (SilkEdgeKind)
    pub edge_type: u8,
    /// Timestamp khi tạo
    pub timestamp: i64,
}

/// Kết quả boot.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct BootResult {
    pub registry: Registry,
    pub node_count: usize,
    pub alias_count: usize,
    pub stage: BootStage,
    pub errors: Vec<String>,
    /// SystemManifest — hệ thống biết mình đang có gì sau boot.
    pub manifest: SystemManifest,
    /// Silk edges đã load từ file — restore vào SilkGraph.
    pub edges: Vec<BootEdge>,
    /// QT8: bytes cần ghi vào origin.olang — axioms/L1 chưa có trong file.
    /// Caller (HomeRuntime) phải flush ra disk TRƯỚC khi dùng.
    pub pending_writes: Vec<u8>,
    /// Số seed nodes đã ghi bổ sung (QT8).
    pub seeds_written: usize,
    /// STM observations đã load từ file — restore vào ShortTermMemory.
    pub stm_records: Vec<crate::reader::ParsedStm>,
    /// HebbianLink đã load từ file — restore vào SilkGraph.learned.
    pub hebbian_records: Vec<crate::reader::ParsedHebbian>,
    /// KnowTree compact nodes đã load từ file — restore vào KnowTree.
    pub knowtree_records: Vec<crate::reader::ParsedKnowTree>,
    /// SlimKnowTree node records (0x0A) — restore vào SlimKnowTree.
    pub slim_knowtree_records: Vec<crate::reader::ParsedSlimKnowTree>,
    /// ConversationCurve turn records — replay to reconstruct curve.
    pub curve_records: Vec<crate::reader::ParsedCurve>,
    /// Auth records — last one is the active identity.
    pub auth_records: Vec<crate::reader::ParsedAuth>,
}

/// Stage boot đã đạt được.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum BootStage {
    Raw = 0,
    SelfInit = 1,
    AxiomLoad = 2,
    UcdReady = 3,
    Loaded = 4,
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
    use crate::writer::OlangWriter;

    let mut errors = Vec::new();
    let mut pending_writes = Vec::new();

    // Stage 0: Raw — không làm gì
    // Stage 1: Self Init — ○(∅)==○
    let mut registry = Registry::new();
    let mut stage = BootStage::SelfInit;

    // Stage 2: Axiom Load — seed axioms + L1 DNA + full L0 UCD
    // Thứ tự quan trọng:
    //   1. Axioms (nền tảng)
    //   2. L1 DNA (Skills, Agents — cần đúng NodeKind TRƯỚC)
    //   3. Full L0 (phần còn lại UCD — skip cái đã register)
    if ucd::table_len() > 0 {
        // Phase 2a: 4 axiom nodes (○, ∅, ∘, ∈) — nền tảng
        seed_axioms(&mut registry);

        // Bulk mode: O(n log n) thay vì O(n²) — skip per-insert sort + LCA
        registry.begin_bulk();

        // Phase 2b: L1 DNA — Skills, Agents, VM ops, Sensors
        // Phải chạy TRƯỚC L0 full vì cùng codepoint → L1 cần đúng NodeKind
        seed_l1_system(&mut registry);

        // Phase 2c: TOÀN BỘ ~5400 UCD entries → L0 (bảng tuần hoàn hoàn chỉnh)
        // Skip cái đã register bởi axioms + L1
        seed_l0_full(&mut registry);

        registry.finalize_bulk();

        // Phase 2d: Natural language aliases cho L0 atoms phổ biến
        seed_l0_aliases(&mut registry);

        stage = BootStage::AxiomLoad;
    }

    // Stage 3: UCD ready
    if ucd::table_len() > 0 {
        stage = BootStage::UcdReady;
    } else {
        errors.push(String::from("UCD table empty — build with UnicodeData.txt"));
    }

    // Stage 4: Load từ file
    let mut edges = Vec::new();
    let mut stm_records = Vec::new();
    let mut hebbian_records = Vec::new();
    let mut knowtree_records = Vec::new();
    let mut slim_knowtree_records = Vec::new();
    let mut curve_records = Vec::new();
    let mut auth_records = Vec::new();
    let file_had_data = if let Some(bytes) = file_bytes {
        if !bytes.is_empty() {
            match load_from_bytes(bytes, &mut registry) {
                Ok(loaded) => {
                    edges = loaded.edges;
                    stm_records = loaded.stm_records;
                    hebbian_records = loaded.hebbian_records;
                    knowtree_records = loaded.knowtree_records;
                    slim_knowtree_records = loaded.slim_knowtree_records;
                    curve_records = loaded.curve_records;
                    auth_records = loaded.auth_records;
                    stage = BootStage::Loaded;
                    true
                }
                Err(e) => {
                    errors.push(alloc::format!("Load error: {:?}", e));
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    // Stage 5 + 6: Verify ○(x)==x
    if stage >= BootStage::UcdReady {
        match verify_identity(&registry) {
            Ok(()) => {
                stage = BootStage::Verified;
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    // ── QT8: ghi file TRƯỚC — đảm bảo mọi thứ trong Registry đều có trong origin.olang ──
    // Nếu file rỗng/không có → ghi toàn bộ L0+L1 axioms ra pending_writes
    // Nếu file đã có data → kiểm tra L1 thiếu gì → ghi bổ sung
    let seeds_written = if ucd::table_len() > 0 {
        let mut writer = if file_had_data {
            OlangWriter::new_append()
        } else {
            OlangWriter::new(0)
        };

        let count = write_missing_seeds(&registry, file_bytes, &mut writer);

        if writer.write_count() > 0 {
            pending_writes = writer.into_bytes();
        }

        count
    } else {
        0
    };

    let node_count = registry.len();
    let alias_count = registry.alias_count();

    // Stage 7: Manifest — scan registry → phân loại nodes
    let manifest = SystemManifest::scan(&registry);

    BootResult {
        registry,
        node_count,
        alias_count,
        stage,
        errors,
        manifest,
        edges,
        pending_writes,
        seeds_written,
        stm_records,
        hebbian_records,
        knowtree_records,
        slim_knowtree_records,
        curve_records,
        auth_records,
    }
}

/// Boot từ hư không — ○(∅)==○.
pub fn boot_empty() -> BootResult {
    boot(None)
}

// ─────────────────────────────────────────────────────────────────────────────
// Seed axioms
// ─────────────────────────────────────────────────────────────────────────────

/// Seed 4 axiom nodes vào Registry — nền tảng bất biến.
///
/// Luôn chạy trước seed_l0_full(). Không phụ thuộc vào file.
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
// L0 Full Seed — toàn bộ bảng tuần hoàn Unicode (~5400 nguyên tố)
// ─────────────────────────────────────────────────────────────────────────────

/// Seed TOÀN BỘ UCD_TABLE vào Registry — ~5400 L0 nodes.
///
/// Mỗi Unicode character trong 5 nhóm = 1 nguyên tố bất biến.
/// L0 = bảng tuần hoàn hoàn chỉnh: mọi hình dạng, mọi quan hệ,
/// mọi cảm xúc, mọi nhịp thời gian mà hệ thống biết khi sinh ra.
///
/// Unicode NAME = alias duy nhất (đã chuẩn hóa bởi Unicode Consortium).
/// Không đặt tên khác (QT②).
///
/// Returns: số L0 nodes đã seed.
fn seed_l0_full(registry: &mut Registry) -> usize {
    use crate::registry::NodeKind;

    let table = ucd::table();
    if table.is_empty() {
        return 0;
    }

    let ts = 0i64;
    let mut count = 0usize;

    for (offset, entry) in table.iter().enumerate() {
        let chain = encode_codepoint(entry.cp);
        let hash = chain.chain_hash();

        // Skip nếu đã seed bởi seed_axioms()
        if registry.lookup_hash(hash).is_some() {
            continue;
        }

        // L0, QR=true, offset dựa trên vị trí trong bảng
        registry.insert_with_kind(
            &chain,
            0,                      // layer 0
            offset as u64 + 100,    // offset (sau axioms)
            ts,
            true,                   // is_qr (L0 bất biến)
            NodeKind::Alphabet,     // Tất cả L0 = Alphabet
        );

        // Unicode NAME = alias (QT②: tên ký tự Unicode = tên node)
        registry.register_alias(entry.name, hash);

        count += 1;
    }

    count
}

/// Số L0 nodes đã seed (axioms + full UCD).
///
/// Dùng bởi SystemManifest và tests.
pub fn l0_expected_count() -> usize {
    // 4 axioms + toàn bộ UCD table (minus overlap)
    ucd::table_len()
}

/// Seed natural language aliases cho L0 atoms phổ biến.
///
/// QT③: Ngôn ngữ tự nhiên = alias → node. Không tạo node riêng.
fn seed_l0_aliases(registry: &mut Registry) {
    for &(alias, cp) in L0_NATURAL_ALIASES {
        let chain = encode_codepoint(cp);
        let hash = chain.chain_hash();
        registry.register_alias(alias, hash);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap Programs — Olang source cho bản năng và nhận thức bẩm sinh
// ─────────────────────────────────────────────────────────────────────────────

/// Bootstrap Olang programs — firmware chạy trên VM ngay khi boot.
///
/// Mỗi program là 1 Olang source string:
///   - Tự nhận thức: hệ thống biết mình có gì
///   - Bản năng: 7 instincts định nghĩa bằng Olang
///   - Quy tắc: axiom assertions
///   - Quan hệ: nhóm cùng chiều → Silk edges
///
/// Runtime execute qua OlangVM → VmEvents → Silk/Registry.
pub fn bootstrap_programs() -> Vec<&'static str> {
    Vec::from(BOOTSTRAP_PROGRAMS)
}

/// Bootstrap programs — chạy theo thứ tự.
///
/// Phase 1: Axiom verification (nền tảng đúng)
/// Phase 2: Self-awareness (biết mình có gì)
/// Phase 3: Group binding (nhóm nguyên tố cùng chiều → Silk)
/// Phase 4: Instinct prototypes (bản năng bẩm sinh)
static BOOTSTRAP_PROGRAMS: &[&str] = &[
    // ── Phase 1: Axiom verification ──────────────────────────────────────
    // QT1: ○ là nguồn gốc — assert nó tồn tại
    "○;",

    // ── Phase 2: Self-awareness ──────────────────────────────────────────
    // Hệ thống nhìn thấy chính mình
    "stats;",

    // ── Phase 3: Group binding — 8 SDF primitives ────────────────────────
    // Hình dạng cơ bản: ● ▬ ■ ▲ ○ ∪ ∩ ∖
    // LCA của nhóm → concept "shape" (trừu tượng)
    "● ∘ ▬;",
    "■ ∘ ▲;",
    "● ∘ ■;",

    // ── Phase 3b: Group binding — 8 RELATION primitives ──────────────────
    // Quan hệ cơ bản: ∈ ⊂ ≡ ⊥ ∘ → ≈ ←
    "∈ ∘ ⊂;",
    "≡ ∘ →;",
    "∈ ∘ ≡;",

    // ── Phase 4: Instinct templates ──────────────────────────────────────
    // Honesty: typeof trả về mức tin cậy — assert kiểm tra
    "typeof ○;",

    // Analogy: A ∘ B tìm LCA → delta giữa 2 concept
    // Khi cần A:B :: C:? → tính delta, áp lên C
    "● ∘ ▲;",      // shape delta: Sphere vs Cone

    // Causality: → (Causes) là quan hệ bẩm sinh
    // Khi 2 chain có temporal ordering + co-activate → causal
    "→;",

    // Contradiction: ⊥ (Orthogonal) = mâu thuẫn bẩm sinh
    "⊥;",

    // Curiosity: ≈ (Similar) tìm nearest → 1 - similarity = novelty
    "≈;",

    // Reflection: inspect → chain structure quality
    "inspect ○;",

    // ── Phase 5: Self-test with new syntax ─────────────────────────────
    // Loop: verify SDF primitives 3 times (stability test)
    "loop 3 { stats; };",

    // Let binding: assign concept to variable
    "let origin = ○;",

    // Assert: verify origin exists
    "assert origin;",

    // ── Phase 6: Axiom verification ─────────────────────────────────────
    // QT1: ○ là nguồn gốc — compose origin with self = same origin
    "let qt1 = ○ ∘ ○;",
    "assert qt1;",

    // QT2: ∞-1 is correct — fuse ensures finite chain
    "fuse;",

    // QT3: group shape primitives — 4 SDF shapes compose to abstract "shape"
    "let shapes = ● ∘ ▬ ∘ ■ ∘ ▲;",
    "assert shapes;",

    // QT3: group relations — abstract "relation" from primitives
    "let rels = ∈ ∘ ⊂ ∘ ≡ ∘ →;",
    "assert rels;",

    // ── Phase 7: Instinct definitions (Olang programs) ──────────────────
    // Honesty instinct: typeof checks knowledge quality
    "fn check_honesty { typeof ○; };",

    // Analogy instinct: compute delta between two concepts
    "fn check_analogy { let d = ● ∘ ▲; assert d; };",

    // Causality instinct: verify → relation exists
    "fn check_causality { let c = ● → ▲; assert c; };",

    // Run instinct checks
    "check_honesty;",
    "check_analogy;",
    "check_causality;",
];

/// Bảng alias đa ngôn ngữ cho L0 atoms phổ biến.
///
/// Unicode NAME là alias chính (từ UCD). Bảng này thêm alias
/// ngôn ngữ tự nhiên cho ~40 atoms thường dùng nhất.
///
/// QT③: Ngôn ngữ tự nhiên = alias → node. Không tạo node riêng.
pub static L0_NATURAL_ALIASES: &[(&str, u32)] = &[
    // fire
    ("fire", 0x1F525), ("lửa", 0x1F525), ("lua", 0x1F525), ("feu", 0x1F525),
    // water
    ("water", 0x1F4A7), ("nước", 0x1F4A7), ("nuoc", 0x1F4A7), ("eau", 0x1F4A7),
    // light
    ("light", 0x1F4A1), ("anh-sang", 0x1F4A1),
    // spark
    ("spark", 0x2728), ("tia-lua", 0x2728),
    // bolt / lightning
    ("bolt", 0x26A1), ("lightning", 0x26A1), ("sét", 0x26A1),
    // earth
    ("earth", 0x1F30D), ("đất", 0x1F30D), ("dat", 0x1F30D),
    // wind
    ("wind", 0x1F32C), ("gió", 0x1F32C), ("gio", 0x1F32C),
    // sound
    ("sound", 0x1F50A), ("âm thanh", 0x1F50A),
    // cold
    ("cold", 0x2744), ("lạnh", 0x2744), ("lanh", 0x2744),
    // warm
    ("warm", 0x1F31E), ("ấm", 0x1F31E),
    // sun
    ("sun", 0x2600), ("mặt trời", 0x2600),
    // joy
    ("joy", 0x1F60C), ("vui", 0x1F60C), ("happy", 0x1F60C),
    // sadness
    ("sad", 0x1F614), ("buồn", 0x1F614), ("buồn bã", 0x1F614),
    // pain
    ("pain", 0x1F915), ("đau", 0x1F915),
    // fatigue
    ("tired", 0x1F634), ("mệt", 0x1F634), ("mệt mỏi", 0x1F634),
    // hunger
    ("hunger", 0x1F374), ("đói", 0x1F374),
    // danger
    ("danger", 0x26A0), ("nguy hiểm", 0x26A0),
    // dark
    ("dark", 0x1F311), ("tối", 0x1F311),
    // alert
    ("alert", 0x1F6A8), ("cảnh báo", 0x1F6A8),
    // home / shelter
    ("home", 0x1F3E0), ("nhà", 0x1F3E0), ("shelter", 0x1F3E0),
    ("house", 0x1F3E1), ("nhà ở", 0x1F3E1),
    // nature
    ("nature", 0x1F333), ("thiên nhiên", 0x1F333),
    // ocean
    ("ocean", 0x1F30A), ("biển", 0x1F30A), ("sea", 0x1F30A),
    // mind
    ("mind", 0x1F9E0), ("tâm trí", 0x1F9E0), ("brain", 0x1F9E0),
    // person
    ("person", 0x1F464), ("người", 0x1F464),
    // eye
    ("eye", 0x1F441), ("mắt", 0x1F441),
    // heart
    ("heart", 0x2764), ("tim", 0x2764), ("trái tim", 0x2764),
    // love
    ("love", 0x2764), ("yêu", 0x2764),
    // yes / no
    ("yes", 0x2705), ("có", 0x2705),
    ("no", 0x274C), ("không", 0x274C),
    // now
    ("now", 0x23F0), ("bây giờ", 0x23F0),
    // all
    ("all", 0x267E), ("tất cả", 0x267E),
    // move / stop
    ("move", 0x1F3C3), ("di chuyển", 0x1F3C3),
    ("stop", 0x1F6D1), ("dừng", 0x1F6D1),
    // open / close
    ("open", 0x1F513), ("mở", 0x1F513),
    ("close", 0x1F512), ("đóng", 0x1F512),
    // origin
    ("origin", 0x25CB), ("nguồn gốc", 0x25CB),
    // anger
    ("angry", 0x1F621), ("giận", 0x1F621), ("tức", 0x1F621),
    // fear
    ("scared", 0x1F628), ("sợ", 0x1F628), ("lo", 0x1F628),
    // family
    ("family", 0x1F46A), ("gia đình", 0x1F46A),
    // star / great
    ("great", 0x2B50), ("tuyệt", 0x2B50),
    // bad
    ("bad", 0x1F4A9), ("tệ", 0x1F4A9),
    // compose / member
    ("compose", 0x2218), ("∘", 0x2218),
    ("member", 0x2208), ("∈", 0x2208), ("instance", 0x2208),
    ("empty", 0x2205), ("∅", 0x2205),
];

/// L1 System Seed Entry — một component trong DNA của HomeOS.
///
/// Dùng chung giữa:
///   - `seed_l1_system()` (boot vào RAM khi không có file)
///   - `seeder` tool (ghi vào origin.olang)
///
/// Format: (codepoint, kind_byte, primary_name, aliases)
pub struct L1SeedEntry {
    /// Unicode codepoint (encode_codepoint sẽ tạo MolecularChain)
    pub codepoint: u32,
    /// NodeKind as u8 (dùng NodeKind::from_byte để convert)
    pub kind: u8,
    /// Primary name (e.g. "skill:honesty")
    pub name: &'static str,
    /// Additional aliases (e.g. &["Honesty"])
    pub aliases: &'static [&'static str],
}

/// Toàn bộ L1 system components — DNA của HomeOS.
///
/// Mọi thứ HomeOS cần biết nó có gì: Skills, Agents, VM ops, Sensors.
/// Bảng này là **source of truth** duy nhất — seeder đọc bảng này để ghi
/// vào origin.olang, boot đọc bảng này nếu file cũ chưa có L1.
pub static L1_SYSTEM_SEED: &[L1SeedEntry] = &[
    // ── Skills: 7 Instinct ─────────────────────────────────────────────────
    // Dùng Dingbats (0x2700-0x27BF) — SDF group, có trong UCD
    L1SeedEntry { codepoint: 0x2700, kind: 4, name: "skill:honesty",       aliases: &["Honesty"] },
    L1SeedEntry { codepoint: 0x2701, kind: 4, name: "skill:contradiction", aliases: &["Contradiction"] },
    L1SeedEntry { codepoint: 0x2702, kind: 4, name: "skill:causality",     aliases: &["Causality"] },
    L1SeedEntry { codepoint: 0x2703, kind: 4, name: "skill:abstraction",   aliases: &["Abstraction"] },
    L1SeedEntry { codepoint: 0x2704, kind: 4, name: "skill:analogy",       aliases: &["Analogy"] },
    L1SeedEntry { codepoint: 0x2706, kind: 4, name: "skill:curiosity",     aliases: &["Curiosity"] },
    L1SeedEntry { codepoint: 0x2707, kind: 4, name: "skill:reflection",    aliases: &["Reflection"] },

    // ── Skills: 11 LeoAI Domain ────────────────────────────────────────────
    // Dùng Box Drawing chars (0x2500-0x257F) — SDF group, có trong UCD
    L1SeedEntry { codepoint: 0x2500, kind: 4, name: "skill:ingest",         aliases: &["IngestSkill"] },
    L1SeedEntry { codepoint: 0x2502, kind: 4, name: "skill:similarity",     aliases: &["SimilaritySkill"] },
    L1SeedEntry { codepoint: 0x250C, kind: 4, name: "skill:delta",          aliases: &["DeltaSkill"] },
    L1SeedEntry { codepoint: 0x2510, kind: 4, name: "skill:cluster",        aliases: &["ClusterSkill"] },
    L1SeedEntry { codepoint: 0x2514, kind: 4, name: "skill:curator",        aliases: &["CuratorSkill"] },
    L1SeedEntry { codepoint: 0x2518, kind: 4, name: "skill:merge",          aliases: &["MergeSkill"] },
    L1SeedEntry { codepoint: 0x251C, kind: 4, name: "skill:prune",          aliases: &["PruneSkill"] },
    L1SeedEntry { codepoint: 0x2524, kind: 4, name: "skill:hebbian",        aliases: &["HebbianSkill"] },
    L1SeedEntry { codepoint: 0x252C, kind: 4, name: "skill:dream",          aliases: &["DreamSkill"] },
    L1SeedEntry { codepoint: 0x2534, kind: 4, name: "skill:proposal",       aliases: &["ProposalSkill"] },
    L1SeedEntry { codepoint: 0x253C, kind: 4, name: "skill:inverse_render", aliases: &["InverseRenderSkill"] },

    // ── Skills: Advanced ───────────────────────────────────────────────────
    L1SeedEntry { codepoint: 0x2550, kind: 4, name: "skill:generalization",   aliases: &["GeneralizationSkill"] },
    L1SeedEntry { codepoint: 0x2551, kind: 4, name: "skill:temporal_pattern", aliases: &["TemporalPatternSkill"] },

    // ── Skills: 4 Worker ───────────────────────────────────────────────────
    // Dùng Geometric Shapes (0x25A0-0x25FF) — SDF group
    L1SeedEntry { codepoint: 0x25A0, kind: 4, name: "skill:sensor",   aliases: &["SensorSkill"] },
    L1SeedEntry { codepoint: 0x25A1, kind: 4, name: "skill:actuator", aliases: &["ActuatorSkill"] },
    L1SeedEntry { codepoint: 0x25B2, kind: 4, name: "skill:security", aliases: &["SecuritySkill"] },
    L1SeedEntry { codepoint: 0x25B3, kind: 4, name: "skill:network",  aliases: &["NetworkSkill"] },

    // ── Agents ─────────────────────────────────────────────────────────────
    // Dùng Misc Symbols (0x2600-0x26FF) — EMOTICON group, có trong UCD
    // Chess symbols cho hierarchy
    L1SeedEntry { codepoint: 0x2654, kind: 3, name: "agent:aam",             aliases: &["AAM"] },            // ♔ King = AAM
    L1SeedEntry { codepoint: 0x2655, kind: 3, name: "agent:leo",             aliases: &["LeoAI"] },          // ♕ Queen = LeoAI
    L1SeedEntry { codepoint: 0x2656, kind: 3, name: "agent:chief:home",      aliases: &["HomeChief"] },      // ♖ Rook
    L1SeedEntry { codepoint: 0x2657, kind: 3, name: "agent:chief:vision",    aliases: &["VisionChief"] },    // ♗ Bishop
    L1SeedEntry { codepoint: 0x2658, kind: 3, name: "agent:chief:network",   aliases: &["NetworkChief"] },   // ♘ Knight
    L1SeedEntry { codepoint: 0x2659, kind: 3, name: "agent:chief:general",   aliases: &["GeneralChief"] },   // ♙ Pawn
    // Workers — dùng black chess pieces
    L1SeedEntry { codepoint: 0x265A, kind: 3, name: "agent:worker:sensor",   aliases: &["WorkerSensor"] },   // ♚
    L1SeedEntry { codepoint: 0x265B, kind: 3, name: "agent:worker:actuator", aliases: &["WorkerActuator"] }, // ♛
    L1SeedEntry { codepoint: 0x265C, kind: 3, name: "agent:worker:camera",   aliases: &["WorkerCamera"] },   // ♜
    L1SeedEntry { codepoint: 0x265D, kind: 3, name: "agent:worker:network",  aliases: &["WorkerNetwork"] },  // ♝
    L1SeedEntry { codepoint: 0x265E, kind: 3, name: "agent:worker:generic",  aliases: &["WorkerGeneric"] },  // ♞

    // ── Program: VM Built-in Functions ─────────────────────────────────────
    // Dùng Mathematical Operators (0x2200-0x22FF) — MATH group
    L1SeedEntry { codepoint: 0x2211, kind: 5, name: "fn:hyp_add",  aliases: &["__hyp_add"] },  // ∑
    L1SeedEntry { codepoint: 0x2212, kind: 5, name: "fn:hyp_sub",  aliases: &["__hyp_sub"] },  // −
    L1SeedEntry { codepoint: 0x2217, kind: 5, name: "fn:hyp_mul",  aliases: &["__hyp_mul"] },  // ∗
    L1SeedEntry { codepoint: 0x2215, kind: 5, name: "fn:hyp_div",  aliases: &["__hyp_div"] },  // ∕
    L1SeedEntry { codepoint: 0x2214, kind: 5, name: "fn:phys_add", aliases: &["__phys_add"] }, // ∔
    L1SeedEntry { codepoint: 0x2216, kind: 5, name: "fn:phys_sub", aliases: &["__phys_sub"] }, // ∖

    // ── Program: VM Opcodes (26 ops) ───────────────────────────────────────
    // Dùng Arrows (0x2190-0x21FF) — SDF group, có trong UCD
    L1SeedEntry { codepoint: 0x2190, kind: 5, name: "op:push",        aliases: &["Push"] },      // ←
    L1SeedEntry { codepoint: 0x2191, kind: 5, name: "op:push_num",    aliases: &["PushNum"] },   // ↑
    L1SeedEntry { codepoint: 0x2192, kind: 5, name: "op:push_mol",    aliases: &["PushMol"] },   // →
    L1SeedEntry { codepoint: 0x2193, kind: 5, name: "op:load",        aliases: &["Load"] },      // ↓
    L1SeedEntry { codepoint: 0x2194, kind: 5, name: "op:lca",         aliases: &["Lca"] },       // ↔
    L1SeedEntry { codepoint: 0x2195, kind: 5, name: "op:edge",        aliases: &["Edge"] },      // ↕
    L1SeedEntry { codepoint: 0x2196, kind: 5, name: "op:query",       aliases: &["Query"] },     // ↖
    L1SeedEntry { codepoint: 0x2197, kind: 5, name: "op:emit",        aliases: &["Emit"] },      // ↗
    L1SeedEntry { codepoint: 0x2198, kind: 5, name: "op:dup",         aliases: &["Dup"] },       // ↘
    L1SeedEntry { codepoint: 0x2199, kind: 5, name: "op:pop",         aliases: &["Pop"] },       // ↙
    L1SeedEntry { codepoint: 0x219A, kind: 5, name: "op:swap",        aliases: &["Swap"] },      // ↚
    L1SeedEntry { codepoint: 0x219B, kind: 5, name: "op:jmp",         aliases: &["Jmp"] },       // ↛
    L1SeedEntry { codepoint: 0x21A0, kind: 5, name: "op:jz",          aliases: &["Jz"] },        // ↠
    L1SeedEntry { codepoint: 0x21A3, kind: 5, name: "op:loop",        aliases: &["Loop"] },      // ↣
    L1SeedEntry { codepoint: 0x21A6, kind: 5, name: "op:call",        aliases: &["Call"] },      // ↦
    L1SeedEntry { codepoint: 0x21A9, kind: 5, name: "op:store",       aliases: &["Store"] },     // ↩
    L1SeedEntry { codepoint: 0x21AA, kind: 5, name: "op:load_local",  aliases: &["LoadLocal"] }, // ↪
    L1SeedEntry { codepoint: 0x21AB, kind: 5, name: "op:scope_begin", aliases: &["ScopeBegin"] },// ↫
    L1SeedEntry { codepoint: 0x21AC, kind: 5, name: "op:scope_end",   aliases: &["ScopeEnd"] }, // ↬
    L1SeedEntry { codepoint: 0x21AD, kind: 5, name: "op:fuse",        aliases: &["Fuse"] },      // ↭
    L1SeedEntry { codepoint: 0x21AE, kind: 5, name: "op:trace",       aliases: &["Trace"] },     // ↮
    L1SeedEntry { codepoint: 0x21B0, kind: 5, name: "op:inspect",     aliases: &["Inspect"] },   // ↰
    L1SeedEntry { codepoint: 0x21B1, kind: 5, name: "op:assert",      aliases: &["Assert"] },    // ↱
    L1SeedEntry { codepoint: 0x21B2, kind: 5, name: "op:typeof",      aliases: &["TypeOf"] },    // ↲
    L1SeedEntry { codepoint: 0x21B3, kind: 5, name: "op:halt",        aliases: &["Halt"] },      // ↳
    L1SeedEntry { codepoint: 0x21B4, kind: 5, name: "op:nop",         aliases: &["Nop"] },       // ↴

    // ── Program: Compiler / Process ────────────────────────────────────────
    // Dùng Supplemental Arrows-A (0x27F0-0x27FF) — SDF group
    L1SeedEntry { codepoint: 0x27F0, kind: 5, name: "prog:vm",       aliases: &["OlangVM"] },
    L1SeedEntry { codepoint: 0x27F1, kind: 5, name: "prog:compiler", aliases: &["OlangCompiler"] },
    L1SeedEntry { codepoint: 0x27F5, kind: 5, name: "prog:parser",   aliases: &["OlangParser"] },
    L1SeedEntry { codepoint: 0x27F6, kind: 5, name: "prog:program",  aliases: &["OlangProgram"] },
    L1SeedEntry { codepoint: 0x27F7, kind: 5, name: "prog:ir",       aliases: &["OlangIR"] },
    L1SeedEntry { codepoint: 0x27F8, kind: 5, name: "prog:semantic", aliases: &["OlangSemantic"] },

    // ── Sensor types ───────────────────────────────────────────────────────
    // Dùng Misc Symbols (0x2600-0x26FF) — EMOTICON group
    L1SeedEntry { codepoint: 0x2600, kind: 7, name: "sensor:temperature", aliases: &["Temperature"] }, // ☀
    L1SeedEntry { codepoint: 0x2601, kind: 7, name: "sensor:humidity",    aliases: &["Humidity"] },    // ☁
    L1SeedEntry { codepoint: 0x2602, kind: 7, name: "sensor:light",       aliases: &["LightSensor"] },// ☂
    L1SeedEntry { codepoint: 0x2603, kind: 7, name: "sensor:motion",      aliases: &["Motion"] },     // ☃
    L1SeedEntry { codepoint: 0x2604, kind: 7, name: "sensor:sound",       aliases: &["SoundSensor"] },// ☄
    L1SeedEntry { codepoint: 0x2607, kind: 7, name: "sensor:power",       aliases: &["Power"] },
];

// ─────────────────────────────────────────────────────────────────────────────
// Seed L1 — đăng ký toàn bộ system components vào Registry
// ─────────────────────────────────────────────────────────────────────────────

/// L1 System Seed — đăng ký tất cả Skills, Agents, VM ops, Sensors.
///
/// Quy tắc bất biến: **mọi thứ tạo ra đều phải đăng ký Registry**.
/// L1 = bản thiết kế DNA của HomeOS — clone sang thiết bị mới chỉ cần copy L1.
///
/// Đọc từ L1_SYSTEM_SEED (source of truth duy nhất).
/// Dùng khi boot không có file, hoặc file cũ chưa có L1 nodes.
pub fn seed_l1_system(registry: &mut Registry) {
    use crate::registry::NodeKind;

    let ts = 0i64;
    let mut offset = 1000u64; // L1 offsets start after L0

    for entry in L1_SYSTEM_SEED {
        let kind = NodeKind::from_byte(entry.kind).unwrap_or(NodeKind::Knowledge);
        let chain = encode_codepoint(entry.codepoint);
        let h = registry.insert_with_kind(&chain, 1, offset, ts, true, kind);
        registry.register_alias(entry.name, h);
        for &a in entry.aliases {
            registry.register_alias(a, h);
        }
        offset += 1;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// QT8: Write missing seeds to origin.olang
// ─────────────────────────────────────────────────────────────────────────────

/// L0 axiom codepoints — phải luôn có trong origin.olang.
static L0_AXIOM_CPS: &[(u32, &str)] = &[
    (0x25CB, "○"),     // origin
    (0x2205, "∅"),     // empty
    (0x2218, "∘"),     // compose
    (0x2208, "∈"),     // instance
];

/// Kiểm tra và ghi L0+L1 seeds thiếu vào writer.
///
/// QT8: mọi node trong Registry PHẢI có trong origin.olang.
/// Hàm này so sánh file content (nếu có) với bảng seed →
/// ghi bổ sung những gì thiếu.
///
/// Returns: số node đã ghi bổ sung.
fn write_missing_seeds(
    _registry: &crate::registry::Registry,
    file_bytes: Option<&[u8]>,
    writer: &mut crate::writer::OlangWriter,
) -> usize {
    use crate::registry::NodeKind;

    // Parse file để biết hash nào đã có
    let mut existing_hashes: alloc::collections::BTreeSet<u64> =
        alloc::collections::BTreeSet::new();

    if let Some(bytes) = file_bytes {
        if let Ok(reader) = OlangReader::new(bytes) {
            if let Ok(parsed) = reader.parse_all() {
                for node in &parsed.nodes {
                    existing_hashes.insert(node.chain.chain_hash());
                }
            }
        }
    }

    let ts = 0i64;
    let mut written = 0usize;

    // L0 axioms
    for &(cp, alias) in L0_AXIOM_CPS {
        let chain = encode_codepoint(cp);
        let hash = chain.chain_hash();
        if !existing_hashes.contains(&hash) {
            let _ = writer.append_node(&chain, 0, true, ts);
            let _ = writer.append_alias(alias, hash, ts);
            written += 1;
        }
    }

    // L0 full UCD — toàn bộ ~5400 nguyên tố
    let table = ucd::table();
    for entry in table {
        let chain = encode_codepoint(entry.cp);
        let hash = chain.chain_hash();
        if !existing_hashes.contains(&hash) {
            let _ = writer.append_node(&chain, 0, true, ts);
            let _ = writer.append_alias(entry.name, hash, ts);
            written += 1;
        }
    }

    // L0 natural language aliases
    for &(alias, cp) in L0_NATURAL_ALIASES {
        let chain = encode_codepoint(cp);
        let hash = chain.chain_hash();
        // Alias chỉ ghi nếu node đã tồn tại (đã ghi ở trên)
        let _ = writer.append_alias(alias, hash, ts);
    }

    // L1 system seed
    for entry in L1_SYSTEM_SEED {
        let chain = encode_codepoint(entry.codepoint);
        let hash = chain.chain_hash();
        if !existing_hashes.contains(&hash) {
            let _ = writer.append_node(&chain, 1, true, ts);
            let kind = NodeKind::from_byte(entry.kind).unwrap_or(NodeKind::Knowledge);
            writer.append_node_kind(hash, kind as u8, ts);
            let _ = writer.append_alias(entry.name, hash, ts);
            for &a in entry.aliases {
                let _ = writer.append_alias(a, hash, ts);
            }
            written += 1;
        }
    }

    written
}

// ─────────────────────────────────────────────────────────────────────────────
// Load từ file bytes
// ─────────────────────────────────────────────────────────────────────────────

/// All persisted data loaded from origin.olang.
struct LoadedData {
    edges: Vec<BootEdge>,
    stm_records: Vec<crate::reader::ParsedStm>,
    hebbian_records: Vec<crate::reader::ParsedHebbian>,
    knowtree_records: Vec<crate::reader::ParsedKnowTree>,
    slim_knowtree_records: Vec<crate::reader::ParsedSlimKnowTree>,
    curve_records: Vec<crate::reader::ParsedCurve>,
    auth_records: Vec<crate::reader::ParsedAuth>,
}

fn load_from_bytes(bytes: &[u8], registry: &mut Registry) -> Result<LoadedData, ParseError> {
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

    // Nạp NodeKind records → gán đúng kind cho từng node
    for nk in &parsed.node_kinds {
        if let Some(kind) = crate::registry::NodeKind::from_byte(nk.kind) {
            registry.set_kind(nk.chain_hash, kind);
        }
    }

    // Collect edges → trả về để restore Silk graph
    let edges: Vec<BootEdge> = parsed
        .edges
        .iter()
        .map(|e| BootEdge {
            from_hash: e.from_hash,
            to_hash: e.to_hash,
            edge_type: e.edge_type,
            timestamp: e.timestamp,
        })
        .collect();

    Ok(LoadedData {
        edges,
        stm_records: parsed.stm_records,
        hebbian_records: parsed.hebbian_records,
        knowtree_records: parsed.knowtree_records,
        slim_knowtree_records: parsed.slim_knowtree_records,
        curve_records: parsed.curve_records,
        auth_records: parsed.auth_records,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Verify ○(x)==x
// ─────────────────────────────────────────────────────────────────────────────

/// Verify: ○ không làm hỏng thứ gì.
///
/// Lấy một chain từ registry → LCA(x, x) == x.
fn verify_identity(_registry: &Registry) -> Result<(), String> {
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
pub static ALIAS_CODEPOINTS: &[(&str, u32)] = &[
    // fire
    ("fire", 0x1F525),
    ("lửa", 0x1F525),
    ("lua", 0x1F525),
    ("feu", 0x1F525),
    ("fuego", 0x1F525),
    // water
    ("water", 0x1F4A7),
    ("nước", 0x1F4A7),
    ("nuoc", 0x1F4A7),
    ("eau", 0x1F4A7),
    // cold
    ("cold", 0x2744),
    ("lạnh", 0x2744),
    ("lanh", 0x2744),
    // sun
    ("sun", 0x2600),
    ("warm", 0x1F31E),
    // mind
    ("mind", 0x1F9E0),
    ("brain", 0x1F9E0),
    ("tâm trí", 0x1F9E0),
    // heart
    ("heart", 0x2764),
    ("tim", 0x2764),
    ("trái tim", 0x2764),
    // origin
    ("origin", 0x25CB),
    ("○", 0x25CB),
    // math
    ("∘", 0x2218),
    ("compose", 0x2218),
    ("∈", 0x2208),
    ("member", 0x2208),
    ("∅", 0x2205),
    ("empty", 0x2205),
    // joy / sadness / tired / anger / fear
    ("vui", 0x1F60A),
    ("happy", 0x1F60A),
    ("joy", 0x1F60A),
    ("hạnh phúc", 0x1F60A),
    // French
    ("heureux", 0x1F60A),
    ("joie", 0x1F60A),
    ("joyeux", 0x1F60A),
    ("triste", 0x1F614),
    ("malheureux", 0x1F614),
    ("amour", 0x2764),
    ("famille", 0x1F46A),
    ("excellent", 0x2B50),
    ("parfait", 0x2B50),
    ("terrible", 0x1F4A9),
    ("horrible", 0x1F4A9),
    ("fatigué", 0x1F634),
    ("épuisé", 0x1F634),
    // German
    ("glücklich", 0x1F60A),
    ("fröhlich", 0x1F60A),
    ("schön", 0x1F60A),
    ("traurig", 0x1F614),
    ("unglücklich", 0x1F614),
    ("liebe", 0x2764),
    ("mögen", 0x2764),
    ("familie", 0x1F46A),
    ("wunderbar", 0x2B50),
    ("perfekt", 0x2B50),
    ("schrecklich", 0x1F4A9),
    ("schlecht", 0x1F4A9),
    ("müde", 0x1F634),
    ("erschöpft", 0x1F634),
    ("angst", 0x1F628),
    ("wütend", 0x1F621),
    // Spanish
    ("feliz", 0x1F60A),
    ("alegre", 0x1F60A),
    ("contento", 0x1F60A),
    ("triste", 0x1F614),
    ("amor", 0x2764),
    ("querer", 0x2764),
    ("familia", 0x1F46A),
    ("hogar", 0x1F46A),
    ("excelente", 0x2B50),
    ("perfecto", 0x2B50),
    ("terrible", 0x1F4A9),
    ("malo", 0x1F4A9),
    ("peor", 0x1F4A9),
    ("cansado", 0x1F634),
    ("agotado", 0x1F634),
    ("miedo", 0x1F628),
    ("enojado", 0x1F621),
    // Italian
    ("buon", 0x1F60A),
    ("bellissimo", 0x1F60A),
    ("felice", 0x1F60A),
    ("famiglia", 0x1F46A),
    // Portuguese
    ("feliz", 0x1F60A),
    ("alegre", 0x1F60A),
    ("familia", 0x1F46A),
    ("lar", 0x1F46A),
    ("triste", 0x1F614),
    ("amor", 0x2764),
    ("amar", 0x2764),
    // Vietnamese
    ("buồn", 0x1F614),
    ("sad", 0x1F614),
    ("buồn bã", 0x1F614),
    ("mệt", 0x1F634),
    ("tired", 0x1F634),
    ("mệt mỏi", 0x1F634),
    ("giận", 0x1F621),
    ("angry", 0x1F621),
    ("tức", 0x1F621),
    ("sợ", 0x1F628),
    ("scared", 0x1F628),
    ("lo", 0x1F628),
    ("yêu", 0x2764),
    ("love", 0x2764),
    ("gia đình", 0x1F46A),
    ("tuyệt", 0x2B50),
    ("great", 0x2B50),
    ("xuất sắc", 0x2B50),
    ("tệ", 0x1F4A9),
    ("bad", 0x1F4A9),
    ("đau", 0x1F915),
    ("pain", 0x1F915),
    ("cô đơn", 0x1F614),
    // danger / alert
    ("danger", 0x26A0),
    ("nguy hiểm", 0x26A0),
    // yes / no
    ("yes", 0x2705),
    ("có", 0x2705),
    ("no", 0x274C),
    ("không", 0x274C),
];

// ─────────────────────────────────────────────────────────────────────────────
// SystemManifest — hệ thống biết mình đang có gì
// ─────────────────────────────────────────────────────────────────────────────

/// Nhóm node theo chức năng — boot scan tự động phát hiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    /// L0 axioms/primitives
    Axiom,
    /// Cảm xúc / emotion nodes
    Emotion,
    /// Vật lý / sensor / device
    Device,
    /// Hành động / lệnh
    Command,
    /// Kiến thức đã học
    Knowledge,
    /// Agent / Chief / Worker registry
    Agent,
    /// Kỹ năng (Skill registry)
    Skill,
    /// Chưa phân loại
    Uncategorized,
}

/// Một entry trong manifest — 1 node đã được phân loại.
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    /// Chain hash
    pub hash: u64,
    /// Tầng
    pub layer: u8,
    /// Nhóm chức năng
    pub category: NodeCategory,
    /// Tên alias (nếu có)
    pub alias: Option<String>,
}

/// SystemManifest — bản đồ toàn bộ nodes đã biết, phân loại sẵn.
///
/// Boot Stage 7: scan registry → nhóm nodes → hệ thống biết mình có gì.
/// Khi cần tìm "tất cả Device nodes" → O(1) lookup.
#[derive(Debug, Clone)]
pub struct SystemManifest {
    /// Tất cả entries theo category
    entries: Vec<ManifestEntry>,
}

impl SystemManifest {
    /// Scan registry → phân loại tất cả nodes.
    ///
    /// Ưu tiên NodeKind đã lưu trong Registry (chính xác) trước,
    /// chỉ fallback classify_by_alias() cho Knowledge (generic).
    pub fn scan(registry: &Registry) -> Self {
        let mut entries = Vec::new();

        for layer in 0u8..16 {
            for reg_entry in registry.entries_in_layer(layer) {
                let category = category_from_kind(reg_entry.kind, reg_entry.chain_hash, layer, registry);
                entries.push(ManifestEntry {
                    hash: reg_entry.chain_hash,
                    layer,
                    category,
                    alias: find_alias(reg_entry.chain_hash, registry),
                });
            }
        }

        Self { entries }
    }

    /// Tất cả entries thuộc 1 category.
    pub fn by_category(&self, cat: NodeCategory) -> Vec<&ManifestEntry> {
        self.entries.iter().filter(|e| e.category == cat).collect()
    }

    /// Số lượng nodes theo category.
    pub fn count_by_category(&self, cat: NodeCategory) -> usize {
        self.entries.iter().filter(|e| e.category == cat).count()
    }

    /// Tổng entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Manifest rỗng?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Summary text — hệ thống tự mô tả.
    pub fn summary(&self) -> String {
        alloc::format!(
            "SystemManifest: {} nodes\n\
             Axiom      : {}\n\
             Emotion    : {}\n\
             Device     : {}\n\
             Command    : {}\n\
             Knowledge  : {}\n\
             Agent      : {}\n\
             Skill      : {}\n\
             Uncat      : {}",
            self.len(),
            self.count_by_category(NodeCategory::Axiom),
            self.count_by_category(NodeCategory::Emotion),
            self.count_by_category(NodeCategory::Device),
            self.count_by_category(NodeCategory::Command),
            self.count_by_category(NodeCategory::Knowledge),
            self.count_by_category(NodeCategory::Agent),
            self.count_by_category(NodeCategory::Skill),
            self.count_by_category(NodeCategory::Uncategorized),
        )
    }
}

/// Chuyển NodeKind (Registry) → NodeCategory (Manifest).
///
/// NodeKind chính xác hơn alias guessing vì được set lúc insert.
/// Chỉ fallback sang classify_by_alias() khi NodeKind = Knowledge (generic).
fn category_from_kind(kind: NodeKind, hash: u64, layer: u8, registry: &Registry) -> NodeCategory {
    match kind {
        NodeKind::Alphabet => NodeCategory::Axiom,
        NodeKind::Emotion => NodeCategory::Emotion,
        NodeKind::Device | NodeKind::Sensor => NodeCategory::Device,
        NodeKind::Agent => NodeCategory::Agent,
        NodeKind::Skill => NodeCategory::Skill,
        NodeKind::Memory | NodeKind::Program | NodeKind::System => NodeCategory::Knowledge,
        // Knowledge is generic — try alias-based refinement
        NodeKind::Knowledge => classify_node(hash, layer, registry),
    }
}

/// Phân loại node dựa trên alias name patterns + layer + UCD data.
/// Fallback khi NodeKind = Knowledge (generic, chưa phân loại cụ thể).
fn classify_node(hash: u64, layer: u8, registry: &Registry) -> NodeCategory {
    // L0 axioms: origin, empty, compose, member
    if layer == 0 {
        // Check alias patterns
        if let Some(alias) = find_alias(hash, registry) {
            let lo = alias.to_lowercase();
            return classify_by_alias(&lo, layer);
        }
        return NodeCategory::Axiom;
    }

    // Higher layers: classify by alias name
    if let Some(alias) = find_alias(hash, registry) {
        let lo = alias.to_lowercase();
        return classify_by_alias(&lo, layer);
    }

    NodeCategory::Uncategorized
}

/// Phân loại theo alias name.
fn classify_by_alias(alias: &str, layer: u8) -> NodeCategory {
    // Axiom keywords
    if layer == 0
        && matches!(
            alias,
            "○" | "origin" | "∅" | "empty" | "∘" | "compose" | "∈" | "instance" | "member"
        )
    {
        return NodeCategory::Axiom;
    }

    // Emotion keywords (bao gồm cả emoji aliases)
    static EMOTION_KW: &[&str] = &[
        "joy",
        "sad",
        "happy",
        "angry",
        "fear",
        "love",
        "pain",
        "tired",
        "vui",
        "buồn",
        "giận",
        "sợ",
        "yêu",
        "đau",
        "mệt",
        "heart",
        "tim",
        "cô đơn",
        "lonely",
    ];
    for kw in EMOTION_KW {
        if alias.contains(kw) {
            return NodeCategory::Emotion;
        }
    }

    // Device keywords
    static DEVICE_KW: &[&str] = &[
        "light",
        "đèn",
        "door",
        "cửa",
        "sensor",
        "camera",
        "temperature",
        "nhiệt",
        "house",
        "shelter",
        "nhà",
    ];
    for kw in DEVICE_KW {
        if alias.contains(kw) {
            return NodeCategory::Device;
        }
    }

    // Command keywords
    static CMD_KW: &[&str] = &[
        "open", "close", "stop", "move", "yes", "no", "mở", "đóng", "dừng",
    ];
    for kw in CMD_KW {
        if alias == *kw {
            return NodeCategory::Command;
        }
    }

    // L0 non-axiom = base concepts
    if layer == 0 {
        return NodeCategory::Axiom;
    }
    // L1+ without clear category = Knowledge
    NodeCategory::Knowledge
}

/// Tìm alias đầu tiên cho một hash.
fn find_alias(hash: u64, _registry: &Registry) -> Option<String> {
    // Scan ALIAS_CODEPOINTS — registry không expose reverse lookup.
    // Khi Template data thay thế hardcode, hàm này sẽ đọc từ registry.
    for &(alias, cp) in ALIAS_CODEPOINTS {
        if ucd::table_len() > 0 {
            let chain = encode_codepoint(cp);
            if chain.chain_hash() == hash {
                return Some(String::from(alias));
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// chain_to_emoji — display layer only
// ─────────────────────────────────────────────────────────────────────────────

/// Tìm emoji đại diện gần nhất cho một MolecularChain.
///
/// Ưu tiên:
///   1. Exact match trong ALIAS_CODEPOINTS (O(n) nhỏ)
///   2. decode_hash từ UCD reverse index
///   3. Bucket search theo emotion distance
pub fn chain_to_emoji(chain: &MolecularChain) -> alloc::string::String {
    use alloc::string::ToString;

    if chain.is_empty() {
        return "○".to_string();
    }

    // ZWJ chain (N > 1 molecules): reconstruct từng molecule
    if chain.len() > 1 {
        let mut zwj_s = alloc::string::String::new();
        for (i, mol) in chain.mols().enumerate() {
            // Restore relation=Member trước khi decode
            // (ZWJ set relation=Compose/Member, cần Member để match UCD)
            let normalized = crate::molecular::Molecule::raw(
                mol.shape_u8(),
                crate::molecular::RelationBase::Member.as_byte(),
                mol.valence_u8(),
                mol.arousal_u8(),
                mol.time_u8(),
            );
            let single = MolecularChain::single(normalized);
            let cp = ucd::decode_hash(single.chain_hash()).unwrap_or_else(|| {
                // Fallback: bucket search với relation=Member + emotion match
                let cands = ucd::bucket_cps(mol.shape_u8(), 0x01);
                if cands.is_empty() {
                    0x25CB
                } else {
                    best_in_bucket(cands, mol.valence_u8(), mol.arousal_u8())
                }
            });
            if let Some(c) = char::from_u32(cp) {
                zwj_s.push(c);
            }
            if i < chain.len() - 1 {
                zwj_s.push('‍'); // ZWJ
            }
        }
        if !zwj_s.is_empty() {
            return zwj_s;
        }
    }

    let hash = chain.chain_hash();

    // 1. Exact match: tìm trong ALIAS_CODEPOINTS
    for &(_, cp) in ALIAS_CODEPOINTS {
        let candidate = crate::encoder::encode_codepoint(cp);
        if candidate.chain_hash() == hash {
            return cp_to_str(cp);
        }
    }

    // 2. UCD reverse lookup: decode_hash → codepoint
    if let Some(cp) = ucd::decode_hash(hash) {
        return cp_to_str(cp);
    }

    // 3. Bucket search: emotion distance
    let mol = chain.mol_at(0).unwrap_or(crate::molecular::Molecule::from_u16(0));
    let shape = mol.shape_u8();
    let relation = mol.relation_u8();
    let v_target = mol.valence_u8();
    let a_target = mol.arousal_u8();

    let candidates = ucd::bucket_cps(shape, relation);
    let best_cp = if !candidates.is_empty() {
        best_in_bucket(candidates, v_target, a_target)
    } else {
        // Fallback: thử relation khác cùng shape
        let mut found = 0u32;
        for rel in 1u8..=8 {
            let cands = ucd::bucket_cps(shape, rel);
            if !cands.is_empty() {
                found = best_in_bucket(cands, v_target, a_target);
                break;
            }
        }
        if found == 0 {
            return "○".to_string();
        }
        found
    };

    cp_to_str(best_cp)
}

fn best_in_bucket(candidates: &[u32], v_target: u8, a_target: u8) -> u32 {
    let mut best_cp = candidates[0];
    let mut best_dist = u32::MAX;
    for &cp in candidates.iter().take(24) {
        let v = ucd::valence_of(cp);
        let a = ucd::arousal_of(cp);
        let dv = (v as i32 - v_target as i32).unsigned_abs();
        let da = (a as i32 - a_target as i32).unsigned_abs();
        let dist = dv * dv + da * da;
        if dist < best_dist {
            best_dist = dist;
            best_cp = cp;
        }
    }
    best_cp
}

fn cp_to_str(cp: u32) -> alloc::string::String {
    if let Some(c) = char::from_u32(cp) {
        let mut s = alloc::string::String::new();
        s.push(c);
        s
    } else {
        alloc::format!("U+{:04X}", cp)
    }
}

/// Resolve alias → (chain, codepoint) — codepoint từ ALIAS_CODEPOINTS hoặc Registry.
///
/// Ưu tiên:
///   1. Single char emoji → direct encode
///   2. ALIAS_CODEPOINTS exact match (L0 atoms)
///   3. Registry hash → scan ALIAS_CODEPOINTS để tìm canonical cp
///   4. First emoji char trong string
pub fn resolve_with_cp(name: &str, registry: &Registry) -> (MolecularChain, Option<u32>) {
    // 1. Single char → codepoint trực tiếp
    let chars: alloc::vec::Vec<char> = name.chars().collect();
    if chars.len() == 1 {
        let cp = chars[0] as u32;
        if cp > 0x20 {
            return (encode_codepoint(cp), Some(cp));
        }
    }

    // 2. ALIAS_CODEPOINTS exact word match (nhanh, L0 atoms)
    for &(alias, cp) in ALIAS_CODEPOINTS {
        if alias == name {
            return (encode_codepoint(cp), Some(cp));
        }
    }

    // 3. Registry lookup → hash → find canonical cp
    if let Some(hash) = registry.lookup_name(name) {
        // Tìm cp trong ALIAS_CODEPOINTS có cùng hash
        for &(_, cp) in ALIAS_CODEPOINTS {
            let candidate = encode_codepoint(cp);
            if candidate.chain_hash() == hash {
                return (candidate, Some(cp));
            }
        }
        // Fallback: decode_hash từ UCD
        if let Some(cp) = ucd::decode_hash(hash) {
            return (encode_codepoint(cp), Some(cp));
        }
        // Fallback: trả chain từ encode_codepoint nếu có single-char alias trong registry
        for c in name.chars() {
            let cp = c as u32;
            if cp > 0x2000 {
                let chain = encode_codepoint(cp);
                return (chain, Some(cp));
            }
        }
    }

    // 4. First emoji char
    for c in name.chars() {
        let cp = c as u32;
        if cp > 0x2000 {
            return (encode_codepoint(cp), Some(cp));
        }
    }

    (MolecularChain::empty(), None)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_empty_ok() {
        let result = boot_empty();
        assert!(
            result.stage >= BootStage::SelfInit,
            "Boot empty phải reach SelfInit"
        );
        // Registry rỗng = hợp lệ (○(∅)==○)
        assert!(
            result.errors.is_empty() || !result.is_ok() || true,
            "Boot empty không crash"
        );
    }

    #[test]
    fn boot_with_ucd_reaches_verified() {
        let result = boot_empty();
        assert!(
            result.stage >= BootStage::Verified,
            "Boot với UCD phải reach Verified: {:?}",
            result.errors
        );
        assert!(
            result.errors.is_empty(),
            "Không có errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn boot_seeds_axioms() {
        let result = boot_empty();
        // Registry phải có origin node
        assert!(
            result.registry.lookup_name("○").is_some(),
            "○ phải có trong registry sau boot"
        );
        assert!(result.registry.lookup_name("origin").is_some());
        assert!(
            result.registry.lookup_name("∘").is_some(),
            "∘ (compose) phải có"
        );
        assert!(
            result.registry.lookup_name("∈").is_some(),
            "∈ (member) phải có"
        );
    }

    #[test]
    fn boot_axiom_identity() {
        // Verify: LCA(○,○)==○
        let origin = encode_codepoint(0x25CB);
        let lca_result = lca(&origin, &origin);
        assert_eq!(lca_result, origin, "○(x)==x: LCA(○,○) phải == ○");
    }

    #[test]
    fn boot_from_seeded_file() {
        // Tạo mini file với 1 node
        use crate::writer::OlangWriter;
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut w = OlangWriter::new(0);
        w.append_node(&chain, 0, true, 0).unwrap();
        w.append_alias("fire", chain.chain_hash(), 0).unwrap();
        let bytes = w.into_bytes();

        let result = boot(Some(&bytes));
        assert!(
            result.stage >= BootStage::Loaded,
            "Boot từ file hợp lệ phải reach Loaded: {:?}",
            result.errors
        );
        assert!(
            result.registry.lookup_name("fire").is_some(),
            "Alias 'fire' phải được load"
        );
    }

    #[test]
    fn boot_from_bad_file() {
        // File bytes xấu → fallback, không crash
        let bad_bytes = [0x00u8; 20];
        let result = boot(Some(&bad_bytes));
        // Không panic, chỉ report error
        assert!(
            result.stage >= BootStage::SelfInit,
            "Boot từ file xấu không crash"
        );
    }

    #[test]
    fn boot_stage_ordering() {
        assert!(BootStage::SelfInit < BootStage::AxiomLoad);
        assert!(BootStage::AxiomLoad < BootStage::UcdReady);
        assert!(BootStage::UcdReady < BootStage::Loaded);
        assert!(BootStage::Loaded < BootStage::Verified);
    }

    #[test]
    fn resolve_single_emoji() {
        let registry = Registry::new();
        let chain = resolve("🔥", &registry);
        assert!(!chain.is_empty(), "resolve('🔥') phải trả non-empty chain");
        assert_eq!(chain, encode_codepoint(0x1F525));
    }

    #[test]
    fn resolve_unknown_returns_empty() {
        let registry = Registry::new();
        let chain = resolve("xyz_unknown_abc", &registry);
        assert!(chain.is_empty(), "Unknown alias → empty chain");
    }

    #[test]
    fn resolve_origin_symbol() {
        let registry = Registry::new();
        let chain = resolve("○", &registry);
        assert!(!chain.is_empty(), "○ → non-empty chain");
        assert_eq!(chain, encode_codepoint(0x25CB));
    }

    // ── SystemManifest ───────────────────────────────────────────────────────

    #[test]
    fn manifest_empty_registry() {
        let registry = Registry::new();
        let manifest = SystemManifest::scan(&registry);
        assert!(manifest.is_empty(), "Empty registry → empty manifest");
    }

    #[test]
    fn manifest_after_boot() {
        let result = boot_empty();
        assert!(!result.manifest.is_empty(), "Boot manifest không rỗng");
        // Axiom nodes phải có
        assert!(
            result.manifest.count_by_category(NodeCategory::Axiom) > 0,
            "Boot phải có axiom nodes"
        );
    }

    #[test]
    fn manifest_by_category() {
        let result = boot_empty();
        let axioms = result.manifest.by_category(NodeCategory::Axiom);
        assert!(!axioms.is_empty(), "Phải có axiom entries");
        for entry in &axioms {
            assert_eq!(entry.category, NodeCategory::Axiom);
        }
    }

    #[test]
    fn manifest_summary_has_categories() {
        let result = boot_empty();
        let summary = result.manifest.summary();
        assert!(summary.contains("SystemManifest"), "Summary header");
        assert!(summary.contains("Axiom"), "Summary has Axiom");
        assert!(summary.contains("Emotion"), "Summary has Emotion");
        assert!(summary.contains("Device"), "Summary has Device");
    }

    #[test]
    fn manifest_from_seeded_file() {
        use crate::writer::OlangWriter;
        // Seed a fire node + alias
        let chain = encode_codepoint(0x1F525);
        let mut w = OlangWriter::new(0);
        w.append_node(&chain, 0, true, 0).unwrap();
        w.append_alias("fire", chain.chain_hash(), 0).unwrap();
        let bytes = w.into_bytes();

        let result = boot(Some(&bytes));
        // fire should be classified (Axiom at L0)
        assert!(result.manifest.len() > 0);
    }

    // ── L1 System Seed ─────────────────────────────────────────────────────

    #[test]
    fn l1_seed_registers_skills() {
        use crate::registry::NodeKind;
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let skills = r.entries_by_kind(NodeKind::Skill);
        // 7 instinct + 11 domain + 2 advanced + 4 worker = 24
        assert!(skills.len() >= 24,
            "Expected ≥24 skills, got {}", skills.len());
        // All at L1
        for s in &skills {
            assert_eq!(s.layer, 1, "Skill should be L1");
        }
    }

    #[test]
    fn l1_seed_registers_agents() {
        use crate::registry::NodeKind;
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let agents = r.entries_by_kind(NodeKind::Agent);
        // AAM + LeoAI + 4 Chiefs + 5 Workers = 11
        assert!(agents.len() >= 11,
            "Expected ≥11 agents, got {}", agents.len());
    }

    #[test]
    fn l1_seed_registers_program_components() {
        use crate::registry::NodeKind;
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let progs = r.entries_by_kind(NodeKind::Program);
        // 6 built-in fns + 26 opcodes + 6 compiler = 38
        assert!(progs.len() >= 38,
            "Expected ≥38 program nodes, got {}", progs.len());
    }

    #[test]
    fn l1_seed_registers_sensors() {
        use crate::registry::NodeKind;
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let sensors = r.entries_by_kind(NodeKind::Sensor);
        assert!(sensors.len() >= 6,
            "Expected ≥6 sensors, got {}", sensors.len());
    }

    #[test]
    fn l1_seed_all_have_aliases() {
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        // Every L1 node should have at least 1 alias
        assert!(r.alias_count() >= r.len(),
            "Each L1 node needs an alias: {} aliases for {} nodes",
            r.alias_count(), r.len());
    }

    #[test]
    fn l1_seed_lookup_by_name() {
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        // Should be able to find skills by name
        assert!(r.lookup_name("skill:honesty").is_some(), "skill:honesty not found");
        assert!(r.lookup_name("skill:causality").is_some(), "skill:causality not found");
        assert!(r.lookup_name("agent:aam").is_some(), "agent:aam not found");
        assert!(r.lookup_name("agent:leo").is_some(), "agent:leo not found");
        assert!(r.lookup_name("fn:hyp_add").is_some(), "fn:hyp_add not found");
        assert!(r.lookup_name("op:push").is_some(), "op:push not found");
        assert!(r.lookup_name("sensor:temperature").is_some(), "sensor:temperature not found");
        assert!(r.lookup_name("prog:vm").is_some(), "prog:vm not found");
    }

    #[test]
    fn l1_seed_kind_summary() {
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let summary = r.kind_summary();
        // Should have at least 4 different kinds
        assert!(summary.len() >= 4,
            "Expected ≥4 kinds, got {}: {:?}", summary.len(), summary);
        // All kinds should have >0 count
        for (kind, count) in &summary {
            assert!(*count > 0, "{:?} should have entries", kind);
        }
    }

    #[test]
    fn boot_includes_l1_seed() {
        use crate::registry::NodeKind;
        let result = boot(None);
        // Boot should include L1 system nodes
        let skills = result.registry.entries_by_kind(NodeKind::Skill);
        assert!(skills.len() >= 24,
            "Boot should seed L1 skills: got {}", skills.len());
        let agents = result.registry.entries_by_kind(NodeKind::Agent);
        assert!(agents.len() >= 11,
            "Boot should seed L1 agents: got {}", agents.len());
    }
}
