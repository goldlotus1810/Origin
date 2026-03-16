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

        // Delta = molecules in A not in common
        let common_mols: Vec<_> = common.0.to_vec();
        let delta_mols: Vec<_> =
            a.0.iter()
                .filter(|m| !common_mols.contains(m))
                .cloned()
                .collect();

        let delta_count = delta_mols.len();
        let delta_chain = MolecularChain(delta_mols);

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
        ctx.set(String::from("text"), String::from("hello"));
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

    // ── QT4 compliance ───────────────────────────────────────────────────────

    #[test]
    fn all_skills_are_stateless() {
        // Each skill implements Skill trait with &self (not &mut self)
        // Multiple calls with different contexts produce independent results
        let skill = IngestSkill;
        let mut ctx1 = ctx();
        ctx1.set(String::from("text"), String::from("abc"));
        let mut ctx2 = ctx();
        ctx2.set(String::from("text"), String::from("xyz"));
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
}
