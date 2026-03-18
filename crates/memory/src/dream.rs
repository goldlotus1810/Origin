//! # dream — DreamCycle
//!
//! Trigger: idle > 5 phút (hay gọi thủ công).
//!
//! Pipeline:
//!   1. Scan STM top-N observations
//!   2. Dual-threshold cluster:
//!      score = 0.3×chain_sim + 0.4×hebbian_weight + 0.3×co_activation_ratio
//!      cluster nếu score ≥ 0.6
//!   3. LCA(cluster) → chain mới
//!   4. Tạo DreamProposal → gửi lên AAM
//!   5. AAM approve → promote QR
//!   6. Cập nhật Registry + Silk

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use olang::lca::lca_many_weighted;
use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;
use silk::hebbian::fib;

use crate::proposal::{AAMDecision, DreamProposal, AAM};
use agents::learning::{Observation, ShortTermMemory};

// ─────────────────────────────────────────────────────────────────────────────
// DreamConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Config cho DreamCycle.
///
/// α, β, γ = scoring weights cho cluster_score:
///   score = α×chain_sim + β×hebbian_weight + γ×co_activation_ratio
///
/// Constraint: α + β + γ = 1.0 (normalized tại construction).
pub struct DreamConfig {
    /// Số observations tối đa để xét (top-N by fire_count)
    pub scan_top_n: usize,
    /// Score threshold để cluster (0.6 mặc định)
    pub cluster_threshold: f32,
    /// Min cluster size để tạo proposal
    pub min_cluster_size: usize,
    /// Depth của tree hiện tại (ảnh hưởng Fibonacci threshold)
    pub tree_depth: usize,
    /// α — weight cho chain similarity (default 0.3)
    pub alpha: f32,
    /// β — weight cho Hebbian weight (default 0.4)
    pub beta: f32,
    /// γ — weight cho co-activation ratio (default 0.3)
    pub gamma: f32,
}

impl DreamConfig {
    /// Preset cho conversation context: threshold thấp hơn (0.30),
    /// min cluster nhỏ hơn (2), depth nông (2).
    ///
    /// Hội thoại thông thường tạo ít observations với chain_sim thấp,
    /// cần threshold thấp hơn để Dream cluster được.
    pub fn for_conversation() -> Self {
        Self {
            scan_top_n: 32,
            cluster_threshold: 0.30,
            min_cluster_size: 2,
            tree_depth: 2,
            alpha: 0.3,
            beta: 0.4,
            gamma: 0.3,
        }
    }

    /// Tạo DreamConfig với custom α, β, γ.
    ///
    /// Tự normalize nếu tổng ≠ 1.0.
    pub fn with_weights(alpha: f32, beta: f32, gamma: f32) -> Self {
        let sum = alpha + beta + gamma;
        let (a, b, g) = if sum > 0.001 {
            (alpha / sum, beta / sum, gamma / sum)
        } else {
            (0.3, 0.4, 0.3) // fallback to default
        };
        Self {
            alpha: a,
            beta: b,
            gamma: g,
            ..Default::default()
        }
    }
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            scan_top_n: 32,
            cluster_threshold: 0.6,
            min_cluster_size: 3,
            tree_depth: 3,
            alpha: 0.3,
            beta: 0.4,
            gamma: 0.3,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DreamResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả của một Dream cycle.
#[derive(Debug)]
pub struct DreamResult {
    /// Số observations đã scan
    pub scanned: usize,
    /// Số clusters tìm được
    pub clusters_found: usize,
    /// Proposals gửi lên AAM
    pub proposals: Vec<DreamProposal>,
    /// Proposals được AAM approve
    pub approved: usize,
    /// Proposals bị reject/pending
    pub rejected: usize,
    /// chain_hash của nodes vừa đủ điều kiện Mature
    pub matured_nodes: Vec<u64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// DreamCycle
// ─────────────────────────────────────────────────────────────────────────────

/// LeoAI's Dream engine.
///
/// Chạy khi idle, scan STM, cluster, đề xuất lên AAM.
pub struct DreamCycle {
    config: DreamConfig,
    aam: AAM,
}

impl DreamCycle {
    pub fn new(config: DreamConfig) -> Self {
        Self {
            config,
            aam: AAM::new(),
        }
    }

    /// Chạy một Dream cycle.
    ///
    /// stm:   Short-term memory để scan
    /// graph: Silk graph để lấy Hebbian weights
    /// ts:    Timestamp hiện tại
    pub fn run(&self, stm: &ShortTermMemory, graph: &SilkGraph, ts: i64) -> DreamResult {
        // ── 1. Scan top-N observations ────────────────────────────────────────
        let top = stm.top_n(self.config.scan_top_n);
        let scanned = top.len();

        // ── 1a. Collect matured nodes ───────────────────────────────────────
        let fib_threshold = fib(self.config.tree_depth);
        let matured_nodes: Vec<u64> = top
            .iter()
            .filter(|obs| {
                // Đã mature (set trong STM push) hoặc đủ fire_count cho Mature
                if obs.maturity.is_mature() {
                    return true;
                }
                // Dream context: dùng Hebbian weight từ Silk graph
                let ha = obs.chain.chain_hash();
                let weight = graph.assoc_weight(ha, ha).max(
                    // Tìm max weight từ bất kỳ neighbor nào
                    graph.neighbors(ha).iter().fold(0.0f32, |max_w, &nb| {
                        max_w.max(graph.assoc_weight(ha, nb)).max(graph.assoc_weight(nb, ha))
                    })
                );
                obs.maturity.advance(obs.fire_count, weight, fib_threshold).is_mature()
            })
            .map(|obs| obs.chain.chain_hash())
            .collect();

        if scanned < self.config.min_cluster_size {
            return DreamResult {
                scanned,
                clusters_found: 0,
                proposals: Vec::new(),
                approved: 0,
                rejected: 0,
                matured_nodes,
            };
        }

        // ── 2. Cluster ────────────────────────────────────────────────────────
        let clusters = self.find_clusters(&top, graph);
        let clusters_found = clusters.len();

        // ── 3. Tạo proposals từ clusters ─────────────────────────────────────
        let mut proposals: Vec<DreamProposal> = Vec::new();

        for cluster in &clusters {
            if cluster.len() < self.config.min_cluster_size {
                continue;
            }

            // LCA(cluster) → chain mới
            let chains: Vec<MolecularChain> = cluster.iter().map(|obs| obs.chain.clone()).collect();
            let weights: Vec<u32> = cluster.iter().map(|obs| obs.fire_count).collect();

            let lca_chain = lca_many_weighted(&chains, &weights);
            if lca_chain.is_empty() {
                continue;
            }

            // Emotion tổng hợp của cluster
            let cluster_emotion = aggregate_emotion(cluster);

            // Confidence = avg fire_count / Fib[depth]
            let avg_fire =
                cluster.iter().map(|o| o.fire_count as f32).sum::<f32>() / cluster.len() as f32;
            let fib_threshold = fib(self.config.tree_depth) as f32;
            let confidence = (avg_fire / fib_threshold).min(1.0);

            // Sources = hashes của observations
            let sources: Vec<u64> = cluster.iter().map(|obs| obs.chain.chain_hash()).collect();

            proposals.push(DreamProposal::new_node(
                lca_chain,
                cluster_emotion,
                sources,
                confidence,
                ts,
            ));

            // Proposal promote QR cho observations fire nhiều
            // Gap #8: prefer mature, but Dream can also advance maturity
            for obs in cluster {
                let eligible = if obs.maturity.is_mature() {
                    true
                } else {
                    // Dream context: try advancing with Hebbian weight
                    let ha = obs.chain.chain_hash();
                    let w = graph.assoc_weight(ha, ha).max(
                        graph.neighbors(ha).iter().fold(0.0f32, |m, &nb| {
                            m.max(graph.assoc_weight(ha, nb)).max(graph.assoc_weight(nb, ha))
                        })
                    );
                    let adv = obs.maturity.advance(obs.fire_count, w, fib(self.config.tree_depth));
                    adv.is_mature() || obs.fire_count >= fib(self.config.tree_depth)
                };
                if eligible && obs.fire_count >= fib(self.config.tree_depth) {
                    // Gap #5: unified_neighbors enriches QR confidence
                    let neighbor_bonus = obs.mol_summary.as_ref().map_or(0.0, |mol| {
                        let neighbors = graph.unified_neighbors(
                            obs.chain.chain_hash(),
                            Some(mol),
                        );
                        // More high-weight neighbors → higher confidence
                        let strong = neighbors.iter().filter(|n| n.weight >= 0.5).count();
                        (strong as f32 * 0.05).min(0.3)
                    });
                    let qr_confidence = ((obs.fire_count as f32 / 10.0) + neighbor_bonus).min(1.0);
                    proposals.push(DreamProposal::promote_qr(
                        obs.chain.chain_hash(),
                        obs.fire_count,
                        qr_confidence,
                        ts,
                    ));
                }
            }
        }

        // ── 4. AAM review ─────────────────────────────────────────────────────
        let approved_count = proposals
            .iter()
            .filter(|p| matches!(self.aam.review(p), AAMDecision::Approved))
            .count();
        let rejected_count = proposals.len() - approved_count;

        DreamResult {
            scanned,
            clusters_found,
            proposals,
            approved: approved_count,
            rejected: rejected_count,
            matured_nodes,
        }
    }

    // ── Clustering ────────────────────────────────────────────────────────────

    /// Tìm clusters trong observations dùng dual-threshold.
    ///
    /// QT⑪: Cluster chỉ trong cùng layer — group by layer trước.
    /// score(A,B) = α×(chain_sim + implicit) + β×hebbian_weight + γ×co_act_ratio
    fn find_clusters<'a>(
        &self,
        observations: &[&'a Observation],
        graph: &SilkGraph,
    ) -> Vec<Vec<&'a Observation>> {
        if observations.is_empty() {
            return Vec::new();
        }

        // QT⑪: Group by layer trước khi cluster
        let mut by_layer: BTreeMap<u8, Vec<&'a Observation>> = BTreeMap::new();
        for &obs in observations {
            by_layer.entry(obs.layer).or_default().push(obs);
        }

        let mut all_clusters: Vec<Vec<&'a Observation>> = Vec::new();

        for layer_obs in by_layer.values() {
            let max_fire = layer_obs.iter().map(|o| o.fire_count).max().unwrap_or(1);
            let n = layer_obs.len();
            let mut parent: Vec<usize> = (0..n).collect();

            // Union-Find: find root iteratively
            fn find_root(parent: &mut [usize], mut x: usize) -> usize {
                while parent[x] != x {
                    let pp = parent[parent[x]];
                    parent[x] = pp;
                    x = parent[x];
                }
                x
            }

            // Compare all pairs within same layer
            for i in 0..n {
                for j in (i + 1)..n {
                    let score =
                        self.cluster_score(layer_obs[i], layer_obs[j], graph, max_fire);
                    if score >= self.config.cluster_threshold {
                        let ri = find_root(&mut parent, i);
                        let rj = find_root(&mut parent, j);
                        if ri != rj {
                            parent[ri] = rj;
                        }
                    }
                }
            }

            // Group by root
            let mut groups: BTreeMap<usize, Vec<&'a Observation>> = BTreeMap::new();
            for (i, &obs) in layer_obs.iter().enumerate() {
                let root = find_root(&mut parent, i);
                groups.entry(root).or_default().push(obs);
            }

            all_clusters.extend(groups.into_values());
        }

        all_clusters
    }

    /// Tính cluster_score(A, B).
    ///
    /// score = α×(chain_sim + implicit_bonus) + β×hebbian_weight + γ×co_act_ratio
    ///
    /// Dùng MolSummary::similarity() (5D-aware) thay vì chain byte similarity.
    /// implicit_bonus từ implicit Silk (5D shared dimensions).
    fn cluster_score(
        &self,
        a: &Observation,
        b: &Observation,
        graph: &SilkGraph,
        max_fire: u32,
    ) -> f32 {
        let ha = a.chain.chain_hash();
        let hb = b.chain.chain_hash();

        // Chain similarity — prefer MolSummary (5D-aware) over byte-level
        let chain_sim = match (&a.mol_summary, &b.mol_summary) {
            (Some(ma), Some(mb)) => ma.similarity(mb),
            _ => a.chain.similarity_full(&b.chain), // fallback
        };

        // Implicit Silk bonus (5D shared dimensions)
        let implicit_bonus = match (&a.mol_summary, &b.mol_summary) {
            (Some(ma), Some(mb)) => {
                let silk = silk::index::SilkIndex::implicit_silk(ma, mb);
                silk.strength * 0.5 // scale implicit bonus
            }
            _ => 0.0,
        };

        // Hebbian weight (bidirectional max)
        let hebbian = graph.assoc_weight(ha, hb).max(graph.assoc_weight(hb, ha));

        // Co-activation ratio
        let co_score = graph.cluster_score_partial(ha, hb, max_fire);

        self.config.alpha * (chain_sim + implicit_bonus)
            + self.config.beta * hebbian
            + self.config.gamma * co_score
    }
}

impl Default for DreamCycle {
    fn default() -> Self {
        Self::new(DreamConfig::default())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn aggregate_emotion(observations: &[&Observation]) -> EmotionTag {
    if observations.is_empty() {
        return EmotionTag::NEUTRAL;
    }
    let mut tv = 0.0f32;
    let mut ta = 0.0f32;
    let mut td = 0.0f32;
    let mut ti = 0.0f32;
    for obs in observations {
        tv += obs.emotion.valence;
        ta += obs.emotion.arousal;
        td += obs.emotion.dominance;
        ti += obs.emotion.intensity;
    }
    let n = observations.len() as f32;
    EmotionTag::new(tv / n, ta / n, td / n, ti / n)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use olang::encoder::encode_codepoint;
    use silk::edge::EmotionTag;


    fn make_stm_with_fire(entries: &[(u32, u32)]) -> ShortTermMemory {
        // entries = (codepoint, fire_count)
        // Note: codepoints with same UCD bytes → same chain_hash → deduplicated
        // Use codepoints from DIFFERENT UCD sub-ranges for distinct entries
        let mut stm = ShortTermMemory::new(512);
        for &(cp, fires) in entries {
            let chain = encode_codepoint(cp);
            for i in 0..fires {
                stm.push(chain.clone(), EmotionTag::NEUTRAL, i as i64 * 1000);
            }
        }
        stm
    }

    /// STM với chains thủ công để đảm bảo uniqueness trong test.
    fn make_stm_unique(emotions: &[(f32, f32, u32)]) -> ShortTermMemory {
        // emotions = (valence, arousal, fire_count)
        // Tạo chains từ codepoints có V/A khác nhau
        use olang::molecular::EmotionDim;
        let mut stm = ShortTermMemory::new(512);
        let cps = [
            0x1F525u32, 0x1F4A7, 0x2744, 0x1F9E0, 0x25CF, 0x2208, 0x2192, 0x26A0,
        ];
        for (i, &(v, a, fires)) in emotions.iter().enumerate() {
            let cp = cps[i % cps.len()];
            let mut chain = encode_codepoint(cp);
            // Override emotion bytes to make unique
            if let Some(mol) = chain.0.first_mut() {
                mol.emotion = EmotionDim {
                    valence: ((v + 1.0) * 127.5) as u8,
                    arousal: (a * 255.0) as u8,
                };
            }
            for j in 0..fires {
                stm.push(
                    chain.clone(),
                    EmotionTag::new(v, a, 0.5, 0.5),
                    j as i64 * 1000,
                );
            }
        }
        stm
    }

    // ── DreamCycle basic ──────────────────────────────────────────────────────

    #[test]
    fn dream_empty_stm_no_proposals() {
        let stm = ShortTermMemory::new(512);
        let graph = SilkGraph::new();
        let dream = DreamCycle::default();
        let result = dream.run(&stm, &graph, 1000);
        assert_eq!(result.proposals.len(), 0);
        assert_eq!(result.scanned, 0);
    }

    #[test]
    fn dream_too_few_nodes_no_cluster() {
        // Chỉ 2 observations < min_cluster_size=3
        let stm = make_stm_with_fire(&[(0x1F525, 3), (0x1F4A7, 2)]);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.6,
            min_cluster_size: 3,
            tree_depth: 3,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);
        // scanned=2 < min_cluster_size=3 → no proposals
        assert_eq!(result.proposals.len(), 0);
    }

    #[test]
    fn dream_similar_nodes_cluster() {
        // Nodes với chain similarity cao (same shape/relation, khác emotion nhẹ)
        let stm = make_stm_unique(&[
            (0.8, 0.9, 5),   // positive high
            (0.75, 0.85, 4), // positive high (similar)
            (0.7, 0.8, 3),   // positive high (similar)
            (0.65, 0.75, 3), // positive high (similar)
        ]);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.25,
            min_cluster_size: 3,
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);
        assert!(result.scanned >= 4, "scanned={}", result.scanned);
        assert!(
            !result.proposals.is_empty(),
            "Similar nodes phải cluster: clusters={}",
            result.clusters_found
        );
    }

    #[test]
    fn dream_proposals_have_lca_chain() {
        let stm = make_stm_with_fire(&[
            (0x1F600, 8), // 😀
            (0x1F601, 7), // 😁
            (0x1F602, 6), // 😂
            (0x1F603, 5), // 😃
        ]);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.2,
            min_cluster_size: 3,
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);

        for p in &result.proposals {
            if let crate::proposal::ProposalKind::NewNode { chain, sources, .. } = &p.kind {
                assert!(!chain.is_empty(), "Proposal phải có chain");
                assert!(!sources.is_empty(), "Proposal phải có sources");
            }
        }
    }

    #[test]
    fn dream_aam_approval_flow() {
        // Nodes fire nhiều → confidence cao → AAM approve
        let stm = make_stm_unique(&[
            (0.8, 0.9, 15),
            (0.75, 0.85, 12),
            (0.7, 0.8, 10),
            (0.65, 0.75, 8),
        ]);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.2,
            min_cluster_size: 3,
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);
        assert!(result.scanned >= 4, "scanned={}", result.scanned);
        // Fire cao → proposals với confidence cao → AAM approve nhiều
        assert!(
            !result.proposals.is_empty(),
            "Fire cao → phải có proposals: {}",
            result.proposals.len()
        );
    }

    #[test]
    fn dream_hebbian_boosts_clustering() {
        // Tạo Silk co-activation giữa 2 nodes trước
        let mut graph = SilkGraph::new();
        let chain_a = encode_codepoint(0x2744); // ❄
        let chain_b = encode_codepoint(0x1F311); // 🌑

        // Co-activate nhiều lần → Hebbian weight cao
        for _ in 0..30 {
            graph.co_activate(
                chain_a.chain_hash(),
                chain_b.chain_hash(),
                EmotionTag::new(-0.5, 0.3, 0.4, 0.5),
                0.9,
                0,
            );
        }

        let stm = make_stm_with_fire(&[
            (0x2744, 5),  // ❄
            (0x1F311, 4), // 🌑
            (0x1F4A7, 3), // 💧 (water — related to cold)
        ]);

        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.4,
            min_cluster_size: 2,
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);

        // Với Hebbian weight cao, ❄ và 🌑 phải cluster
        assert!(
            result.clusters_found >= 1,
            "Hebbian boost → cluster: {}",
            result.clusters_found
        );
    }

    #[test]
    fn dream_does_not_cluster_dissimilar() {
        // 🔥 và ❄ — opposite emotions → chain similarity thấp
        // Không có Hebbian weight
        let stm = make_stm_with_fire(&[
            (0x1F525, 5), // 🔥 fire — hot, high arousal
            (0x2744, 4),  // ❄  cold — low, calm
            (0x2200, 3),  // ∀  math — very different
        ]);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            scan_top_n: 32,
            cluster_threshold: 0.7, // high threshold
            min_cluster_size: 2,
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);
        // Với threshold cao=0.7 và không có Hebbian
        // Không có cluster đủ lớn (≥min_cluster_size=2) → proposals=0
        assert_eq!(
            result.proposals.len(),
            0,
            "Dissimilar nodes không tạo proposals với threshold cao: clusters={}",
            result.clusters_found
        );
    }

    // ── Maturity detection ────────────────────────────────────────────────────

    #[test]
    fn dream_detects_mature_nodes() {
        let mut stm = ShortTermMemory::new(512);
        let chain_a = encode_codepoint(0x1F525);
        let chain_b = encode_codepoint(0x1F4A7);
        for i in 0..10 {
            stm.push(chain_a.clone(), EmotionTag::NEUTRAL, i as i64 * 1000);
        }
        stm.push(chain_b.clone(), EmotionTag::NEUTRAL, 11000);

        // Silk co-activation nhiều lần → weight >= 0.854 (PROMOTE_WEIGHT)
        let mut graph = SilkGraph::new();
        for _ in 0..50 {
            graph.co_activate(
                chain_a.chain_hash(),
                chain_b.chain_hash(),
                EmotionTag::new(0.5, 0.5, 0.5, 0.5),
                0.9,
                0,
            );
        }

        let dream = DreamCycle::new(DreamConfig {
            tree_depth: 2,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 12000);
        assert!(!result.matured_nodes.is_empty(), "fire_count=10 + high Hebbian weight → phải có mature nodes");
    }

    #[test]
    fn dream_no_mature_nodes_when_fire_low() {
        let mut stm = ShortTermMemory::new(512);
        let chain = encode_codepoint(0x1F525);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
        let graph = SilkGraph::new();
        let dream = DreamCycle::new(DreamConfig {
            tree_depth: 5,
            ..Default::default()
        });
        let result = dream.run(&stm, &graph, 1000);
        assert!(result.matured_nodes.is_empty(), "fire_count=1 << fib(5)=8 → không mature");
    }

    #[test]
    fn dream_result_has_matured_nodes_field() {
        let result = DreamResult {
            scanned: 0,
            clusters_found: 0,
            proposals: alloc::vec![],
            approved: 0,
            rejected: 0,
            matured_nodes: alloc::vec![0xDEADu64],
        };
        assert_eq!(result.matured_nodes.len(), 1);
    }

    // ── Aggregate emotion ─────────────────────────────────────────────────────

    #[test]
    fn aggregate_emotion_avg() {
        let chain = encode_codepoint(0x1F525);
        let obs1 = Observation {
            chain: chain.clone(),
            emotion: EmotionTag::new(0.8, 0.9, 0.7, 0.8),
            timestamp: 1000,
            fire_count: 1,
            mol_summary: None,
            maturity: olang::molecular::Maturity::Formula,
            layer: 0,
        };
        let obs2 = Observation {
            chain: chain.clone(),
            emotion: EmotionTag::new(-0.2, 0.5, 0.3, 0.4),
            timestamp: 2000,
            fire_count: 1,
            mol_summary: None,
            maturity: olang::molecular::Maturity::Formula,
            layer: 0,
        };
        let result = aggregate_emotion(&[&obs1, &obs2]);
        assert!(
            (result.valence - 0.3).abs() < 0.01,
            "avg((0.8,-0.2)/2)=0.3: {}",
            result.valence
        );
    }
}
