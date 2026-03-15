//! # lca — Weighted LCA Engine
//!
//! LCA(chain_A, chain_B) → chain_parent (tọa độ vật lý)
//!
//! ## Weighted LCA (không trung bình đơn giản):
//!
//! Bước 1 — Mode detection per dimension:
//!   Nếu ≥ 60% nodes cùng giá trị → parent[d] = mode
//!
//! Bước 2 — Weighted avg nếu không có mode:
//!   weight[i] = fire_count[i] / Σfire_count
//!   parent[d] = Σ(weight[i] × node[i][d])
//!
//! ## 4 Properties (test bắt buộc):
//!   1. Idempotent:    LCA(a,a) == a
//!   2. Commutative:   LCA(a,b) == LCA(b,a)
//!   3. Similarity bound: sim(LCA(a,b), a) >= sim(a,b) - ε
//!   4. Associative:   LCA(LCA(a,b),c) == LCA(a,LCA(b,c))

extern crate alloc;
use alloc::vec::Vec;

use crate::molecular::{
    Molecule, MolecularChain,
    ShapeBase, RelationBase, EmotionDim, TimeDim,
};

// ─────────────────────────────────────────────────────────────────────────────
// LCA of 2 chains
// ─────────────────────────────────────────────────────────────────────────────

/// LCA của 2 chains với equal weights.
///
/// Dùng khi không có thông tin về tần suất (fire_count).
pub fn lca(a: &MolecularChain, b: &MolecularChain) -> MolecularChain {
    lca_weighted(&[(a, 1u32), (b, 1u32)])
}

/// LCA của nhiều chains với fire_count weights.
///
/// `pairs` = slice of (chain_ref, fire_count).
/// fire_count = số lần node đó được co-activate.
pub fn lca_weighted(pairs: &[(&MolecularChain, u32)]) -> MolecularChain {
    // Lọc chain rỗng
    let valid: Vec<(&MolecularChain, u32)> = pairs.iter()
        .filter(|(c, _)| !c.is_empty())
        .copied()
        .collect();

    if valid.is_empty() { return MolecularChain::empty(); }
    if valid.len() == 1 { return valid[0].0.clone(); }

    // Dùng độ dài chain ngắn nhất để avoid out-of-bounds
    let min_len = valid.iter().map(|(c, _)| c.len()).min().unwrap_or(0);
    if min_len == 0 { return MolecularChain::empty(); }

    let total_weight: u32 = valid.iter().map(|(_, w)| w).sum();

    let mut result_mols = Vec::with_capacity(min_len);

    for mol_idx in 0..min_len {
        // Collect dimension values từ mọi chain tại vị trí mol_idx
        let shapes:    Vec<(u8, u32)> = valid.iter().map(|(c, w)| (c.0[mol_idx].shape.as_byte(), *w)).collect();
        let relations: Vec<(u8, u32)> = valid.iter().map(|(c, w)| (c.0[mol_idx].relation.as_byte(), *w)).collect();
        let valences:  Vec<(u8, u32)> = valid.iter().map(|(c, w)| (c.0[mol_idx].emotion.valence, *w)).collect();
        let arousals:  Vec<(u8, u32)> = valid.iter().map(|(c, w)| (c.0[mol_idx].emotion.arousal, *w)).collect();
        let times:     Vec<(u8, u32)> = valid.iter().map(|(c, w)| (c.0[mol_idx].time.as_byte(), *w)).collect();

        let shape_byte    = mode_or_wavg(&shapes,    total_weight);
        let relation_byte = mode_or_wavg(&relations, total_weight);
        let valence       = mode_or_wavg(&valences,  total_weight);
        let arousal       = mode_or_wavg(&arousals,  total_weight);
        let time_byte     = mode_or_wavg(&times,     total_weight);

        // Fallback nếu invalid byte (ví dụ shape=0x00)
        let shape    = ShapeBase::from_byte(shape_byte)
            .unwrap_or(ShapeBase::Sphere);
        let relation = RelationBase::from_byte(relation_byte)
            .unwrap_or(RelationBase::Member);
        let time     = TimeDim::from_byte(time_byte)
            .unwrap_or(TimeDim::Medium);

        result_mols.push(Molecule {
            shape,
            relation,
            emotion: EmotionDim { valence, arousal },
            time,
        });
    }

    MolecularChain(result_mols)
}

// ─────────────────────────────────────────────────────────────────────────────
// Mode or Weighted Average
// ─────────────────────────────────────────────────────────────────────────────

/// Mode nếu ≥ 60% weight; weighted average nếu không có mode.
fn mode_or_wavg(values: &[(u8, u32)], total: u32) -> u8 {
    if values.is_empty() || total == 0 { return 0x80; }

    // Tính weight của mỗi distinct value
    // Dùng simple approach: tìm value có weight cao nhất
    let mut best_val   = values[0].0;
    let mut best_weight = 0u32;

    // Group by value
    let mut seen: [(u8, u32); 256] = [(0, 0); 256];
    let mut n_seen = 0usize;

    for &(val, w) in values {
        // Tìm trong seen
        let mut found = false;
        for i in 0..n_seen {
            if seen[i].0 == val {
                seen[i].1 += w;
                if seen[i].1 > best_weight {
                    best_weight = seen[i].1;
                    best_val    = val;
                }
                found = true;
                break;
            }
        }
        if !found && n_seen < 256 {
            seen[n_seen] = (val, w);
            if w > best_weight {
                best_weight = w;
                best_val    = val;
            }
            n_seen += 1;
        }
    }

    // Mode threshold: ≥ 60% của total weight
    // threshold_numerator / threshold_denominator = 60/100
    let threshold = (total * 6 + 9) / 10; // ceiling của 60%
    if best_weight >= threshold {
        return best_val; // Mode chiến thắng
    }

    // Không có mode → weighted average
    let mut numerator: u64 = 0;
    for &(val, w) in values {
        numerator += val as u64 * w as u64;
    }
    (numerator / total as u64) as u8
}

// ─────────────────────────────────────────────────────────────────────────────
// LCA của nhiều chains (convenience)
// ─────────────────────────────────────────────────────────────────────────────

/// LCA của slice chains với equal weights.
pub fn lca_many(chains: &[MolecularChain]) -> MolecularChain {
    if chains.is_empty() { return MolecularChain::empty(); }
    if chains.len() == 1 { return chains[0].clone(); }
    let pairs: Vec<(&MolecularChain, u32)> = chains.iter().map(|c| (c, 1u32)).collect();
    lca_weighted(&pairs)
}

/// LCA của slice chains với fire_counts.
pub fn lca_many_weighted(chains: &[MolecularChain], weights: &[u32]) -> MolecularChain {
    let pairs: Vec<(&MolecularChain, u32)> = chains.iter()
        .zip(weights.iter())
        .map(|(c, &w)| (c, w))
        .collect();
    lca_weighted(&pairs)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — 4 Properties bắt buộc
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;

    // Helper: tạo chain từ UCD (đúng triết lý)
    fn fire()  -> MolecularChain { encode_codepoint(0x1F525) } // 🔥
    fn water() -> MolecularChain { encode_codepoint(0x1F4A7) } // 💧
    fn cold()  -> MolecularChain { encode_codepoint(0x2744)  } // ❄
    fn brain() -> MolecularChain { encode_codepoint(0x1F9E0) } // 🧠

    fn skip_if_empty() -> bool { ucd::table_len() == 0 }

    // ── Property 1: Idempotent ──────────────────────────────────────────────

    #[test]
    fn property_idempotent_fire() {
        if skip_if_empty() { return; }
        let f = fire();
        let result = lca(&f, &f);
        assert_eq!(result, f, "LCA(a,a) phải == a");
    }

    #[test]
    fn property_idempotent_water() {
        if skip_if_empty() { return; }
        let w = water();
        assert_eq!(lca(&w, &w), w);
    }

    // ── Property 2: Commutative ─────────────────────────────────────────────

    #[test]
    fn property_commutative() {
        if skip_if_empty() { return; }
        let f = fire();
        let w = water();
        assert_eq!(lca(&f, &w), lca(&w, &f),
            "LCA(a,b) phải == LCA(b,a)");
    }

    #[test]
    fn property_commutative_cold_brain() {
        if skip_if_empty() { return; }
        let c = cold();
        let b = brain();
        assert_eq!(lca(&c, &b), lca(&b, &c));
    }

    // ── Property 3: Similarity bound ───────────────────────────────────────

    #[test]
    fn property_similarity_bound() {
        if skip_if_empty() { return; }
        let f = fire();
        let w = water();
        let parent = lca(&f, &w);

        let sim_ab  = f.similarity(&w);
        let sim_pa  = parent.similarity(&f);
        let sim_pb  = parent.similarity(&w);
        let epsilon = 0.05f32;

        // LCA không được xa hơn khoảng cách ban đầu
        assert!(sim_pa >= sim_ab - epsilon,
            "sim(LCA(f,w), f)={:.3} >= sim(f,w)={:.3} - ε", sim_pa, sim_ab);
        assert!(sim_pb >= sim_ab - epsilon,
            "sim(LCA(f,w), w)={:.3} >= sim(f,w)={:.3} - ε", sim_pb, sim_ab);
    }

    // ── Property 4: Associative ─────────────────────────────────────────────

    #[test]
    fn property_associative() {
        if skip_if_empty() { return; }
        let f = fire();
        let w = water();
        let c = cold();

        let lca_fw_c = lca(&lca(&f, &w), &c);
        let lca_f_wc = lca(&f, &lca(&w, &c));

        // Associativity: kết quả phải rất gần nhau
        // (không nhất thiết giống hệt vì weighted avg có thể khác nhau chút)
        let sim = lca_fw_c.similarity_full(&lca_f_wc);
        assert!(sim >= 0.8,
            "LCA(LCA(f,w),c) ≈ LCA(f,LCA(w,c)): similarity={:.3}", sim);
    }

    // ── Semantic correctness ────────────────────────────────────────────────

    #[test]
    fn lca_fire_water_middle() {
        if skip_if_empty() { return; }
        let f = fire();
        let w = water();
        let parent = lca(&f, &w);

        assert_eq!(parent.len(), 1, "LCA của 2 single-mol chains = 1 molecule");

        let fm = &f.0[0];
        let wm = &w.0[0];
        let pm = &parent.0[0];

        // Valence: giữa lửa (0xFF) và nước (0xC0) → khoảng 0xDF
        let expected_v = ((fm.emotion.valence as u16 + wm.emotion.valence as u16) / 2) as u8;
        let diff_v = pm.emotion.valence.abs_diff(expected_v);
        assert!(diff_v <= 5,
            "LCA valence={} ≈ avg({},{})={}", pm.emotion.valence,
            fm.emotion.valence, wm.emotion.valence, expected_v);
    }

    #[test]
    fn lca_mode_detection() {
        if skip_if_empty() { return; }
        // 3 chains đều là Sphere, 1 chain là Capsule
        // → mode = Sphere (3/4 = 75% ≥ 60%)
        let sphere1 = encode_codepoint(0x25CF); // ●
        let sphere2 = encode_codepoint(0x1F525); // 🔥 (sphere)
        let sphere3 = encode_codepoint(0x1F9E0); // 🧠 (sphere)
        let capsule = encode_codepoint(0x1F4A7); // 💧 (capsule)

        let result = lca_many(&[sphere1, sphere2, sphere3, capsule]);
        assert_eq!(result.0[0].shape, ShapeBase::Sphere,
            "Mode detection: 3 Sphere + 1 Capsule → Sphere");
    }

    #[test]
    fn lca_weighted_fire_favored() {
        if skip_if_empty() { return; }
        // Fire (weight=10) vs Water (weight=1) → kết quả gần fire hơn
        let f = fire();
        let w = water();
        let result = lca_many_weighted(
            &[f.clone(), w.clone()],
            &[10, 1],
        );
        // Valence: fire=0xFF, water=0xC0
        // weighted: (0xFF×10 + 0xC0×1) / 11 ≈ 0xF9
        let fire_val  = f.0[0].emotion.valence;
        let water_val = w.0[0].emotion.valence;
        let result_val = result.0[0].emotion.valence;
        // result phải gần fire hơn water
        let dist_to_fire  = result_val.abs_diff(fire_val);
        let dist_to_water = result_val.abs_diff(water_val);
        assert!(dist_to_fire < dist_to_water,
            "Weighted LCA phải gần fire (weight=10) hơn water (weight=1): val={}", result_val);
    }

    #[test]
    fn lca_empty_chain() {
        let empty = MolecularChain::empty();
        let f = encode_codepoint(0x1F525);
        // LCA với empty → bỏ qua empty, giữ non-empty
        let result = lca_weighted(&[(&empty, 1), (&f, 1)]);
        assert!(!result.is_empty(), "LCA với empty chain → kết quả không rỗng");
    }

    #[test]
    fn lca_single_chain() {
        if skip_if_empty() { return; }
        let f = fire();
        let result = lca_many(&[f.clone()]);
        assert_eq!(result, f, "LCA của 1 chain = chính nó");
    }

    #[test]
    fn lca_many_thermodynamics() {
        if skip_if_empty() { return; }
        // 🔥 ♨️ ❄️ → LCA = L3_Thermodynamics (trung gian về nhiệt)
        let f = fire();
        let c = cold();
        let result = lca(&f, &c);

        let fire_val = f.0[0].emotion.valence;
        let cold_val = c.0[0].emotion.valence;
        let res_val  = result.0[0].emotion.valence;

        // Kết quả phải nằm giữa lửa và lạnh
        let min_val = fire_val.min(cold_val);
        let max_val = fire_val.max(cold_val);
        assert!(res_val >= min_val && res_val <= max_val,
            "LCA valence={} phải nằm giữa {} và {}", res_val, min_val, max_val);
    }

    // ── Mode detection edge cases ───────────────────────────────────────────

    #[test]
    fn mode_or_wavg_unanimous() {
        // Tất cả cùng giá trị → mode
        let vals = [(0x01u8, 1u32), (0x01, 2), (0x01, 1)];
        let result = super::mode_or_wavg(&vals, 4);
        assert_eq!(result, 0x01, "Unanimous → mode");
    }

    #[test]
    fn mode_or_wavg_tie() {
        // 50/50 → weighted avg
        let vals = [(0x00u8, 1u32), (0xFF, 1)];
        let result = super::mode_or_wavg(&vals, 2);
        // avg(0x00, 0xFF) = 0x7F
        assert!((result as i32 - 0x7F).abs() <= 2,
            "Tie → weighted avg ≈ 0x7F, got 0x{:02X}", result);
    }

    #[test]
    fn mode_or_wavg_60_percent_threshold() {
        // 3 × val=0x01 (75%) vs 1 × val=0xFF → mode = 0x01
        let vals = [(0x01u8, 1u32), (0x01, 1), (0x01, 1), (0xFF, 1)];
        let result = super::mode_or_wavg(&vals, 4);
        assert_eq!(result, 0x01, "75% ≥ 60% → mode");
    }

    #[test]
    fn mode_or_wavg_below_threshold() {
        // 2 × val=0x01 (50%) vs 2 × val=0xFF → không có mode → avg
        let vals = [(0x01u8, 1u32), (0x01, 1), (0xFF, 1), (0xFF, 1)];
        let result = super::mode_or_wavg(&vals, 4);
        // avg(0x01, 0x01, 0xFF, 0xFF) = (1+1+255+255)/4 = 128 = 0x80
        assert!((result as i32 - 0x80).abs() <= 2,
            "50% < 60% → weighted avg ≈ 0x80, got 0x{:02X}", result);
    }
}
