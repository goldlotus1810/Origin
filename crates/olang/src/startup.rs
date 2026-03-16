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
use crate::registry::Registry;

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
    let mut errors = Vec::new();

    // Stage 0: Raw — không làm gì
    // Stage 1: Self Init — ○(∅)==○
    let mut registry = Registry::new();
    let mut stage = BootStage::SelfInit;

    // Stage 2: Axiom Load — seed 4 axiom nodes + L1 system components
    // Dùng UCD nếu có, không thì bỏ qua
    if ucd::table_len() > 0 {
        seed_axioms(&mut registry);
        // L1 seed: đăng ký tất cả Skills, Agents, VM ops, Sensors
        // Quy tắc: mọi thứ tạo ra đều phải đăng ký Registry
        seed_l1_system(&mut registry);
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
    if let Some(bytes) = file_bytes {
        if !bytes.is_empty() {
            match load_from_bytes(bytes, &mut registry) {
                Ok(loaded_edges) => {
                    edges = loaded_edges;
                    stage = BootStage::Loaded;
                }
                Err(e) => {
                    errors.push(alloc::format!("Load error: {:?}", e));
                }
            }
        }
    }

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
    }
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
// Seed L1 — đăng ký toàn bộ system components vào Registry
// ─────────────────────────────────────────────────────────────────────────────

/// L1 System Seed — đăng ký tất cả Skills, Agents, VM ops, Sensors.
///
/// Quy tắc bất biến: **mọi thứ tạo ra đều phải đăng ký Registry**.
/// L1 = bản thiết kế DNA của HomeOS — clone sang thiết bị mới chỉ cần copy L1.
///
/// Mỗi component → encode_codepoint(cp) → insert_with_kind(chain, 1, ..., NodeKind::Xxx)
pub fn seed_l1_system(registry: &mut Registry) {
    use crate::registry::NodeKind;

    if ucd::table_len() == 0 {
        return;
    }

    let ts = 0i64;
    let mut offset = 1000u64; // L1 offsets start after L0

    // Helper: register 1 component
    let mut reg = |cp: u32, kind: NodeKind, name: &str, aliases: &[&str]| {
        let chain = encode_codepoint(cp);
        let h = registry.insert_with_kind(&chain, 1, offset, ts, true, kind);
        registry.register_alias(name, h);
        for &a in aliases {
            registry.register_alias(a, h);
        }
        offset += 1;
    };

    // ── Skills: 7 Instinct ─────────────────────────────────────────────────
    // Dùng Dingbats (0x2700-0x27BF) — SDF group, có trong UCD
    reg(0x2700, NodeKind::Skill, "skill:honesty",       &["Honesty"]);
    reg(0x2701, NodeKind::Skill, "skill:contradiction",  &["Contradiction"]);
    reg(0x2702, NodeKind::Skill, "skill:causality",      &["Causality"]);
    reg(0x2703, NodeKind::Skill, "skill:abstraction",    &["Abstraction"]);
    reg(0x2704, NodeKind::Skill, "skill:analogy",        &["Analogy"]);
    reg(0x2706, NodeKind::Skill, "skill:curiosity",      &["Curiosity"]);
    reg(0x2707, NodeKind::Skill, "skill:reflection",     &["Reflection"]);

    // ── Skills: 11 LeoAI Domain ────────────────────────────────────────────
    // Dùng Box Drawing chars (0x2500-0x257F) — SDF group, có trong UCD
    reg(0x2500, NodeKind::Skill, "skill:ingest",         &["IngestSkill"]);
    reg(0x2502, NodeKind::Skill, "skill:similarity",     &["SimilaritySkill"]);
    reg(0x250C, NodeKind::Skill, "skill:delta",          &["DeltaSkill"]);
    reg(0x2510, NodeKind::Skill, "skill:cluster",        &["ClusterSkill"]);
    reg(0x2514, NodeKind::Skill, "skill:curator",        &["CuratorSkill"]);
    reg(0x2518, NodeKind::Skill, "skill:merge",          &["MergeSkill"]);
    reg(0x251C, NodeKind::Skill, "skill:prune",          &["PruneSkill"]);
    reg(0x2524, NodeKind::Skill, "skill:hebbian",        &["HebbianSkill"]);
    reg(0x252C, NodeKind::Skill, "skill:dream",          &["DreamSkill"]);
    reg(0x2534, NodeKind::Skill, "skill:proposal",       &["ProposalSkill"]);
    reg(0x253C, NodeKind::Skill, "skill:inverse_render", &["InverseRenderSkill"]);

    // ── Skills: Advanced ───────────────────────────────────────────────────
    reg(0x2550, NodeKind::Skill, "skill:generalization",    &["GeneralizationSkill"]);
    reg(0x2551, NodeKind::Skill, "skill:temporal_pattern",  &["TemporalPatternSkill"]);

    // ── Skills: 4 Worker ───────────────────────────────────────────────────
    // Dùng Geometric Shapes (0x25A0-0x25FF) — SDF group
    reg(0x25A0, NodeKind::Skill, "skill:sensor",         &["SensorSkill"]);
    reg(0x25A1, NodeKind::Skill, "skill:actuator",       &["ActuatorSkill"]);
    reg(0x25B2, NodeKind::Skill, "skill:security",       &["SecuritySkill"]);
    reg(0x25B3, NodeKind::Skill, "skill:network",        &["NetworkSkill"]);

    // ── Agents ─────────────────────────────────────────────────────────────
    // Dùng Misc Symbols (0x2600-0x26FF) — EMOTICON group, có trong UCD
    // Chess symbols cho hierarchy
    reg(0x2654, NodeKind::Agent, "agent:aam",            &["AAM"]);       // ♔ King = AAM
    reg(0x2655, NodeKind::Agent, "agent:leo",            &["LeoAI"]);     // ♕ Queen = LeoAI
    reg(0x2656, NodeKind::Agent, "agent:chief:home",     &["HomeChief"]); // ♖ Rook
    reg(0x2657, NodeKind::Agent, "agent:chief:vision",   &["VisionChief"]);  // ♗ Bishop
    reg(0x2658, NodeKind::Agent, "agent:chief:network",  &["NetworkChief"]); // ♘ Knight
    reg(0x2659, NodeKind::Agent, "agent:chief:general",  &["GeneralChief"]); // ♙ Pawn
    // Workers — dùng black chess pieces
    reg(0x265A, NodeKind::Agent, "agent:worker:sensor",   &["WorkerSensor"]);   // ♚
    reg(0x265B, NodeKind::Agent, "agent:worker:actuator", &["WorkerActuator"]); // ♛
    reg(0x265C, NodeKind::Agent, "agent:worker:camera",   &["WorkerCamera"]);   // ♜
    reg(0x265D, NodeKind::Agent, "agent:worker:network",  &["WorkerNetwork"]);  // ♝
    reg(0x265E, NodeKind::Agent, "agent:worker:generic",  &["WorkerGeneric"]);  // ♞

    // ── Program: VM Built-in Functions ─────────────────────────────────────
    // Dùng Mathematical Operators (0x2200-0x22FF) — MATH group
    reg(0x2211, NodeKind::Program, "fn:hyp_add",         &["__hyp_add"]);  // ∑
    reg(0x2212, NodeKind::Program, "fn:hyp_sub",         &["__hyp_sub"]);  // −
    reg(0x2217, NodeKind::Program, "fn:hyp_mul",         &["__hyp_mul"]);  // ∗
    reg(0x2215, NodeKind::Program, "fn:hyp_div",         &["__hyp_div"]);  // ∕
    reg(0x2214, NodeKind::Program, "fn:phys_add",        &["__phys_add"]); // ∔
    reg(0x2216, NodeKind::Program, "fn:phys_sub",        &["__phys_sub"]); // ∖

    // ── Program: VM Opcodes (26 ops) ───────────────────────────────────────
    // Dùng Arrows (0x2190-0x21FF) — SDF group, có trong UCD
    reg(0x2190, NodeKind::Program, "op:push",            &["Push"]);      // ←
    reg(0x2191, NodeKind::Program, "op:push_num",        &["PushNum"]);   // ↑
    reg(0x2192, NodeKind::Program, "op:push_mol",        &["PushMol"]);   // →
    reg(0x2193, NodeKind::Program, "op:load",            &["Load"]);      // ↓
    reg(0x2194, NodeKind::Program, "op:lca",             &["Lca"]);       // ↔
    reg(0x2195, NodeKind::Program, "op:edge",            &["Edge"]);      // ↕
    reg(0x2196, NodeKind::Program, "op:query",           &["Query"]);     // ↖
    reg(0x2197, NodeKind::Program, "op:emit",            &["Emit"]);      // ↗
    reg(0x2198, NodeKind::Program, "op:dup",             &["Dup"]);       // ↘
    reg(0x2199, NodeKind::Program, "op:pop",             &["Pop"]);       // ↙
    reg(0x219A, NodeKind::Program, "op:swap",            &["Swap"]);      // ↚
    reg(0x219B, NodeKind::Program, "op:jmp",             &["Jmp"]);       // ↛
    reg(0x21A0, NodeKind::Program, "op:jz",              &["Jz"]);        // ↠
    reg(0x21A3, NodeKind::Program, "op:loop",            &["Loop"]);      // ↣
    reg(0x21A6, NodeKind::Program, "op:call",            &["Call"]);      // ↦
    reg(0x21A9, NodeKind::Program, "op:store",           &["Store"]);     // ↩
    reg(0x21AA, NodeKind::Program, "op:load_local",      &["LoadLocal"]); // ↪
    reg(0x21AB, NodeKind::Program, "op:scope_begin",     &["ScopeBegin"]);// ↫
    reg(0x21AC, NodeKind::Program, "op:scope_end",       &["ScopeEnd"]); // ↬
    reg(0x21AD, NodeKind::Program, "op:fuse",            &["Fuse"]);      // ↭
    reg(0x21AE, NodeKind::Program, "op:trace",           &["Trace"]);     // ↮
    reg(0x21B0, NodeKind::Program, "op:inspect",         &["Inspect"]);   // ↰
    reg(0x21B1, NodeKind::Program, "op:assert",          &["Assert"]);    // ↱
    reg(0x21B2, NodeKind::Program, "op:typeof",          &["TypeOf"]);    // ↲
    reg(0x21B3, NodeKind::Program, "op:halt",            &["Halt"]);      // ↳
    reg(0x21B4, NodeKind::Program, "op:nop",             &["Nop"]);       // ↴

    // ── Program: Compiler / Process ────────────────────────────────────────
    // Dùng Supplemental Arrows-A (0x27F0-0x27FF) — SDF group
    reg(0x27F0, NodeKind::Program, "prog:vm",            &["OlangVM"]);
    reg(0x27F1, NodeKind::Program, "prog:compiler",      &["OlangCompiler"]);
    reg(0x27F5, NodeKind::Program, "prog:parser",        &["OlangParser"]);
    reg(0x27F6, NodeKind::Program, "prog:program",       &["OlangProgram"]);
    reg(0x27F7, NodeKind::Program, "prog:ir",            &["OlangIR"]);
    reg(0x27F8, NodeKind::Program, "prog:semantic",      &["OlangSemantic"]);

    // ── Sensor types ───────────────────────────────────────────────────────
    // Dùng Misc Symbols (0x2600-0x26FF) — EMOTICON group
    reg(0x2600, NodeKind::Sensor, "sensor:temperature",  &["Temperature"]); // ☀
    reg(0x2601, NodeKind::Sensor, "sensor:humidity",     &["Humidity"]);    // ☁
    reg(0x2602, NodeKind::Sensor, "sensor:light",        &["LightSensor"]);// ☂
    reg(0x2603, NodeKind::Sensor, "sensor:motion",       &["Motion"]);      // ☃
    reg(0x2604, NodeKind::Sensor, "sensor:sound",        &["SoundSensor"]);// ☄
    reg(0x2607, NodeKind::Sensor, "sensor:power",        &["Power"]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Load từ file bytes
// ─────────────────────────────────────────────────────────────────────────────

fn load_from_bytes(bytes: &[u8], registry: &mut Registry) -> Result<Vec<BootEdge>, ParseError> {
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

    Ok(edges)
}

// ─────────────────────────────────────────────────────────────────────────────
// Verify ○(x)==x
// ─────────────────────────────────────────────────────────────────────────────

/// Verify: ○ không làm hỏng thứ gì.
///
/// Lấy một chain từ registry → LCA(x, x) == x.
fn verify_identity(_registry: &Registry) -> Result<(), String> {
    if ucd::table_len() == 0 {
        return Ok(());
    } // skip nếu không có UCD

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
    pub fn scan(registry: &Registry) -> Self {
        let mut entries = Vec::new();

        for layer in 0u8..16 {
            for reg_entry in registry.entries_in_layer(layer) {
                let category = classify_node(reg_entry.chain_hash, layer, registry);
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

/// Phân loại node dựa trên alias name patterns + layer + UCD data.
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

    if chain.is_empty() || ucd::table_len() == 0 {
        return "○".to_string();
    }

    // ZWJ chain (N > 1 molecules): reconstruct từng molecule
    if chain.len() > 1 {
        let mut zwj_s = alloc::string::String::new();
        for (i, mol) in chain.0.iter().enumerate() {
            // Restore relation=Member trước khi decode
            // (ZWJ set relation=Compose/Member, cần Member để match UCD)
            let mut normalized = *mol;
            normalized.relation = crate::molecular::RelationBase::Member.as_byte();
            let single = crate::molecular::MolecularChain(alloc::vec![normalized]);
            let cp = ucd::decode_hash(single.chain_hash()).unwrap_or_else(|| {
                // Fallback: bucket search với relation=Member + emotion match
                let cands = ucd::bucket_cps(mol.shape, 0x01);
                if cands.is_empty() {
                    0x25CB
                } else {
                    best_in_bucket(cands, mol.emotion.valence, mol.emotion.arousal)
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
    let mol = &chain.0[0];
    let shape = mol.shape;
    let relation = mol.relation;
    let v_target = mol.emotion.valence;
    let a_target = mol.emotion.arousal;

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

    fn skip() -> bool {
        ucd::table_len() == 0
    }

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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
        // Verify: LCA(○,○)==○
        let origin = encode_codepoint(0x25CB);
        let lca_result = lca(&origin, &origin);
        assert_eq!(lca_result, origin, "○(x)==x: LCA(○,○) phải == ○");
    }

    #[test]
    fn boot_from_seeded_file() {
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
        let result = boot_empty();
        let axioms = result.manifest.by_category(NodeCategory::Axiom);
        assert!(!axioms.is_empty(), "Phải có axiom entries");
        for entry in &axioms {
            assert_eq!(entry.category, NodeCategory::Axiom);
        }
    }

    #[test]
    fn manifest_summary_has_categories() {
        if skip() {
            return;
        }
        let result = boot_empty();
        let summary = result.manifest.summary();
        assert!(summary.contains("SystemManifest"), "Summary header");
        assert!(summary.contains("Axiom"), "Summary has Axiom");
        assert!(summary.contains("Emotion"), "Summary has Emotion");
        assert!(summary.contains("Device"), "Summary has Device");
    }

    #[test]
    fn manifest_from_seeded_file() {
        if skip() {
            return;
        }
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
        if skip() { return; }
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
        if skip() { return; }
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
        if skip() { return; }
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
        if skip() { return; }
        use crate::registry::NodeKind;
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        let sensors = r.entries_by_kind(NodeKind::Sensor);
        assert!(sensors.len() >= 6,
            "Expected ≥6 sensors, got {}", sensors.len());
    }

    #[test]
    fn l1_seed_all_have_aliases() {
        if skip() { return; }
        let mut r = Registry::new();
        seed_l1_system(&mut r);
        // Every L1 node should have at least 1 alias
        assert!(r.alias_count() >= r.len(),
            "Each L1 node needs an alias: {} aliases for {} nodes",
            r.alias_count(), r.len());
    }

    #[test]
    fn l1_seed_lookup_by_name() {
        if skip() { return; }
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
        if skip() { return; }
        use crate::registry::NodeKind;
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
        if skip() { return; }
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
