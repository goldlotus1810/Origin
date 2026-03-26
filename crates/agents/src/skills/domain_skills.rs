//! # domain_skills — 15 Domain Skills cho HomeOS
//!
//! LeoAI Skills (knowledge processing):
//!   Nhận:    IngestSkill · ModalityFusionSkill
//!   Hiểu:    ClusterSkill · SimilaritySkill · DeltaSkill
//!   Sắp xếp: CuratorSkill · MergeSkill · PruneSkill
//!   Học:     HebbianSkill · DreamSkill
//!   Đề xuất: ProposalSkill
//!
//! Worker Skills (device/sensor):
//!   SensorSkill · ActuatorSkill · SecuritySkill · NetworkSkill
//!
//! Tất cả tuân thủ QT4:
//!   ① 1 Skill = 1 trách nhiệm
//!   ② Skill không biết Agent là gì
//!   ③ Skill không biết Skill khác tồn tại
//!   ④ Skill giao tiếp qua ExecContext.State
//!   ⑤ Skill không giữ state — state nằm trong Agent

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use olang::encoder::encode_codepoint;
use olang::lca::{lca, lca_many};
use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;

use crate::skill::{ExecContext, Skill, SkillResult};

// ═════════════════════════════════════════════════════════════════════════════
// LeoAI Skills — Knowledge Processing
// ═════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// IngestSkill — encode raw text → MolecularChain
// ─────────────────────────────────────────────────────────────────────────────

/// Encode text input → MolecularChain qua UCD lookup.
///
/// Đọc ctx.state["text"], encode → chain, ghi vào ctx.output_chains.
pub struct IngestSkill;

impl Skill for IngestSkill {
    fn name(&self) -> &str {
        "Ingest"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        let text = match ctx.get("text") {
            Some(t) => t,
            None => return SkillResult::Insufficient,
        };

        let chains: Vec<MolecularChain> = text
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
            .take(64)
            .map(|c| encode_codepoint(c as u32))
            .filter(|ch| !ch.is_empty())
            .collect();

        if chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let chain = lca_many(&chains);
        ctx.push_output(chain.clone());
        ctx.set(
            String::from("ingested_count"),
            alloc::format!("{}", chains.len()),
        );

        SkillResult::Ok {
            chain,
            emotion: ctx.current_emotion,
            note: String::from("text ingested"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SimilaritySkill — compute similarity between chains
// ─────────────────────────────────────────────────────────────────────────────

/// Tính similarity giữa 2+ chains qua LCA variance.
///
/// Input: ctx.input_chains (≥2).
/// Output: state["similarity"] = "0.XX".
pub struct SimilaritySkill;

impl Skill for SimilaritySkill {
    fn name(&self) -> &str {
        "Similarity"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let a = &ctx.input_chains[0];
        let b = &ctx.input_chains[1];
        let merged = lca(a, b);

        // Similarity = how much LCA preserves from originals
        // More molecules in LCA = more similar
        let max_len = a.0.len().max(b.0.len()).max(1);
        let sim = merged.0.len() as f32 / max_len as f32;

        ctx.set(String::from("similarity"), alloc::format!("{:.3}", sim));
        ctx.push_output(merged.clone());

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: alloc::format!("similarity={:.3}", sim),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DeltaSkill — compute difference between chains
// ─────────────────────────────────────────────────────────────────────────────

/// Tính delta (difference) giữa 2 chains.
///
/// Delta = molecules in A nhưng không có trong LCA(A,B).
/// Đây là "phần riêng" của A so với B.
pub struct DeltaSkill;

impl Skill for DeltaSkill {
    fn name(&self) -> &str {
        "Delta"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let a = &ctx.input_chains[0];
        let b = &ctx.input_chains[1];
        let common = lca(a, b);

        // Delta = u16 bits in A not in common
        let common_bits: Vec<u16> = common.0.to_vec();
        let delta_bits: Vec<u16> =
            a.0.iter()
                .filter(|b| !common_bits.contains(b))
                .cloned()
                .collect();

        let delta_count = delta_bits.len();
        let delta_chain = MolecularChain(delta_bits);

        ctx.set(
            String::from("delta_count"),
            alloc::format!("{}", delta_count),
        );
        ctx.push_output(delta_chain.clone());

        SkillResult::Ok {
            chain: delta_chain,
            emotion: ctx.current_emotion,
            note: alloc::format!("delta={}", delta_count),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClusterSkill — group similar chains
// ─────────────────────────────────────────────────────────────────────────────

/// Cluster input chains by similarity.
///
/// Greedy: so sánh LCA length giữa mỗi pair, group nếu similarity > threshold.
/// Output: state["cluster_count"] = số clusters, output_chains = LCA per cluster.
pub struct ClusterSkill;

impl Skill for ClusterSkill {
    fn name(&self) -> &str {
        "Cluster"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let threshold: f32 = ctx
            .get("cluster_threshold")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.3);

        // Simple greedy clustering: first chain = seed, merge if similar enough
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = alloc::vec![false; ctx.input_chains.len()];

        for i in 0..ctx.input_chains.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = alloc::vec![i];
            assigned[i] = true;

            for (j, is_assigned) in assigned.iter_mut().enumerate().skip(i + 1) {
                if *is_assigned {
                    continue;
                }
                let merged = lca(&ctx.input_chains[i], &ctx.input_chains[j]);
                let max_len = ctx.input_chains[i]
                    .0
                    .len()
                    .max(ctx.input_chains[j].0.len())
                    .max(1);
                let sim = merged.0.len() as f32 / max_len as f32;
                if sim >= threshold {
                    cluster.push(j);
                    *is_assigned = true;
                }
            }
            clusters.push(cluster);
        }

        // Output: LCA per cluster
        for cluster in &clusters {
            let chains: Vec<MolecularChain> = cluster
                .iter()
                .map(|&idx| ctx.input_chains[idx].clone())
                .collect();
            let representative = lca_many(&chains);
            ctx.push_output(representative);
        }

        let count = clusters.len();
        ctx.set(String::from("cluster_count"), alloc::format!("{}", count));

        // Return first cluster's representative
        let first = if !ctx.output_chains.is_empty() {
            ctx.output_chains[0].clone()
        } else {
            MolecularChain(Vec::new())
        };

        SkillResult::Ok {
            chain: first,
            emotion: ctx.current_emotion,
            note: alloc::format!("{} clusters", count),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CuratorSkill — organize/sort chains by quality
// ─────────────────────────────────────────────────────────────────────────────

/// Sort input chains by length (longer = richer knowledge).
///
/// Output: sorted chains in ctx.output_chains.
pub struct CuratorSkill;

impl Skill for CuratorSkill {
    fn name(&self) -> &str {
        "Curator"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let mut sorted = ctx.input_chains.clone();
        sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        ctx.set(
            String::from("curated_count"),
            alloc::format!("{}", sorted.len()),
        );

        let best = sorted[0].clone();
        for chain in sorted {
            ctx.push_output(chain);
        }

        SkillResult::Ok {
            chain: best,
            emotion: ctx.current_emotion,
            note: String::from("curated by richness"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MergeSkill — merge related chains via LCA
// ─────────────────────────────────────────────────────────────────────────────

/// Merge tất cả input chains → 1 chain đại diện qua LCA.
pub struct MergeSkill;

impl Skill for MergeSkill {
    fn name(&self) -> &str {
        "Merge"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let merged = lca_many(&ctx.input_chains);
        ctx.push_output(merged.clone());
        ctx.set(
            String::from("merged_from"),
            alloc::format!("{}", ctx.input_chains.len()),
        );

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: alloc::format!("merged {} chains", ctx.input_chains.len()),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PruneSkill — remove weak/short chains
// ─────────────────────────────────────────────────────────────────────────────

/// Prune chains shorter than threshold.
///
/// state["prune_min_len"] = minimum molecule count (default: 2).
pub struct PruneSkill;

impl Skill for PruneSkill {
    fn name(&self) -> &str {
        "Prune"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let min_len: usize = ctx
            .get("prune_min_len")
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        let before = ctx.input_chains.len();
        let kept: Vec<MolecularChain> = ctx
            .input_chains
            .iter()
            .filter(|c| c.0.len() >= min_len)
            .cloned()
            .collect();
        let pruned = before - kept.len();

        ctx.set(String::from("pruned_count"), alloc::format!("{}", pruned));

        let best = kept
            .first()
            .cloned()
            .unwrap_or_else(|| MolecularChain(Vec::new()));
        for chain in kept {
            ctx.push_output(chain);
        }

        SkillResult::Ok {
            chain: best,
            emotion: ctx.current_emotion,
            note: alloc::format!("pruned {}/{}", pruned, before),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HebbianSkill — apply Hebbian learning rules
// ─────────────────────────────────────────────────────────────────────────────

/// Apply Hebbian strengthen to chain pairs.
///
/// Input: ctx.input_chains (pairs to co-activate).
/// Output: state["new_weight"] = updated weight.
pub struct HebbianSkill;

impl Skill for HebbianSkill {
    fn name(&self) -> &str {
        "Hebbian"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let current_weight: f32 = ctx
            .get("current_weight")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let reward = ctx.current_emotion.intensity;
        let new_weight = silk::hebbian::hebbian_strengthen(current_weight, reward);

        ctx.set(
            String::from("new_weight"),
            alloc::format!("{:.4}", new_weight),
        );

        // Check promotion
        let fire_count: u32 = ctx
            .get("fire_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let depth: usize = ctx.get("depth").and_then(|s| s.parse().ok()).unwrap_or(0);
        let should_promote = silk::hebbian::should_promote(new_weight, fire_count, depth);
        ctx.set(
            String::from("should_promote"),
            alloc::format!("{}", should_promote),
        );

        let chain = lca(&ctx.input_chains[0], &ctx.input_chains[1]);
        ctx.push_output(chain.clone());

        SkillResult::Ok {
            chain,
            emotion: ctx.current_emotion,
            note: alloc::format!("w={:.4} promote={}", new_weight, should_promote),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DreamSkill — evaluate cluster for dream promotion
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluate whether a cluster of chains qualifies for dream promotion.
///
/// Uses DreamConfig α,β,γ scoring.
/// Input: ctx.input_chains = cluster members.
/// Output: state["dream_score"], state["dream_qualified"].
pub struct DreamSkill;

impl Skill for DreamSkill {
    fn name(&self) -> &str {
        "Dream"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        // α=0.3 (frequency), β=0.4 (connectivity), γ=0.3 (emotion)
        let alpha: f32 = ctx
            .get("dream_alpha")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.3);
        let beta: f32 = ctx
            .get("dream_beta")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.4);
        let gamma: f32 = ctx
            .get("dream_gamma")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.3);

        let frequency = ctx.input_chains.len() as f32 / 10.0; // normalize
        let connectivity: f32 = ctx
            .get("connectivity")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5);
        let emotion_intensity = ctx.current_emotion.intensity;

        let score = alpha * frequency.min(1.0) + beta * connectivity + gamma * emotion_intensity;
        let qualified = score >= 0.5;

        ctx.set(String::from("dream_score"), alloc::format!("{:.3}", score));
        ctx.set(
            String::from("dream_qualified"),
            alloc::format!("{}", qualified),
        );

        let merged = lca_many(&ctx.input_chains);
        ctx.push_output(merged.clone());

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: alloc::format!("dream score={:.3} q={}", score, qualified),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ProposalSkill — generate proposal for AAM
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a QR promotion proposal from qualified dream cluster.
///
/// Input: ctx.input_chains (cluster), state["dream_qualified"]="true".
/// Output: state["proposal_hash"] = chain_hash of proposed QR.
pub struct ProposalSkill;

impl Skill for ProposalSkill {
    fn name(&self) -> &str {
        "Proposal"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let qualified = ctx
            .get("dream_qualified")
            .map(|s| s == "true")
            .unwrap_or(false);

        if !qualified {
            return SkillResult::Insufficient;
        }

        let merged = lca_many(&ctx.input_chains);
        let hash = merged.chain_hash();

        ctx.set(String::from("proposal_hash"), alloc::format!("{}", hash));
        ctx.set(String::from("proposal_layer"), String::from("1"));
        ctx.push_output(merged.clone());

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: alloc::format!("proposal hash={:#018x}", hash),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// InverseRenderSkill — SDF fitting from visual features
// ─────────────────────────────────────────────────────────────────────────────

/// Fit SDF primitives from visual feature description.
///
/// Input: state["edge_ratio"], state["circularity"], state["aspect_ratio"]
/// Output: state["sdf_type"], state["sdf_confidence"], chain = geometric SDF
///
/// Camera Worker profile: L0 + FFR + vSDF + InverseRenderSkill
pub struct InverseRenderSkill;

impl Skill for InverseRenderSkill {
    fn name(&self) -> &str {
        "InverseRender"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        // Extract visual features from state
        let edge_ratio: f32 = ctx
            .get("edge_ratio")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5); // H/V edge ratio: 1.0 = all horizontal
        let circularity: f32 = ctx
            .get("circularity")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5); // 1.0 = perfectly circular
        let aspect_ratio: f32 = ctx
            .get("aspect_ratio")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0); // W/H: 1.0 = square

        // SDF fitting heuristic:
        //   High circularity (>0.8) → Sphere
        //   Low circularity + aspect ~1.0 → Box
        //   Low circularity + aspect >> 1.0 → Cylinder (elongated)
        //   High edge_ratio (>0.8) → Plane (dominant horizontal)
        //   Otherwise → Mixed
        let (sdf_type, confidence) = if circularity > 0.8 {
            (0u8, circularity) // Sphere
        } else if edge_ratio > 0.8 && circularity < 0.3 {
            (3u8, edge_ratio) // Plane
        } else if circularity < 0.4 && (0.8..=1.2).contains(&aspect_ratio) {
            (1u8, 1.0 - circularity) // Box
        } else if !(0.67..=1.5).contains(&aspect_ratio) {
            (2u8, (aspect_ratio - 1.0).abs().min(1.0)) // Cylinder
        } else {
            (4u8, 0.5) // Mixed
        };

        ctx.set(String::from("sdf_type"), alloc::format!("{}", sdf_type));
        ctx.set(
            String::from("sdf_confidence"),
            alloc::format!("{:.3}", confidence),
        );

        // Encode SDF → geometric codepoint
        let cp = match sdf_type {
            0 => 0x25CF, // ● Sphere
            1 => 0x25A0, // ■ Box
            2 => 0x25AD, // ▭ Cylinder
            3 => 0x25B3, // △ Plane
            _ => 0x25C6, // ◆ Mixed
        };
        let chain = encode_codepoint(cp);
        ctx.push_output(chain.clone());

        SkillResult::Ok {
            chain,
            emotion: ctx.current_emotion,
            note: alloc::format!("sdf={} conf={:.2}", sdf_type, confidence),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GeneralizationSkill — extract rules from clusters
// ─────────────────────────────────────────────────────────────────────────────

/// Extract IF-THEN rules from cluster patterns.
///
/// After ClusterSkill groups chains, GeneralizationSkill examines each cluster
/// to find shared dimensions (the "IF" condition) and divergent dimensions
/// (the "THEN" consequence).
///
/// Input: ctx.input_chains = cluster members (from ClusterSkill output)
/// Output: state["rule_count"], state["rule_0"], state["rule_1"], ...
///         Each rule: "IF shape=X AND relation=Y THEN valence=Z"
pub struct GeneralizationSkill;

impl Skill for GeneralizationSkill {
    fn name(&self) -> &str {
        "Generalization"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let mut rules: Vec<String> = Vec::new();

        // Analyze cluster: find shared vs divergent dimensions
        // Each MolecularChain has molecules with [shape, relation, valence, arousal, time]
        let n = ctx.input_chains.len();

        // Compute per-dimension variance across cluster members
        let mut shape_counts: [u32; 8] = [0; 8];
        let mut rel_counts: [u32; 8] = [0; 8];
        let mut val_sum: f32 = 0.0;
        let mut aro_sum: f32 = 0.0;
        let mut total_mols: u32 = 0;

        for chain in &ctx.input_chains {
            for &bits in &chain.0 {
                let mol = olang::molecular::Molecule::from_u16(bits);
                let s = (mol.shape_u8() & 0x07) as usize;
                let r = (mol.relation_u8() & 0x07) as usize;
                shape_counts[s] += 1;
                rel_counts[r] += 1;
                val_sum += mol.valence_u8() as f32 / 255.0;
                aro_sum += mol.arousal_u8() as f32 / 255.0;
                total_mols += 1;
            }
        }

        if total_mols == 0 {
            return SkillResult::Insufficient;
        }

        // Find dominant shape (shared condition)
        let dom_shape = shape_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let shape_ratio = shape_counts[dom_shape] as f32 / total_mols as f32;

        // Find dominant relation
        let dom_rel = rel_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let rel_ratio = rel_counts[dom_rel] as f32 / total_mols as f32;

        // Average emotion
        let avg_val = val_sum / total_mols as f32;
        let avg_aro = aro_sum / total_mols as f32;

        // Rule generation: only if dimension is sufficiently consistent (>60%)
        if shape_ratio > 0.6 {
            let rule = alloc::format!(
                "IF shape={} ({}%) THEN valence={:.2} arousal={:.2}",
                dom_shape,
                (shape_ratio * 100.0) as u32,
                avg_val,
                avg_aro
            );
            rules.push(rule);
        }
        if rel_ratio > 0.6 {
            let rule = alloc::format!(
                "IF relation={} ({}%) THEN valence={:.2} arousal={:.2}",
                dom_rel,
                (rel_ratio * 100.0) as u32,
                avg_val,
                avg_aro
            );
            rules.push(rule);
        }

        // Cross-dimensional rule: if both shape AND relation are consistent
        if shape_ratio > 0.5 && rel_ratio > 0.5 {
            let rule = alloc::format!(
                "IF shape={} AND relation={} THEN valence={:.2} (n={})",
                dom_shape, dom_rel, avg_val, n
            );
            rules.push(rule);
        }

        // Compute merged chain before mutable borrow of ctx
        let merged = lca_many(&ctx.input_chains);

        // Store rules in context
        ctx.set(
            String::from("rule_count"),
            alloc::format!("{}", rules.len()),
        );
        for (i, rule) in rules.iter().enumerate() {
            ctx.set(alloc::format!("rule_{}", i), rule.clone());
        }
        ctx.push_output(merged.clone());

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: alloc::format!("{} rules from {} chains", rules.len(), n),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TemporalPatternSkill — time-based pattern mining
// ─────────────────────────────────────────────────────────────────────────────

/// Mine temporal patterns from timestamped observations.
///
/// Detects:
///   1. Periodicity: events recurring at regular intervals
///   2. Time-of-day clustering: events grouping at similar hours
///   3. Sequence patterns: A always followed by B
///
/// Input: ctx.input_chains + state["timestamps"] = comma-separated i64 values
/// Output: state["temporal_period"], state["temporal_cluster_hour"],
///         state["temporal_sequence"]
pub struct TemporalPatternSkill;

impl Skill for TemporalPatternSkill {
    fn name(&self) -> &str {
        "TemporalPattern"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 3 {
            return SkillResult::Insufficient;
        }

        // Parse timestamps
        let ts_str = match ctx.get("timestamps") {
            Some(s) => s,
            None => return SkillResult::Insufficient,
        };
        let timestamps: Vec<i64> = ts_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if timestamps.len() < 3 {
            return SkillResult::Insufficient;
        }

        // 1. Periodicity detection: compute inter-event intervals
        let mut intervals: Vec<i64> = Vec::new();
        for i in 1..timestamps.len() {
            let dt = (timestamps[i] - timestamps[i - 1]).abs();
            if dt > 0 {
                intervals.push(dt);
            }
        }

        if !intervals.is_empty() {
            // Find median interval (robust to outliers)
            let mut sorted = intervals.clone();
            sorted.sort();
            let median = sorted[sorted.len() / 2];

            // Check if intervals are consistent (within 30% of median)
            let consistent = intervals
                .iter()
                .filter(|&&dt| {
                    let ratio = dt as f64 / median as f64;
                    (0.7..=1.3).contains(&ratio)
                })
                .count();

            let periodicity = consistent as f32 / intervals.len() as f32;
            if periodicity > 0.6 {
                ctx.set(
                    String::from("temporal_period"),
                    alloc::format!("{}", median),
                );
                ctx.set(
                    String::from("temporal_periodicity"),
                    alloc::format!("{:.2}", periodicity),
                );
            }
        }

        // 2. Time-of-day clustering (using modular arithmetic on 24h)
        // Convert timestamps to hour-of-day (assuming milliseconds)
        let hours: Vec<u32> = timestamps
            .iter()
            .map(|&ts| ((ts / 3_600_000) % 24) as u32)
            .collect();

        // Find most common hour bucket (3-hour windows)
        let mut buckets = [0u32; 8]; // 0-2, 3-5, 6-8, ..., 21-23
        for &h in &hours {
            let bucket = (h / 3) as usize;
            if bucket < 8 {
                buckets[bucket] += 1;
            }
        }
        let (peak_bucket, peak_count) = buckets
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .unwrap_or((0, &0));

        if *peak_count as f32 / hours.len() as f32 > 0.4 {
            let peak_hour = peak_bucket * 3;
            ctx.set(
                String::from("temporal_cluster_hour"),
                alloc::format!("{}", peak_hour),
            );
        }

        // 3. Sequence pattern: check if chain[i] → chain[i+1] repeats
        let n = ctx.input_chains.len().min(timestamps.len());
        if n >= 4 {
            let mut seq_count = 0u32;
            let h0 = ctx.input_chains[0].chain_hash();
            let h1 = ctx.input_chains[1].chain_hash();
            for i in 2..n - 1 {
                if ctx.input_chains[i].chain_hash() == h0
                    && ctx.input_chains[i + 1].chain_hash() == h1
                {
                    seq_count += 1;
                }
            }
            if seq_count > 0 {
                ctx.set(
                    String::from("temporal_sequence"),
                    alloc::format!("{}→{} x{}", h0, h1, seq_count + 1),
                );
            }
        }

        let merged = lca_many(&ctx.input_chains);
        ctx.push_output(merged.clone());

        SkillResult::Ok {
            chain: merged,
            emotion: ctx.current_emotion,
            note: String::from("temporal analysis"),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Worker Skills — Device/Sensor Processing
// ═════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// SensorSkill — sensor reading → molecular chain
// ─────────────────────────────────────────────────────────────────────────────

/// Encode sensor reading → MolecularChain.
///
/// state["sensor_value"] = raw value string.
/// state["sensor_unit"] = unit byte as string.
pub struct SensorSkill;

impl Skill for SensorSkill {
    fn name(&self) -> &str {
        "Sensor"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        let value: f32 = match ctx.get("sensor_value").and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => return SkillResult::Insufficient,
        };

        // Map sensor value → unicode codepoint range
        // Temperature: 0x2600..0x2604 (weather symbols)
        // Default: 0x25CF (●)
        let unit: u8 = ctx
            .get("sensor_unit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0xFF);

        let cp = match unit {
            0x01 => {
                // Temperature → thermometer ↔ snowflake
                if value > 30.0 {
                    0x2600
                }
                // ☀
                else if value < 10.0 {
                    0x2744
                }
                // ❄
                else {
                    0x25CF
                } // ●
            }
            0x04 => 0x27A1, // Motion → ➡
            0x05 => 0x266B, // Sound → ♫
            _ => 0x25CF,    // Default → ●
        };

        let chain = encode_codepoint(cp);
        ctx.push_output(chain.clone());

        // Emotion from sensor context
        let emotion = if !(5.0..=35.0).contains(&value) {
            EmotionTag {
                valence: -0.30,
                arousal: 0.65,
                dominance: 0.30,
                intensity: 0.55,
            }
        } else {
            EmotionTag::NEUTRAL
        };

        SkillResult::Ok {
            chain,
            emotion,
            note: alloc::format!("sensor u={:#04x} v={:.1}", unit, value),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ActuatorSkill — execute actuator commands
// ─────────────────────────────────────────────────────────────────────────────

/// Process actuator command → execute + encode result.
///
/// state["actuator_cmd"] = command byte.
/// state["actuator_value"] = value byte.
pub struct ActuatorSkill;

impl Skill for ActuatorSkill {
    fn name(&self) -> &str {
        "Actuator"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        let cmd: u8 = match ctx.get("actuator_cmd").and_then(|s| s.parse().ok()) {
            Some(c) => c,
            None => return SkillResult::Insufficient,
        };
        let value: u8 = ctx
            .get("actuator_value")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Execute: encode command result as chain
        // ON/OFF → ● / ○
        let cp = if value > 0 { 0x25CF } else { 0x25CB }; // ● or ○
        let chain = encode_codepoint(cp);
        ctx.push_output(chain.clone());

        ctx.set(String::from("actuator_executed"), String::from("true"));
        ctx.set(
            String::from("actuator_status"),
            alloc::format!("cmd={:#04x} val={:#04x}", cmd, value),
        );

        SkillResult::Ok {
            chain,
            emotion: EmotionTag::NEUTRAL,
            note: alloc::format!("actuator cmd={:#04x}", cmd),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SecuritySkill — security check for door workers
// ─────────────────────────────────────────────────────────────────────────────

/// Security gate check — verify authorization before actuating.
///
/// state["security_locked"] = "true"/"false".
/// state["auth_level"] = required auth level (0-3).
pub struct SecuritySkill;

impl Skill for SecuritySkill {
    fn name(&self) -> &str {
        "Security"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        let locked = ctx
            .get("security_locked")
            .map(|s| s == "true")
            .unwrap_or(false);

        let auth_level: u8 = ctx
            .get("auth_level")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if locked && auth_level < 2 {
            ctx.set(String::from("security_result"), String::from("denied"));
            // High arousal — security concern
            let chain = encode_codepoint(0x26A0); // ⚠
            ctx.push_output(chain.clone());
            return SkillResult::Ok {
                chain,
                emotion: EmotionTag {
                    valence: -0.50,
                    arousal: 0.80,
                    dominance: 0.70,
                    intensity: 0.65,
                },
                note: String::from("access denied"),
            };
        }

        ctx.set(String::from("security_result"), String::from("allowed"));
        let chain = encode_codepoint(0x2714); // ✔
        ctx.push_output(chain.clone());

        SkillResult::Ok {
            chain,
            emotion: EmotionTag::NEUTRAL,
            note: String::from("access allowed"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NetworkSkill — network monitoring + anomaly detection
// ─────────────────────────────────────────────────────────────────────────────

/// Monitor network traffic, detect anomalies.
///
/// state["anomaly_score"] = 0.0..1.0
/// state["bytes_in"] = traffic volume
pub struct NetworkSkill;

impl Skill for NetworkSkill {
    fn name(&self) -> &str {
        "Network"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        let anomaly: f32 = match ctx.get("anomaly_score").and_then(|s| s.parse().ok()) {
            Some(a) => a,
            None => return SkillResult::Insufficient,
        };

        let alert_level = if anomaly > 0.7 {
            "critical"
        } else if anomaly > 0.4 {
            "elevated"
        } else {
            "normal"
        };

        ctx.set(String::from("alert_level"), String::from(alert_level));

        let cp = match alert_level {
            "critical" => 0x26D4, // ⛔
            "elevated" => 0x26A0, // ⚠
            _ => 0x2714,          // ✔
        };
        let chain = encode_codepoint(cp);
        ctx.push_output(chain.clone());

        let emotion = match alert_level {
            "critical" => EmotionTag {
                valence: -0.70,
                arousal: 0.95,
                dominance: 0.20,
                intensity: 0.90,
            },
            "elevated" => EmotionTag {
                valence: -0.30,
                arousal: 0.65,
                dominance: 0.35,
                intensity: 0.55,
            },
            _ => EmotionTag::NEUTRAL,
        };

        SkillResult::Ok {
            chain,
            emotion,
            note: alloc::format!("network {}", alert_level),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImmunitySkill — threat response + countermeasures (complement to NetworkSkill)
// ─────────────────────────────────────────────────────────────────────────────

/// Respond to detected threats — quarantine, block, escalate.
///
/// NetworkSkill detects anomalies → ImmunitySkill responds to them.
/// Biological analogy: immune system response after pathogen detection.
///
/// state["alert_level"] = "normal" | "elevated" | "critical" (from NetworkSkill)
/// state["threat_type"] = "port_scan" | "brute_force" | "data_exfil" | "malware" | "unknown"
/// state["repeat_count"] = how many times this threat has been seen
///
/// Output: state["immunity_action"] = "monitor" | "block" | "quarantine" | "escalate"
pub struct ImmunitySkill;

impl Skill for ImmunitySkill {
    fn name(&self) -> &str {
        "Immunity"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        // Require alert_level from NetworkSkill
        let alert_level = match ctx.get("alert_level") {
            Some(level) => String::from(level),
            None => return SkillResult::Insufficient,
        };

        let threat_type = String::from(ctx.get("threat_type").unwrap_or("unknown"));

        let repeat_count: u32 = ctx
            .get("repeat_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        // Determine response action based on severity + repetition
        let action = match (alert_level.as_str(), repeat_count) {
            ("critical", _) => "quarantine",
            ("elevated", n) if n >= 3 => "quarantine",
            ("elevated", _) => "block",
            (_, n) if n >= 5 => "block",
            _ => "monitor",
        };

        // Escalate to Chief if quarantine needed
        let escalate = action == "quarantine";
        if escalate {
            ctx.set(String::from("immunity_escalate"), String::from("true"));
        }

        ctx.set(String::from("immunity_action"), String::from(action));
        ctx.set(
            String::from("immunity_summary"),
            alloc::format!("{} → {} (×{})", threat_type, action, repeat_count),
        );

        // Encode response as molecular chain
        let cp = match action {
            "quarantine" => 0x1F6AB, // 🚫
            "block" => 0x26D4,       // ⛔
            "monitor" => 0x1F50D,    // 🔍
            _ => 0x2714,             // ✔
        };
        let chain = encode_codepoint(cp);
        ctx.push_output(chain.clone());

        let emotion = match action {
            "quarantine" => EmotionTag {
                valence: -0.80,
                arousal: 0.90,
                dominance: 0.85,
                intensity: 0.95,
            },
            "block" => EmotionTag {
                valence: -0.50,
                arousal: 0.75,
                dominance: 0.70,
                intensity: 0.70,
            },
            _ => EmotionTag {
                valence: -0.10,
                arousal: 0.40,
                dominance: 0.50,
                intensity: 0.30,
            },
        };

        SkillResult::Ok {
            chain,
            emotion,
            note: alloc::format!("immunity {} {}", action, threat_type),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ExecContext {
        ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0)
    }

    fn sample_chain() -> MolecularChain {
        encode_codepoint(0x25CF) // ●
    }

    fn sample_chain_b() -> MolecularChain {
        encode_codepoint(0x25A0) // ■
    }

    // ── IngestSkill ──────────────────────────────────────────────────────────

    #[test]
    fn ingest_encodes_text() {
        let skill = IngestSkill;
        let mut ctx = ctx();
        // Use UDC codepoints (emoji/symbols) — Latin chars are not in UDC_TABLE
        ctx.set(String::from("text"), String::from("\u{2190}\u{25CF}\u{2200}"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(!ctx.output_chains.is_empty());
        assert!(ctx.get("ingested_count").is_some());
    }

    #[test]
    fn ingest_insufficient_without_text() {
        let skill = IngestSkill;
        let mut ctx = ctx();
        assert!(matches!(skill.execute(&mut ctx), SkillResult::Insufficient));
    }

    // ── SimilaritySkill ──────────────────────────────────────────────────────

    #[test]
    fn similarity_computes() {
        let skill = SimilaritySkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain()); // same → high similarity
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let sim: f32 = ctx.get("similarity").unwrap().parse().unwrap();
        assert!(sim > 0.5, "Same chain → high similarity: {}", sim);
    }

    #[test]
    fn similarity_insufficient_one_chain() {
        let skill = SimilaritySkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        assert!(matches!(skill.execute(&mut ctx), SkillResult::Insufficient));
    }

    // ── DeltaSkill ───────────────────────────────────────────────────────────

    #[test]
    fn delta_finds_difference() {
        let skill = DeltaSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain_b());
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(ctx.get("delta_count").is_some());
    }

    // ── ClusterSkill ─────────────────────────────────────────────────────────

    #[test]
    fn cluster_groups_similar() {
        let skill = ClusterSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain()); // same
        ctx.push_input(sample_chain_b()); // different
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let count: usize = ctx.get("cluster_count").unwrap().parse().unwrap();
        assert!(count >= 1, "At least 1 cluster: {}", count);
    }

    // ── CuratorSkill ─────────────────────────────────────────────────────────

    #[test]
    fn curator_sorts_by_richness() {
        let skill = CuratorSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain_b());
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.output_chains.len(), 2);
    }

    // ── MergeSkill ───────────────────────────────────────────────────────────

    #[test]
    fn merge_combines_chains() {
        let skill = MergeSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain_b());
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("merged_from"), Some("2"));
    }

    // ── PruneSkill ───────────────────────────────────────────────────────────

    #[test]
    fn prune_removes_short_chains() {
        let skill = PruneSkill;
        let mut ctx = ctx();
        ctx.push_input(MolecularChain(Vec::new())); // empty → pruned
        ctx.push_input(sample_chain()); // has molecules → kept
        ctx.set(String::from("prune_min_len"), String::from("1"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let pruned: usize = ctx.get("pruned_count").unwrap().parse().unwrap();
        assert_eq!(pruned, 1, "Empty chain should be pruned");
    }

    // ── HebbianSkill ─────────────────────────────────────────────────────────

    #[test]
    fn hebbian_strengthens() {
        let skill = HebbianSkill;
        let mut ctx = ExecContext::new(
            1000,
            EmotionTag {
                valence: 0.0,
                arousal: 0.5,
                dominance: 0.5,
                intensity: 0.8,
            },
            0.0,
        );
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain_b());
        ctx.set(String::from("current_weight"), String::from("0.3"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let w: f32 = ctx.get("new_weight").unwrap().parse().unwrap();
        assert!(w > 0.3, "Weight should increase: {}", w);
    }

    // ── DreamSkill ───────────────────────────────────────────────────────────

    #[test]
    fn dream_scores_cluster() {
        let skill = DreamSkill;
        let mut ctx = ExecContext::new(
            1000,
            EmotionTag {
                valence: 0.0,
                arousal: 0.5,
                dominance: 0.5,
                intensity: 0.7,
            },
            0.0,
        );
        // 5 chains = frequency 0.5 normalized
        for _ in 0..5 {
            ctx.push_input(sample_chain());
        }
        ctx.set(String::from("connectivity"), String::from("0.8"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let score: f32 = ctx.get("dream_score").unwrap().parse().unwrap();
        assert!(score > 0.0, "Score should be positive: {}", score);
    }

    // ── ProposalSkill ────────────────────────────────────────────────────────

    #[test]
    fn proposal_requires_qualification() {
        let skill = ProposalSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        // Not qualified
        let r = skill.execute(&mut ctx);
        assert!(matches!(r, SkillResult::Insufficient));
    }

    #[test]
    fn proposal_generates_when_qualified() {
        let skill = ProposalSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.set(String::from("dream_qualified"), String::from("true"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(ctx.get("proposal_hash").is_some());
    }

    // ── SensorSkill ──────────────────────────────────────────────────────────

    #[test]
    fn sensor_encodes_reading() {
        let skill = SensorSkill;
        let mut ctx = ctx();
        ctx.set(String::from("sensor_value"), String::from("25.0"));
        ctx.set(String::from("sensor_unit"), String::from("1")); // Temperature
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(!ctx.output_chains.is_empty());
    }

    #[test]
    fn sensor_insufficient_without_value() {
        let skill = SensorSkill;
        let mut ctx = ctx();
        assert!(matches!(skill.execute(&mut ctx), SkillResult::Insufficient));
    }

    // ── ActuatorSkill ────────────────────────────────────────────────────────

    #[test]
    fn actuator_executes() {
        let skill = ActuatorSkill;
        let mut ctx = ctx();
        ctx.set(String::from("actuator_cmd"), String::from("1"));
        ctx.set(String::from("actuator_value"), String::from("255"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("actuator_executed"), Some("true"));
    }

    // ── SecuritySkill ────────────────────────────────────────────────────────

    #[test]
    fn security_denies_when_locked() {
        let skill = SecuritySkill;
        let mut ctx = ctx();
        ctx.set(String::from("security_locked"), String::from("true"));
        ctx.set(String::from("auth_level"), String::from("0"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("security_result"), Some("denied"));
    }

    #[test]
    fn security_allows_when_unlocked() {
        let skill = SecuritySkill;
        let mut ctx = ctx();
        ctx.set(String::from("security_locked"), String::from("false"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("security_result"), Some("allowed"));
    }

    #[test]
    fn security_allows_high_auth_even_locked() {
        let skill = SecuritySkill;
        let mut ctx = ctx();
        ctx.set(String::from("security_locked"), String::from("true"));
        ctx.set(String::from("auth_level"), String::from("3")); // high auth
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("security_result"), Some("allowed"));
    }

    // ── NetworkSkill ─────────────────────────────────────────────────────────

    #[test]
    fn network_critical_on_high_anomaly() {
        let skill = NetworkSkill;
        let mut ctx = ctx();
        ctx.set(String::from("anomaly_score"), String::from("0.9"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("alert_level"), Some("critical"));
    }

    #[test]
    fn network_normal_on_low_anomaly() {
        let skill = NetworkSkill;
        let mut ctx = ctx();
        ctx.set(String::from("anomaly_score"), String::from("0.1"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("alert_level"), Some("normal"));
    }

    // ── ImmunitySkill ─────────────────────────────────────────────────────────

    #[test]
    fn immunity_quarantine_on_critical() {
        let skill = ImmunitySkill;
        let mut ctx = ctx();
        ctx.set(String::from("alert_level"), String::from("critical"));
        ctx.set(String::from("threat_type"), String::from("malware"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("immunity_action"), Some("quarantine"));
        assert_eq!(ctx.get("immunity_escalate"), Some("true"));
    }

    #[test]
    fn immunity_block_on_elevated() {
        let skill = ImmunitySkill;
        let mut ctx = ctx();
        ctx.set(String::from("alert_level"), String::from("elevated"));
        ctx.set(String::from("threat_type"), String::from("port_scan"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("immunity_action"), Some("block"));
    }

    #[test]
    fn immunity_monitor_on_normal() {
        let skill = ImmunitySkill;
        let mut ctx = ctx();
        ctx.set(String::from("alert_level"), String::from("normal"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("immunity_action"), Some("monitor"));
    }

    #[test]
    fn immunity_escalate_on_repeated_elevated() {
        let skill = ImmunitySkill;
        let mut ctx = ctx();
        ctx.set(String::from("alert_level"), String::from("elevated"));
        ctx.set(String::from("repeat_count"), String::from("5"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("immunity_action"), Some("quarantine"));
        assert_eq!(ctx.get("immunity_escalate"), Some("true"));
    }

    #[test]
    fn immunity_insufficient_without_alert() {
        let skill = ImmunitySkill;
        let mut ctx = ctx();
        let r = skill.execute(&mut ctx);
        assert!(matches!(r, SkillResult::Insufficient));
    }

    // ── QT4 compliance ───────────────────────────────────────────────────────

    #[test]
    fn all_skills_are_stateless() {
        // Each skill implements Skill trait with &self (not &mut self)
        // Multiple calls with different contexts produce independent results
        let skill = IngestSkill;
        let mut ctx1 = ctx();
        ctx1.set(String::from("text"), String::from("\u{2190}\u{25CF}"));
        let mut ctx2 = ctx();
        ctx2.set(String::from("text"), String::from("\u{2200}\u{2208}"));
        let r1 = skill.execute(&mut ctx1);
        let r2 = skill.execute(&mut ctx2);
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        // Results are independent — no cross-contamination
    }

    #[test]
    fn skills_communicate_only_via_context() {
        // DreamSkill reads state, ProposalSkill reads DreamSkill's output
        // But they don't know about each other — only through ExecContext state
        let dream = DreamSkill;
        let proposal = ProposalSkill;
        let mut ctx = ExecContext::new(
            1000,
            EmotionTag {
                valence: 0.0,
                arousal: 0.5,
                dominance: 0.5,
                intensity: 0.8,
            },
            0.0,
        );
        for _ in 0..5 {
            ctx.push_input(sample_chain());
        }
        ctx.set(String::from("connectivity"), String::from("0.9"));

        // Dream evaluates
        let _ = dream.execute(&mut ctx);
        // Check if qualified
        let qualified = ctx
            .get("dream_qualified")
            .map(|s| s == "true")
            .unwrap_or(false);

        if qualified {
            // Reset inputs for proposal
            let chain = ctx.output_chains[0].clone();
            ctx.input_chains.clear();
            ctx.output_chains.clear();
            ctx.push_input(chain);
            let r = proposal.execute(&mut ctx);
            assert!(r.is_ok());
            assert!(ctx.get("proposal_hash").is_some());
        }
    }

    // ── InverseRenderSkill ─────────────────────────────────────────────────

    #[test]
    fn inverse_render_high_circularity_sphere() {
        let skill = InverseRenderSkill;
        let mut ctx = ctx();
        ctx.set(String::from("edge_ratio"), String::from("0.3"));
        ctx.set(String::from("circularity"), String::from("0.9"));
        ctx.set(String::from("aspect_ratio"), String::from("1.0"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("sdf_type"), Some("0")); // 0 = Sphere
        let conf: f32 = ctx.get("sdf_confidence").unwrap().parse().unwrap();
        assert!(conf > 0.5, "High circularity → confident sphere: {}", conf);
    }

    #[test]
    fn inverse_render_high_edge_ratio_plane() {
        let skill = InverseRenderSkill;
        let mut ctx = ctx();
        ctx.set(String::from("edge_ratio"), String::from("0.85"));
        ctx.set(String::from("circularity"), String::from("0.2"));
        ctx.set(String::from("aspect_ratio"), String::from("1.0"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("sdf_type"), Some("3")); // 3 = Plane
    }

    #[test]
    fn inverse_render_extreme_aspect_cylinder() {
        let skill = InverseRenderSkill;
        let mut ctx = ctx();
        ctx.set(String::from("edge_ratio"), String::from("0.4"));
        ctx.set(String::from("circularity"), String::from("0.5"));
        ctx.set(String::from("aspect_ratio"), String::from("3.5"));
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert_eq!(ctx.get("sdf_type"), Some("2")); // 2 = Cylinder
    }

    #[test]
    fn inverse_render_defaults_to_mixed() {
        let skill = InverseRenderSkill;
        let mut ctx = ctx();
        // No params → defaults: edge_ratio=0.5, circularity=0.5, aspect=1.0
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        // With defaults, should produce some SDF type
        assert!(ctx.get("sdf_type").is_some());
        assert!(ctx.get("sdf_confidence").is_some());
    }

    // ── GeneralizationSkill ────────────────────────────────────────────────

    #[test]
    fn generalization_extracts_rules() {
        let skill = GeneralizationSkill;
        let mut ctx = ctx();
        // Push multiple identical chains → high shape consistency
        for _ in 0..5 {
            ctx.push_input(sample_chain()); // all ● (same shape)
        }
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        let count: usize = ctx.get("rule_count").unwrap().parse().unwrap();
        assert!(count > 0, "Should extract at least 1 rule from uniform cluster");
    }

    #[test]
    fn generalization_insufficient_one_chain() {
        let skill = GeneralizationSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        let r = skill.execute(&mut ctx);
        assert!(matches!(r, SkillResult::Insufficient));
    }

    #[test]
    fn generalization_mixed_cluster() {
        let skill = GeneralizationSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.push_input(sample_chain_b());
        ctx.push_input(encode_codepoint(0x25B2)); // ▲
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        // Mixed cluster may or may not produce rules depending on shape distribution
    }

    // ── TemporalPatternSkill ───────────────────────────────────────────────

    #[test]
    fn temporal_pattern_detects_periodicity() {
        let skill = TemporalPatternSkill;
        let mut ctx = ctx();
        for _ in 0..6 {
            ctx.push_input(sample_chain());
        }
        // Regular intervals: every 3600 units
        ctx.set(
            String::from("timestamps"),
            String::from("1000,4600,8200,11800,15400,19000"),
        );
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(
            ctx.get("temporal_period").is_some(),
            "Should detect temporal_period"
        );
    }

    #[test]
    fn temporal_pattern_insufficient_few_entries() {
        let skill = TemporalPatternSkill;
        let mut ctx = ctx();
        ctx.push_input(sample_chain());
        ctx.set(String::from("timestamps"), String::from("1000"));
        let r = skill.execute(&mut ctx);
        assert!(matches!(r, SkillResult::Insufficient));
    }

    #[test]
    fn temporal_pattern_time_of_day_buckets() {
        let skill = TemporalPatternSkill;
        let mut ctx = ctx();
        for _ in 0..5 {
            ctx.push_input(sample_chain());
        }
        // All at ~10:00 AM (hour 10) on different days (ms)
        // 10h = 36_000_000 ms, 24h = 86_400_000 ms
        ctx.set(
            String::from("timestamps"),
            String::from("36000000,122400000,208800000,295200000,381600000"),
        );
        let r = skill.execute(&mut ctx);
        assert!(r.is_ok());
        assert!(
            ctx.get("temporal_cluster_hour").is_some(),
            "Should detect cluster hour"
        );
    }
}
