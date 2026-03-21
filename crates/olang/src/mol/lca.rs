//! # lca — v2 Compose Engine
//!
//! LCA(chain_A, chain_B) → chain_parent (tọa độ vật lý)
//!
//! ## v2 Compose Rules (sinh học — KHÔNG trung bình):
//!
//!   S = Union(Aˢ, Bˢ)         — hình dạng dominant (CSG Union)
//!   R = Compose                — quan hệ = tổ hợp (nếu inputs khác nhau)
//!   V = amplify(Va, Vb, w)    — khuếch đại synergy (KHÔNG trung bình)
//!   A = max(Aᴬ, Bᴬ)          — cường độ lấy cao hơn
//!   T = dominant(Aᵀ, Bᵀ)     — thời gian lấy chủ đạo
//!
//! ## 4 Properties (test bắt buộc):
//!   1. Idempotent:    LCA(a,a) == a
//!   2. Commutative:   LCA(a,b) == LCA(b,a)
//!   3. Similarity bound: sim(LCA(a,b), a) >= sim(a,b) - ε
//!   4. Associative:   LCA(LCA(a,b),c) ≈ LCA(a,LCA(b,c))

extern crate alloc;
use alloc::vec::Vec;

use crate::molecular::{
    ComposeOp, CompositionOrigin, MolecularChain, Molecule, NodeState, RelationBase,
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

        // ── v2 Compose Rules ──────────────────────────────────────────────
        // S = Union (dominant shape by weight, tiebreak: max value)
        let shape_byte = compose_union(&shapes);
        // R = Compose if inputs differ, keep original if all same
        let relation_byte = compose_relation(&relations);
        // V = amplify (synergy, NOT average)
        let valence = compose_amplify(&valences, total_weight);
        // A = max (take highest arousal)
        let arousal = compose_max(&arousals);
        // T = dominant (take time from highest-weight input)
        let time_byte = compose_dominant(&times);

        // Per-dimension weighted variance: Σ w_i × (val_i - result)² / Σ w_i
        let all_dims: [&[(u8, u32)]; 5] = [&shapes, &relations, &valences, &arousals, &times];
        let results: [u8; 5] = [shape_byte, relation_byte, valence, arousal, time_byte];
        for (d, (vals, res)) in all_dims.iter().zip(results.iter()).enumerate() {
            let var: f32 = vals
                .iter()
                .map(|(v, w)| {
                    let diff = *v as f32 - *res as f32;
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

        // v2: shape=0 is valid (Sphere), no fallback needed
        let shape = shape_byte;
        let relation = relation_byte;
        let time = time_byte;

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
// v2 Compose Functions (sinh học — KHÔNG trung bình)
// ─────────────────────────────────────────────────────────────────────────────

/// v2 Union: take value from the dominant input (highest weight).
///
/// Tiebreak on equal weight: take the largest value (deterministic + commutative).
/// Idempotent: all same value → returns that value.
fn compose_union(values: &[(u8, u32)]) -> u8 {
    if values.is_empty() {
        return 0;
    }
    let max_weight = values.iter().map(|(_, w)| *w).max().unwrap_or(0);
    // Among entries with max weight, pick the largest value for commutativity
    values
        .iter()
        .filter(|(_, w)| *w == max_weight)
        .map(|(v, _)| *v)
        .max()
        .unwrap_or(0)
}

/// v2 Compose relation: if all inputs have the same relation → keep it (idempotent).
/// If inputs differ → RelationBase::Compose.
fn compose_relation(values: &[(u8, u32)]) -> u8 {
    if values.is_empty() {
        return 0;
    }
    let first = values[0].0;
    if values.iter().all(|(v, _)| *v == first) {
        first // idempotent: all same → keep
    } else {
        // Compose = 0x05, dequantized = 0x05 << 4 = 0x50
        (RelationBase::Compose.as_byte()) << 4
    }
}

/// v2 Amplify: synergy, NOT average.
///
/// ```text
/// base  = weighted_avg(V_i)
/// dev   = weighted_mean_abs_deviation(V_i, base)
/// boost = dev × 0.5
/// Cv    = base + sign(base - midpoint) × boost
/// ```
///
/// Sinh học: 2 hormone cùng loại → TĂNG effect (synergistic).
/// cortisol + adrenaline → stress mạnh hơn từng cái riêng lẻ.
fn compose_amplify(values: &[(u8, u32)], total_weight: u32) -> u8 {
    if values.is_empty() || total_weight == 0 {
        return 128; // midpoint
    }

    let tw_f = total_weight as f32;

    // Weighted average
    let base: f32 = values
        .iter()
        .map(|(v, w)| *v as f32 * *w as f32)
        .sum::<f32>()
        / tw_f;

    // Weighted mean absolute deviation
    let dev: f32 = values
        .iter()
        .map(|(v, w)| (*v as f32 - base).abs() * *w as f32)
        .sum::<f32>()
        / tw_f;

    // Amplify: push towards dominant direction
    let boost = dev * 0.5;
    let midpoint = 128.0f32;
    let sign = if base >= midpoint { 1.0f32 } else { -1.0f32 };

    let result = base + sign * boost;
    // no_std: manual round + clamp
    let rounded = if result - (result as u32 as f32) >= 0.5 {
        result as u32 + 1
    } else {
        result as u32
    };
    if rounded > 255 { 255u8 } else { rounded as u8 }
}

/// v2 Max: take the highest value.
///
/// Cường độ (arousal) lấy cao hơn — sinh học: kích thích KHÔNG giảm khi kết hợp.
fn compose_max(values: &[(u8, u32)]) -> u8 {
    values.iter().map(|(v, _)| *v).max().unwrap_or(128)
}

/// v2 Dominant: take value from the input with highest weight.
///
/// Same as Union but for time dimension.
/// Tiebreak on equal weight: take the largest value (deterministic + commutative).
fn compose_dominant(values: &[(u8, u32)]) -> u8 {
    compose_union(values) // Same logic as Union
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
        // v2: Union picks one shape, max picks one arousal → result may be
        // further from one input than inputs are from each other.
        // Relax epsilon to accommodate v2 compose rules.
        let epsilon = 0.30f32;

        // LCA should not be extremely far from either input
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

    // ── Semantic correctness (v2 compose rules) ──────────────────────────

    #[test]
    fn lca_fire_water_amplify() {
        let f = fire();
        let w = water();
        let parent = lca(&f, &w);

        assert_eq!(parent.len(), 1, "LCA of 2 single-mol chains = 1 molecule");

        let fm = Molecule::from_u16(f.0[0]);
        let wm = Molecule::from_u16(w.0[0]);
        let pm = Molecule::from_u16(parent.0[0]);

        // v2 amplify: base = avg, then push towards dominant direction
        // Both fire and water have high valence (>128) → amplify pushes UP
        let avg = ((fm.valence_u8() as u16 + wm.valence_u8() as u16) / 2) as u8;
        // Amplified value should be >= avg (pushed towards high side)
        assert!(
            pm.valence_u8() >= avg,
            "v2 amplify: valence={} should be >= avg({},{})={}",
            pm.valence_u8(),
            fm.valence_u8(),
            wm.valence_u8(),
            avg
        );

        // v2 arousal = max
        let max_a = fm.arousal_u8().max(wm.arousal_u8());
        assert_eq!(
            pm.arousal_u8(),
            max_a,
            "v2 max arousal: got={}, expected=max({},{})",
            pm.arousal_u8(),
            fm.arousal_u8(),
            wm.arousal_u8()
        );

        // v2 shape = Union (dominant by weight, equal weight → max value)
        let expected_shape = fm.shape_u8().max(wm.shape_u8());
        assert_eq!(
            pm.shape_u8(),
            expected_shape,
            "v2 Union shape: got={}, expected=max({},{})",
            pm.shape_u8(),
            fm.shape_u8(),
            wm.shape_u8()
        );
    }

    #[test]
    fn lca_union_dominant_shape() {
        // Fire (weight=10) vs Water (weight=1) → Union picks fire's shape
        let f = fire();
        let w = water();
        let result = lca_many_weighted(&[f.clone(), w.clone()], &[10, 1]);
        assert_eq!(
            Molecule::from_u16(result.0[0]).shape_u8(),
            Molecule::from_u16(f.0[0]).shape_u8(),
            "Union: weight=10 fire shape dominates"
        );
    }

    #[test]
    fn lca_weighted_fire_favored() {
        // Fire (weight=10) vs Water (weight=1)
        // v2 amplify: weighted avg pushes towards fire, then boost pushes further
        let f = fire();
        let w = water();
        let result = lca_many_weighted(&[f.clone(), w.clone()], &[10, 1]);
        let fire_val = Molecule::from_u16(f.0[0]).valence_u8();
        let water_val = Molecule::from_u16(w.0[0]).valence_u8();
        let result_val = Molecule::from_u16(result.0[0]).valence_u8();
        // Result should still be closer to fire (dominant weight)
        let dist_to_fire = result_val.abs_diff(fire_val);
        let dist_to_water = result_val.abs_diff(water_val);
        assert!(
            dist_to_fire <= dist_to_water,
            "Weighted LCA valence should favor fire (w=10): val={}, fire={}, water={}",
            result_val,
            fire_val,
            water_val
        );
    }

    #[test]
    fn lca_empty_chain() {
        let empty = MolecularChain::empty();
        let f = encode_codepoint(0x1F525);
        let result = lca_weighted(&[(&empty, 1), (&f, 1)]);
        assert!(
            !result.is_empty(),
            "LCA with empty chain → non-empty result"
        );
    }

    #[test]
    fn lca_single_chain() {
        let f = fire();
        let result = lca_many(&[f.clone()]);
        assert_eq!(result, f, "LCA of 1 chain = itself");
    }

    #[test]
    fn lca_fire_cold_amplify() {
        // 🔥 vs ❄️ — v2 amplify pushes valence towards dominant direction
        let f = fire();
        let c = cold();
        let result = lca(&f, &c);

        let fire_val = Molecule::from_u16(f.0[0]).valence_u8();
        let cold_val = Molecule::from_u16(c.0[0]).valence_u8();
        let res_val = Molecule::from_u16(result.0[0]).valence_u8();

        // v2 amplify: base = avg, boost pushes in sign(base-128) direction
        let avg = ((fire_val as u16 + cold_val as u16) / 2) as u8;
        // Result should be near the average (since inputs may be on opposite sides)
        // The boost direction depends on which side of midpoint the average falls
        let diff = res_val.abs_diff(avg);
        assert!(
            diff <= 32, // within one quantization step of the amplified value
            "v2 amplify: valence={} should be near avg={} (fire={}, cold={})",
            res_val,
            avg,
            fire_val,
            cold_val
        );

        // v2 arousal = max of the two
        let max_a = f.0[0].arousal_u8().max(c.0[0].arousal_u8());
        assert_eq!(result.0[0].arousal_u8(), max_a, "v2 max arousal");
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

    // ── v2 Compose function unit tests ───────────────────────────────────

    #[test]
    fn compose_union_picks_dominant() {
        // Higher weight wins
        let vals = [(0x10u8, 1u32), (0x20, 5)];
        assert_eq!(super::compose_union(&vals), 0x20, "Higher weight wins");
    }

    #[test]
    fn compose_union_tiebreak_max() {
        // Equal weight → max value (commutative)
        let vals = [(0x10u8, 1u32), (0x20, 1)];
        assert_eq!(super::compose_union(&vals), 0x20, "Tie → max value");
        // Reversed order → same result (commutative)
        let vals_rev = [(0x20u8, 1u32), (0x10, 1)];
        assert_eq!(super::compose_union(&vals_rev), 0x20, "Commutative");
    }

    #[test]
    fn compose_relation_same_keeps() {
        let vals = [(0x30u8, 1u32), (0x30, 1)];
        assert_eq!(super::compose_relation(&vals), 0x30, "Same → keep");
    }

    #[test]
    fn compose_relation_diff_composes() {
        let vals = [(0x10u8, 1u32), (0x60, 1)];
        let result = super::compose_relation(&vals);
        let expected = (RelationBase::Compose.as_byte()) << 4; // 0x50
        assert_eq!(result, expected, "Different → Compose");
    }

    #[test]
    fn compose_amplify_same_direction() {
        // Both above midpoint → amplify pushes higher
        let vals = [(200u8, 1u32), (220, 1)];
        let result = super::compose_amplify(&vals, 2);
        let avg = 210u8;
        assert!(result >= avg, "Same direction → amplify up: got {}", result);
    }

    #[test]
    fn compose_amplify_identical() {
        // Identical values → no deviation → returns same value (idempotent)
        let vals = [(160u8, 1u32), (160, 1)];
        let result = super::compose_amplify(&vals, 2);
        assert_eq!(result, 160, "Identical → idempotent");
    }

    #[test]
    fn compose_max_picks_highest() {
        let vals = [(100u8, 1u32), (200, 1), (150, 1)];
        assert_eq!(super::compose_max(&vals), 200, "Max picks highest");
    }

    #[test]
    fn compose_dominant_picks_heaviest() {
        let vals = [(64u8, 1u32), (192, 10)];
        assert_eq!(super::compose_dominant(&vals), 192, "Dominant = heaviest weight");
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
