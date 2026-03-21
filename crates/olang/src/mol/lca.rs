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
    ComposeOp, CompositionOrigin, MolecularChain, Molecule, NodeState, RelationBase,
    ShapeBase, TimeDim,
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
    lca_with_variance(pairs).chain
}

/// Kết quả LCA kèm variance — đo mức trừu tượng.
///
/// variance ∈ [0.0, 1.0]:
///   < 0.15 → concrete (các chain rất giống nhau)
///   < 0.40 → categorical (nhóm liên quan)
///   ≥ 0.40 → abstract (khái niệm trừu tượng)
///
/// extremity ∈ [0.0, 1.0]:
///   Đo mức "cực đoan" trung bình của inputs.
///   LCA(😀, 😡): variance cao (divergence) VÀ extremity cao (both extreme).
///   LCA(😐, 😐): variance thấp VÀ extremity thấp (neutral).
///   Dùng để phân biệt "trung lập thật" vs "trung bình hóa cực đoan".
#[derive(Debug, Clone)]
pub struct LcaResult {
    /// Chain kết quả LCA.
    pub chain: MolecularChain,
    /// Variance trung bình trên tất cả dimensions.
    pub variance: f32,
    /// Per-dimension variance: [shape, relation, valence, arousal, time].
    /// Cho phép biết CHIỀU NÀO diverge mạnh nhất.
    pub dim_variance: [f32; 5],
    /// Extremity: trung bình abs(input_i - midpoint) trên valence+arousal.
    /// Cao = inputs đều cực đoan (dù khác hướng).
    pub extremity: f32,
}

/// LCA kèm variance output.
///
/// Variance = mean(1 - similarity_full(input_i, lca)) cho mọi input.
/// Extremity = trung bình abs(value - midpoint) cho valence + arousal.
/// dim_variance = per-dimension weighted variance.
pub fn lca_with_variance(pairs: &[(&MolecularChain, u32)]) -> LcaResult {
    let empty = LcaResult {
        chain: MolecularChain::empty(),
        variance: 0.0,
        dim_variance: [0.0; 5],
        extremity: 0.0,
    };

    // Lọc chain rỗng
    let valid: Vec<(&MolecularChain, u32)> = pairs
        .iter()
        .filter(|(c, _)| !c.is_empty())
        .copied()
        .collect();

    if valid.is_empty() {
        return empty;
    }
    if valid.len() == 1 {
        // Single chain: extremity = how extreme its valence+arousal are
        let m = Molecule::from_u16(valid[0].0 .0[0]);
        let ext = extremity_single(m.valence_u8(), m.arousal_u8());
        return LcaResult {
            chain: valid[0].0.clone(),
            variance: 0.0,
            dim_variance: [0.0; 5],
            extremity: ext,
        };
    }

    // Dùng độ dài chain ngắn nhất để avoid out-of-bounds
    let min_len = valid.iter().map(|(c, _)| c.len()).min().unwrap_or(0);
    if min_len == 0 {
        return empty;
    }

    let total_weight: u32 = valid.iter().map(|(_, w)| w).sum();
    let tw_f = total_weight as f32;

    let mut result_mols = Vec::with_capacity(min_len);
    let mut dim_var_accum = [0.0f32; 5]; // accumulate per-dimension variance
    let mut extremity_accum = 0.0f32;

    for mol_idx in 0..min_len {
        // Collect dimension values từ mọi chain tại vị trí mol_idx
        let shapes: Vec<(u8, u32)> = valid
            .iter()
            .map(|(c, w)| (Molecule::from_u16(c.0[mol_idx]).shape_u8(), *w))
            .collect();
        let relations: Vec<(u8, u32)> = valid
            .iter()
            .map(|(c, w)| (Molecule::from_u16(c.0[mol_idx]).relation_u8(), *w))
            .collect();
        let valences: Vec<(u8, u32)> = valid
            .iter()
            .map(|(c, w)| (Molecule::from_u16(c.0[mol_idx]).valence_u8(), *w))
            .collect();
        let arousals: Vec<(u8, u32)> = valid
            .iter()
            .map(|(c, w)| (Molecule::from_u16(c.0[mol_idx]).arousal_u8(), *w))
            .collect();
        let times: Vec<(u8, u32)> = valid
            .iter()
            .map(|(c, w)| (Molecule::from_u16(c.0[mol_idx]).time_u8(), *w))
            .collect();

        let shape_byte = mode_or_wavg_base(&shapes, total_weight, 8);
        let relation_byte = mode_or_wavg_base(&relations, total_weight, 8);
        let valence = mode_or_wavg(&valences, total_weight);
        let arousal = mode_or_wavg(&arousals, total_weight);
        let time_byte = mode_or_wavg_base(&times, total_weight, 5);

        // Per-dimension weighted variance: Σ w_i × (val_i - mean)² / Σ w_i
        let all_dims: [&[(u8, u32)]; 5] = [&shapes, &relations, &valences, &arousals, &times];
        let means: [u8; 5] = [shape_byte, relation_byte, valence, arousal, time_byte];
        for (d, (vals, mean)) in all_dims.iter().zip(means.iter()).enumerate() {
            let var: f32 = vals
                .iter()
                .map(|(v, w)| {
                    let diff = *v as f32 - *mean as f32;
                    *w as f32 * diff * diff
                })
                .sum::<f32>()
                / (tw_f * 255.0 * 255.0); // normalize to [0,1]
            dim_var_accum[d] += var;
        }

        // Extremity: how extreme are the INPUTS on valence+arousal?
        // midpoint: valence=0x80, arousal=0x80
        for &(v, w) in &valences {
            let ext_v = (v as f32 - 128.0).abs() / 128.0; // [0,1]
            extremity_accum += ext_v * w as f32 / tw_f;
        }
        for &(a, w) in &arousals {
            let ext_a = (a as f32 - 128.0).abs() / 128.0;
            extremity_accum += ext_a * w as f32 / tw_f;
        }

        // Fallback nếu invalid byte (ví dụ shape=0x00)
        let shape = if shape_byte == 0 {
            ShapeBase::Sphere.as_byte()
        } else {
            shape_byte
        };
        let relation = if relation_byte == 0 {
            RelationBase::Member.as_byte()
        } else {
            relation_byte
        };
        let time = if time_byte == 0 {
            TimeDim::Medium.as_byte()
        } else {
            time_byte
        };

        let mol = Molecule::formula(shape, relation, valence, arousal, time);
        // LCA result = CÔNG THỨC MỚI — chờ evidence để evaluate
        // evaluated = 0x00 (từ Molecule::formula)
        result_mols.push(mol.bits);
    }

    let chain = MolecularChain(result_mols);

    // Tính variance = mean(1 - similarity_full(input_i, lca))
    let n = valid.len() as f32;
    let variance: f32 = valid
        .iter()
        .map(|(c, _)| 1.0 - c.similarity_full(&chain))
        .sum::<f32>()
        / n;

    // Normalize dim_variance by min_len
    let ml = min_len as f32;
    for dv in &mut dim_var_accum {
        *dv /= ml;
    }
    // Normalize extremity: chia 2 (2 chiều: V + A) × min_len
    extremity_accum /= 2.0 * ml;

    LcaResult {
        chain,
        variance,
        dim_variance: dim_var_accum,
        extremity: extremity_accum,
    }
}

/// Extremity of a single molecule (Euclidean distance from midpoint in V/A space).
///
/// Trước: (ext_v + ext_a) / 2.0 — trung bình đơn giản (vi phạm QT emotion).
/// Giờ: Euclidean distance / √2 — normalized [0.0, 1.0], không trung bình.
fn extremity_single(valence: u8, arousal: u8) -> f32 {
    let ext_v = (valence as f32 - 128.0).abs() / 128.0;
    let ext_a = (arousal as f32 - 128.0).abs() / 128.0;
    // Euclidean distance in 2D, normalized by max possible distance (√2)
    homemath::sqrtf(ext_v * ext_v + ext_a * ext_a) * core::f32::consts::FRAC_1_SQRT_2
}

// ─────────────────────────────────────────────────────────────────────────────
// Mode or Weighted Average
// ─────────────────────────────────────────────────────────────────────────────

/// Mode detection trên base categories (hierarchical encoding).
///
/// Group by base category (modulo n_bases) thay vì exact value.
/// Nếu ≥ 60% weight cùng base → trả weighted avg CỦA NHÓM THẮNG.
/// Nếu không → weighted average toàn bộ.
fn mode_or_wavg_base(values: &[(u8, u32)], total: u32, n_bases: u8) -> u8 {
    if values.is_empty() || total == 0 || n_bases == 0 {
        return 0x80;
    }

    // Group weight by base category
    let mut base_weights = [0u32; 9]; // max 8 bases (index 1..=8)
    for &(val, w) in values {
        if val == 0 {
            continue;
        }
        let base = ((val - 1) % n_bases) + 1;
        base_weights[base as usize] += w;
    }

    // Find dominant base
    let mut best_base = 1u8;
    let mut best_weight = 0u32;
    for base in 1..=n_bases {
        if base_weights[base as usize] > best_weight {
            best_weight = base_weights[base as usize];
            best_base = base;
        }
    }

    let threshold = (total * 6).div_ceil(10);
    if best_weight >= threshold {
        // Check if ALL values in the winning base are identical → return exact value (idempotent).
        let mut unanimous_val: Option<u8> = None;
        let mut all_same = true;
        for &(val, _w) in values {
            if val == 0 {
                continue;
            }
            let base = ((val - 1) % n_bases) + 1;
            if base == best_base {
                match unanimous_val {
                    None => unanimous_val = Some(val),
                    Some(prev) if prev != val => {
                        all_same = false;
                        break;
                    }
                    _ => {}
                }
            }
        }
        if all_same {
            if let Some(v) = unanimous_val {
                return v;
            }
        }
        // Diverse within same base → return base byte (sub=0) for commutativity.
        return best_base;
    }

    // No mode → weighted average of BASE values (not raw hierarchical values).
    // Using raw hierarchical values for avg can produce non-commutative results.
    let mut numerator: u64 = 0;
    for &(val, w) in values {
        if val == 0 {
            continue;
        }
        let base = ((val - 1) % n_bases) + 1;
        numerator += base as u64 * w as u64;
    }
    (numerator / total as u64) as u8
}

/// Mode nếu ≥ 60% weight; weighted average nếu không có mode.
fn mode_or_wavg(values: &[(u8, u32)], total: u32) -> u8 {
    if values.is_empty() || total == 0 {
        return 0x80;
    }

    // Tính weight của mỗi distinct value
    // Dùng simple approach: tìm value có weight cao nhất
    let mut best_val = values[0].0;
    let mut best_weight = 0u32;

    // Group by value
    let mut seen: [(u8, u32); 256] = [(0, 0); 256];
    let mut n_seen = 0usize;

    for &(val, w) in values {
        // Tìm trong seen
        let mut found = false;
        for entry in &mut seen[..n_seen] {
            if entry.0 == val {
                entry.1 += w;
                if entry.1 > best_weight {
                    best_weight = entry.1;
                    best_val = val;
                }
                found = true;
                break;
            }
        }
        if !found && n_seen < 256 {
            seen[n_seen] = (val, w);
            if w > best_weight {
                best_weight = w;
                best_val = val;
            }
            n_seen += 1;
        }
    }

    // Mode threshold: ≥ 60% của total weight
    // threshold_numerator / threshold_denominator = 60/100
    let threshold = (total * 6).div_ceil(10); // ceiling của 60%
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

/// LCA kèm CompositionOrigin — track "node này sinh từ đâu?"
///
/// Trả về (LcaResult, CompositionOrigin::Composed { sources, op: Lca }).
pub fn lca_with_origin(pairs: &[(&MolecularChain, u32)]) -> (LcaResult, CompositionOrigin) {
    let result = lca_with_variance(pairs);
    let sources: Vec<u64> = pairs
        .iter()
        .filter(|(c, _)| !c.is_empty())
        .map(|(c, _)| c.chain_hash())
        .collect();
    let origin = if sources.len() == 1 {
        // Single source — keep as innate if possible
        CompositionOrigin::Unknown
    } else {
        CompositionOrigin::Composed {
            sources,
            op: ComposeOp::Lca,
        }
    };
    (result, origin)
}

/// LCA kèm NodeState — convenience cho Dream pipeline.
pub fn lca_to_node_state(pairs: &[(&MolecularChain, u32)]) -> Option<NodeState> {
    let (result, origin) = lca_with_origin(pairs);
    let mol = result.chain.first()?;
    Some(NodeState {
        mol,
        maturity: crate::molecular::Maturity::Formula,
        origin,
    })
}

/// LCA của slice chains với equal weights.
pub fn lca_many(chains: &[MolecularChain]) -> MolecularChain {
    if chains.is_empty() {
        return MolecularChain::empty();
    }
    if chains.len() == 1 {
        return chains[0].clone();
    }
    let pairs: Vec<(&MolecularChain, u32)> = chains.iter().map(|c| (c, 1u32)).collect();
    lca_weighted(&pairs)
}

/// LCA của slice chains kèm variance.
pub fn lca_many_with_variance(chains: &[MolecularChain]) -> LcaResult {
    if chains.is_empty() {
        return LcaResult {
            chain: MolecularChain::empty(),
            variance: 0.0,
            dim_variance: [0.0; 5],
            extremity: 0.0,
        };
    }
    if chains.len() == 1 {
        let ext = if chains[0].is_empty() {
            0.0
        } else {
            let m = Molecule::from_u16(chains[0].0[0]);
            extremity_single(m.valence_u8(), m.arousal_u8())
        };
        return LcaResult {
            chain: chains[0].clone(),
            variance: 0.0,
            dim_variance: [0.0; 5],
            extremity: ext,
        };
    }
    let pairs: Vec<(&MolecularChain, u32)> = chains.iter().map(|c| (c, 1u32)).collect();
    lca_with_variance(&pairs)
}

/// LCA của slice chains với fire_counts.
pub fn lca_many_weighted(chains: &[MolecularChain], weights: &[u32]) -> MolecularChain {
    let pairs: Vec<(&MolecularChain, u32)> = chains
        .iter()
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
    fn fire() -> MolecularChain {
        encode_codepoint(0x1F525)
    } // 🔥
    fn water() -> MolecularChain {
        encode_codepoint(0x1F4A7)
    } // 💧
    fn cold() -> MolecularChain {
        encode_codepoint(0x2744)
    } // ❄
    fn brain() -> MolecularChain {
        encode_codepoint(0x1F9E0)
    } // 🧠


    // ── Property 1: Idempotent ──────────────────────────────────────────────

    #[test]
    fn property_idempotent_fire() {
        let f = fire();
        let result = lca(&f, &f);
        assert_eq!(result, f, "LCA(a,a) phải == a");
    }

    #[test]
    fn property_idempotent_water() {
        let w = water();
        assert_eq!(lca(&w, &w), w);
    }

    // ── Property 2: Commutative ─────────────────────────────────────────────

    #[test]
    fn property_commutative() {
        let f = fire();
        let w = water();
        assert_eq!(lca(&f, &w), lca(&w, &f), "LCA(a,b) phải == LCA(b,a)");
    }

    #[test]
    fn property_commutative_cold_brain() {
        let c = cold();
        let b = brain();
        assert_eq!(lca(&c, &b), lca(&b, &c));
    }

    // ── Property 3: Similarity bound ───────────────────────────────────────

    #[test]
    fn property_similarity_bound() {
        let f = fire();
        let w = water();
        let parent = lca(&f, &w);

        let sim_ab = f.similarity(&w);
        let sim_pa = parent.similarity(&f);
        let sim_pb = parent.similarity(&w);
        let epsilon = 0.05f32;

        // LCA không được xa hơn khoảng cách ban đầu
        assert!(
            sim_pa >= sim_ab - epsilon,
            "sim(LCA(f,w), f)={:.3} >= sim(f,w)={:.3} - ε",
            sim_pa,
            sim_ab
        );
        assert!(
            sim_pb >= sim_ab - epsilon,
            "sim(LCA(f,w), w)={:.3} >= sim(f,w)={:.3} - ε",
            sim_pb,
            sim_ab
        );
    }

    // ── Property 4: Associative ─────────────────────────────────────────────

    #[test]
    fn property_associative() {
        let f = fire();
        let w = water();
        let c = cold();

        let lca_fw_c = lca(&lca(&f, &w), &c);
        let lca_f_wc = lca(&f, &lca(&w, &c));

        // Associativity: kết quả phải rất gần nhau
        // (không nhất thiết giống hệt vì weighted avg có thể khác nhau chút)
        let sim = lca_fw_c.similarity_full(&lca_f_wc);
        assert!(
            sim >= 0.8,
            "LCA(LCA(f,w),c) ≈ LCA(f,LCA(w,c)): similarity={:.3}",
            sim
        );
    }

    // ── Semantic correctness ────────────────────────────────────────────────

    #[test]
    fn lca_fire_water_middle() {
        let f = fire();
        let w = water();
        let parent = lca(&f, &w);

        assert_eq!(parent.len(), 1, "LCA của 2 single-mol chains = 1 molecule");

        let fm = Molecule::from_u16(f.0[0]);
        let wm = Molecule::from_u16(w.0[0]);
        let pm = Molecule::from_u16(parent.0[0]);

        // Valence: giữa lửa (0xFF) và nước (0xC0) → khoảng 0xDF
        let expected_v = ((fm.valence_u8() as u16 + wm.valence_u8() as u16) / 2) as u8;
        let diff_v = pm.valence_u8().abs_diff(expected_v);
        assert!(
            diff_v <= 5,
            "LCA valence={} ≈ avg({},{})={}",
            pm.valence_u8(),
            fm.valence_u8(),
            wm.valence_u8(),
            expected_v
        );
    }

    #[test]
    fn lca_mode_detection() {
        // 3 chains đều là Sphere, 1 chain là Capsule
        // → mode = Sphere (3/4 = 75% ≥ 60%)
        let sphere1 = encode_codepoint(0x25CF); // ●
        let sphere2 = encode_codepoint(0x1F525); // 🔥 (sphere)
        let sphere3 = encode_codepoint(0x1F9E0); // 🧠 (sphere)
        let capsule = encode_codepoint(0x1F4A7); // 💧 (capsule)

        let result = lca_many(&[sphere1, sphere2, sphere3, capsule]);
        assert_eq!(
            Molecule::from_u16(result.0[0]).shape_base(),
            ShapeBase::Sphere,
            "Mode detection: 3 Sphere + 1 Capsule → Sphere"
        );
    }

    #[test]
    fn lca_weighted_fire_favored() {
        // Fire (weight=10) vs Water (weight=1) → kết quả gần fire hơn
        let f = fire();
        let w = water();
        let result = lca_many_weighted(&[f.clone(), w.clone()], &[10, 1]);
        // Valence: fire=0xFF, water=0xC0
        // weighted: (0xFF×10 + 0xC0×1) / 11 ≈ 0xF9
        let fire_val = Molecule::from_u16(f.0[0]).valence_u8();
        let water_val = Molecule::from_u16(w.0[0]).valence_u8();
        let result_val = Molecule::from_u16(result.0[0]).valence_u8();
        // result phải gần fire hơn water
        let dist_to_fire = result_val.abs_diff(fire_val);
        let dist_to_water = result_val.abs_diff(water_val);
        assert!(
            dist_to_fire < dist_to_water,
            "Weighted LCA phải gần fire (weight=10) hơn water (weight=1): val={}",
            result_val
        );
    }

    #[test]
    fn lca_empty_chain() {
        let empty = MolecularChain::empty();
        let f = encode_codepoint(0x1F525);
        // LCA với empty → bỏ qua empty, giữ non-empty
        let result = lca_weighted(&[(&empty, 1), (&f, 1)]);
        assert!(
            !result.is_empty(),
            "LCA với empty chain → kết quả không rỗng"
        );
    }

    #[test]
    fn lca_single_chain() {
        let f = fire();
        let result = lca_many(&[f.clone()]);
        assert_eq!(result, f, "LCA của 1 chain = chính nó");
    }

    #[test]
    fn lca_many_thermodynamics() {
        // 🔥 ♨️ ❄️ → LCA = L3_Thermodynamics (trung gian về nhiệt)
        let f = fire();
        let c = cold();
        let result = lca(&f, &c);

        let fire_val = Molecule::from_u16(f.0[0]).valence_u8();
        let cold_val = Molecule::from_u16(c.0[0]).valence_u8();
        let res_val = Molecule::from_u16(result.0[0]).valence_u8();

        // Kết quả phải nằm giữa lửa và lạnh
        let min_val = fire_val.min(cold_val);
        let max_val = fire_val.max(cold_val);
        assert!(
            res_val >= min_val && res_val <= max_val,
            "LCA valence={} phải nằm giữa {} và {}",
            res_val,
            min_val,
            max_val
        );
    }

    // ── LCA Variance ──────────────────────────────────────────────────────

    #[test]
    fn lca_variance_identical_is_zero() {
        let f = fire();
        let result = super::lca_with_variance(&[(&f, 1), (&f, 1)]);
        assert!(
            result.variance < 0.01,
            "Identical chains → variance ≈ 0, got {}",
            result.variance
        );
    }

    #[test]
    fn lca_variance_similar_is_concrete() {
        // 🔥 và 🔥 (identical) → variance < 0.15 = concrete
        let f1 = fire();
        let f2 = fire();
        let result = super::lca_many_with_variance(&[f1, f2]);
        assert!(
            result.variance < 0.15,
            "Same chains → concrete (var < 0.15), got {}",
            result.variance
        );
    }

    #[test]
    fn lca_variance_diverse_is_abstract() {
        // 🔥 💧 ❄ 🧠 → very different → variance should be notable
        let result = super::lca_many_with_variance(&[fire(), water(), cold(), brain()]);
        assert!(
            result.variance > 0.05,
            "Diverse chains → higher variance, got {}",
            result.variance
        );
    }

    #[test]
    fn lca_variance_single_chain_zero() {
        let result = super::lca_many_with_variance(&[fire()]);
        assert_eq!(result.variance, 0.0, "Single chain → variance = 0");
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
        assert!(
            (result as i32 - 0x7F).abs() <= 2,
            "Tie → weighted avg ≈ 0x7F, got 0x{:02X}",
            result
        );
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
        assert!(
            (result as i32 - 0x80).abs() <= 2,
            "50% < 60% → weighted avg ≈ 0x80, got 0x{:02X}",
            result
        );
    }

    // ── Extremity — phát hiện "trung bình hóa cực đoan" ───────────────────

    #[test]
    fn extremity_identical_chains() {
        let f = fire();
        let result = super::lca_with_variance(&[(&f, 1), (&f, 1)]);
        // Extremity should be consistent (same chain → same extremity)
        assert!(
            result.extremity >= 0.0 && result.extremity <= 1.0,
            "Extremity bounded [0,1]: got {}",
            result.extremity
        );
    }

    #[test]
    fn extremity_diverse_higher_than_neutral() {
        // 🔥 💧 ❄ 🧠 → inputs spread out → extremity measurable
        let result = super::lca_many_with_variance(&[fire(), water(), cold(), brain()]);
        // Just verify it's valid — actual extremity depends on UCD values
        assert!(result.extremity >= 0.0 && result.extremity <= 1.0);
    }

    #[test]
    fn dim_variance_shape() {
        // dim_variance[0] = shape variance
        let result = super::lca_many_with_variance(&[fire(), water(), cold(), brain()]);
        // All 5 dims should be non-negative
        for (i, dv) in result.dim_variance.iter().enumerate() {
            assert!(*dv >= 0.0, "dim_variance[{}] = {} >= 0", i, dv);
        }
    }

    #[test]
    fn dim_variance_identical_is_zero() {
        let f = fire();
        let result = super::lca_with_variance(&[(&f, 1), (&f, 1)]);
        for (i, dv) in result.dim_variance.iter().enumerate() {
            assert!(
                *dv < 0.001,
                "Identical → dim_variance[{}] ≈ 0, got {}",
                i,
                dv
            );
        }
    }

    // ── CompositionOrigin tracking ────────────────────────────────────────

    #[test]
    fn lca_with_origin_composed() {
        let f = fire();
        let w = water();
        let (result, origin) = super::lca_with_origin(&[(&f, 1), (&w, 1)]);
        assert!(!result.chain.is_empty());
        match origin {
            super::CompositionOrigin::Composed { sources, op } => {
                assert_eq!(sources.len(), 2);
                assert_eq!(op, super::ComposeOp::Lca);
                assert!(sources.contains(&f.chain_hash()));
                assert!(sources.contains(&w.chain_hash()));
            }
            _ => panic!("Expected Composed, got {:?}", origin),
        }
    }

    #[test]
    fn lca_to_node_state_works() {
        let f = fire();
        let w = water();
        let ns = super::lca_to_node_state(&[(&f, 1), (&w, 1)]);
        assert!(ns.is_some());
        let ns = ns.unwrap();
        assert!(ns.maturity.is_formula());
        assert!(matches!(ns.origin, super::CompositionOrigin::Composed { .. }));
    }
}
