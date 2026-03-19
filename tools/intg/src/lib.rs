//! # intg — Integration Test Suite
//!
//! Cross-crate integration tests cho HomeOS.
//! Mỗi test file kiểm tra mối nối giữa 2+ crate.
//!
//! Không mock — dùng API thật từ các crate.

/// Tạo HomeRuntime mới với seed cố định cho tests.
pub fn create_test_runtime() -> runtime::origin::HomeRuntime {
    runtime::origin::HomeRuntime::new(42)
}

/// Encode codepoint → chain, trả (chain, hash).
pub fn encode_and_hash(cp: u32) -> (olang::molecular::MolecularChain, u64) {
    let chain = olang::encoder::encode_codepoint(cp);
    let hash = chain.chain_hash();
    (chain, hash)
}

/// Tạo MolSummary từ codepoint (cho Silk tests).
pub fn mol_summary_of(cp: u32) -> silk::MolSummary {
    silk::MolSummary {
        shape: ucd::shape_of(cp),
        relation: ucd::relation_of(cp),
        valence: ucd::valence_of(cp),
        arousal: ucd::arousal_of(cp),
        time: ucd::time_of(cp),
    }
}

/// Well-known codepoints cho tests.
pub mod codepoints {
    pub const FIRE: u32 = 0x1F525;       // 🔥
    pub const SPHERE: u32 = 0x25CF;       // ●
    pub const MEMBER: u32 = 0x2208;       // ∈
    pub const ARROW: u32 = 0x2192;        // →
    pub const TORUS: u32 = 0x25CB;        // ○
    pub const HAPPY: u32 = 0x1F600;       // 😀
    pub const SAD: u32 = 0x1F622;         // 😢
    pub const HEART: u32 = 0x2764;        // ❤ (if in UCD)
    pub const DROPLET: u32 = 0x1F4A7;     // 💧
    pub const MUSICAL: u32 = 0x2669;      // ♩
}
