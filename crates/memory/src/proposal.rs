//! # proposal — QR Proposal
//!
//! LeoAI đề xuất lên AAM → AAM approve → QR bất biến.
//! Proposal không phải QR — chỉ là đề xuất.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// ProposalKind
// ─────────────────────────────────────────────────────────────────────────────

/// Loại proposal từ Dream.
#[derive(Debug, Clone, PartialEq)]
pub enum ProposalKind {
    /// Tạo node mới từ cluster
    NewNode {
        chain:   MolecularChain,
        emotion: EmotionTag,
        /// Các observation hashes tạo nên cluster này
        sources: Vec<u64>,
    },
    /// Promote node ĐN lên QR
    PromoteQR {
        chain_hash: u64,
        fire_count: u32,
    },
    /// Tạo Silk edge mới (structural)
    NewEdge {
        from_hash: u64,
        to_hash:   u64,
        edge_kind: u8,
    },
    /// Supersede QR cũ
    SupersedeQR {
        old_hash: u64,
        new_hash: u64,
        reason:   String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// DreamProposal
// ─────────────────────────────────────────────────────────────────────────────

/// Một proposal từ Dream → AAM.
#[derive(Debug, Clone)]
pub struct DreamProposal {
    pub kind:       ProposalKind,
    pub confidence: f32,   // ∈ [0, 1]
    pub timestamp:  i64,
}

impl DreamProposal {
    pub fn new_node(
        chain:      MolecularChain,
        emotion:    EmotionTag,
        sources:    Vec<u64>,
        confidence: f32,
        ts:         i64,
    ) -> Self {
        Self {
            kind: ProposalKind::NewNode { chain, emotion, sources },
            confidence,
            timestamp: ts,
        }
    }

    pub fn promote_qr(chain_hash: u64, fire_count: u32, confidence: f32, ts: i64) -> Self {
        Self {
            kind: ProposalKind::PromoteQR { chain_hash, fire_count },
            confidence,
            timestamp: ts,
        }
    }

    pub fn new_edge(from: u64, to: u64, kind: u8, confidence: f32, ts: i64) -> Self {
        Self {
            kind: ProposalKind::NewEdge { from_hash: from, to_hash: to, edge_kind: kind },
            confidence,
            timestamp: ts,
        }
    }

    /// Proposal đủ tin cậy để gửi lên AAM không?
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.6
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AAMDecision
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// SkillProposal — từ instinct output
// ─────────────────────────────────────────────────────────────────────────────

/// Loại insight từ instinct.
#[derive(Debug, Clone, PartialEq)]
pub enum InsightKind {
    /// Phát hiện nhân quả: A → B
    Causal {
        cause_hash:  u64,
        effect_hash: u64,
    },
    /// Phát hiện mâu thuẫn giữa 2 nodes
    Contradiction {
        chain_a_hash: u64,
        chain_b_hash: u64,
        score:        f32,
    },
    /// Tạo abstraction mới từ N chains
    Abstraction {
        abstract_chain: MolecularChain,
        source_hashes:  Vec<u64>,
        variance:       f32,  // concrete/categorical/abstract
    },
    /// Analogy: tìm được D từ A:B :: C:?
    Analogy {
        result_chain: MolecularChain,
    },
    /// High curiosity — node mới đáng explore
    Curiosity {
        chain_hash: u64,
        novelty:    f32,
    },
}

/// Proposal từ instinct Skills → AAM.
///
/// Khác DreamProposal: DreamProposal từ offline consolidation,
/// SkillProposal từ real-time instinct processing.
#[derive(Debug, Clone)]
pub struct SkillProposal {
    /// Skill nào tạo proposal
    pub skill_name:  String,
    /// Loại insight
    pub kind:        InsightKind,
    /// Confidence ∈ [0, 1]
    pub confidence:  f32,
    /// Timestamp
    pub timestamp:   i64,
}

impl SkillProposal {
    /// Tạo SkillProposal.
    pub fn new(skill_name: &str, kind: InsightKind, confidence: f32, ts: i64) -> Self {
        Self {
            skill_name: String::from(skill_name),
            kind,
            confidence,
            timestamp: ts,
        }
    }

    /// Proposal đủ tin cậy không?
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.6
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AAMDecision
// ─────────────────────────────────────────────────────────────────────────────

/// Quyết định của AAM.
#[derive(Debug, Clone, PartialEq)]
pub enum AAMDecision {
    /// Approve — trở thành QR
    Approved,
    /// Reject — giữ ở ĐN
    Rejected { reason: String },
    /// Pending — cần thêm evidence
    Pending  { needed_fire_count: u32 },
}

// ─────────────────────────────────────────────────────────────────────────────
// AAM (Agent AI Master)
// ─────────────────────────────────────────────────────────────────────────────

/// AAM — stateless, chỉ approve/reject proposals.
/// Silent by default. Không giao tiếp Ln trực tiếp.
pub struct AAM;

impl AAM {
    pub fn new() -> Self { Self }

    /// Xem xét proposal từ LeoAI.
    pub fn review(&self, proposal: &DreamProposal) -> AAMDecision {
        // Reject nếu confidence quá thấp
        if proposal.confidence < 0.5 {
            return AAMDecision::Rejected {
                reason: alloc::format!("confidence={:.2} < 0.5", proposal.confidence),
            };
        }

        match &proposal.kind {
            ProposalKind::NewNode { sources, .. } => {
                // Cần ít nhất 3 sources để tạo node mới
                if sources.len() < 3 {
                    return AAMDecision::Pending { needed_fire_count: 3 };
                }
                AAMDecision::Approved
            }

            ProposalKind::PromoteQR { fire_count, .. } => {
                // Cần fire_count ≥ 5 để promote QR
                if *fire_count < 5 {
                    return AAMDecision::Pending {
                        needed_fire_count: 5 - fire_count,
                    };
                }
                AAMDecision::Approved
            }

            ProposalKind::NewEdge { .. } => {
                // Edge proposal: approve nếu confidence OK
                AAMDecision::Approved
            }

            ProposalKind::SupersedeQR { .. } => {
                // Supersede cần confidence cao hơn
                if proposal.confidence < 0.8 {
                    return AAMDecision::Rejected {
                        reason: "SupersedeQR cần confidence ≥ 0.8".into(),
                    };
                }
                AAMDecision::Approved
            }
        }
    }

    /// Batch review — trả về approved proposals.
    pub fn review_batch<'a>(&self, proposals: &'a [DreamProposal]) -> Vec<&'a DreamProposal> {
        proposals.iter()
            .filter(|p| matches!(self.review(p), AAMDecision::Approved))
            .collect()
    }

    /// Review SkillProposal từ instinct.
    pub fn review_skill(&self, proposal: &SkillProposal) -> AAMDecision {
        if proposal.confidence < 0.5 {
            return AAMDecision::Rejected {
                reason: alloc::format!("skill confidence={:.2} < 0.5", proposal.confidence),
            };
        }

        match &proposal.kind {
            InsightKind::Causal { .. } => {
                // Nhân quả: cần confidence cao
                if proposal.confidence >= 0.7 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Pending { needed_fire_count: 3 }
                }
            }

            InsightKind::Contradiction { score, .. } => {
                // Mâu thuẫn: score cao + confidence → approve
                if *score > 0.5 && proposal.confidence >= 0.6 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Pending { needed_fire_count: 2 }
                }
            }

            InsightKind::Abstraction { source_hashes, .. } => {
                // Abstraction: cần ≥ 3 sources
                if source_hashes.len() >= 3 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Pending { needed_fire_count: 3 }
                }
            }

            InsightKind::Analogy { .. } => {
                // Analogy: approve nếu confidence OK
                AAMDecision::Approved
            }

            InsightKind::Curiosity { novelty, .. } => {
                // Curiosity: novelty cao → worth exploring
                if *novelty > 0.4 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Rejected {
                        reason: alloc::format!("novelty={:.2} too low", novelty),
                    }
                }
            }
        }
    }
}

impl Default for AAM { fn default() -> Self { Self::new() } }

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::string::ToString;

    fn skip() -> bool { ucd::table_len() == 0 }

    #[test]
    fn proposal_is_confident() {
        if skip() { return; }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(chain, silk::edge::EmotionTag::NEUTRAL,
            vec![1,2,3], 0.75, 1000);
        assert!(p.is_confident());
    }

    #[test]
    fn proposal_not_confident() {
        if skip() { return; }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(chain, silk::edge::EmotionTag::NEUTRAL,
            vec![1,2,3], 0.4, 1000);
        assert!(!p.is_confident());
    }

    #[test]
    fn aam_approve_new_node() {
        if skip() { return; }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(chain, silk::edge::EmotionTag::NEUTRAL,
            vec![1,2,3,4], 0.75, 1000);
        assert_eq!(AAM::new().review(&p), AAMDecision::Approved);
    }

    #[test]
    fn aam_pending_insufficient_sources() {
        if skip() { return; }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(chain, silk::edge::EmotionTag::NEUTRAL,
            vec![1,2], 0.75, 1000); // chỉ 2 sources < 3
        assert!(matches!(AAM::new().review(&p), AAMDecision::Pending { .. }));
    }

    #[test]
    fn aam_approve_promote_qr() {
        let p = DreamProposal::promote_qr(0xABCD, 10, 0.8, 1000);
        assert_eq!(AAM::new().review(&p), AAMDecision::Approved);
    }

    #[test]
    fn aam_pending_promote_low_fire() {
        let p = DreamProposal::promote_qr(0xABCD, 3, 0.8, 1000); // fire=3 < 5
        assert!(matches!(AAM::new().review(&p),
            AAMDecision::Pending { needed_fire_count: 2 }));
    }

    #[test]
    fn aam_reject_low_confidence() {
        let p = DreamProposal::promote_qr(0xABCD, 10, 0.3, 1000);
        assert!(matches!(AAM::new().review(&p), AAMDecision::Rejected { .. }));
    }

    #[test]
    fn aam_batch_filters_approved() {
        let proposals = vec![
            DreamProposal::promote_qr(0x01, 10, 0.8, 1000), // approved
            DreamProposal::promote_qr(0x02, 3,  0.8, 1000), // pending
            DreamProposal::promote_qr(0x03, 10, 0.3, 1000), // rejected
        ];
        let aam = AAM::new();
        let approved = aam.review_batch(&proposals);
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn skill_proposal_causal_approved() {
        let p = SkillProposal::new("Causality", InsightKind::Causal {
            cause_hash: 0x01, effect_hash: 0x02,
        }, 0.8, 1000);
        assert!(p.is_confident());
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_causal_pending_low_conf() {
        let p = SkillProposal::new("Causality", InsightKind::Causal {
            cause_hash: 0x01, effect_hash: 0x02,
        }, 0.55, 1000);
        assert!(matches!(AAM::new().review_skill(&p), AAMDecision::Pending { .. }));
    }

    #[test]
    fn skill_proposal_contradiction() {
        let p = SkillProposal::new("Contradiction", InsightKind::Contradiction {
            chain_a_hash: 0x01, chain_b_hash: 0x02, score: 0.75,
        }, 0.7, 1000);
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_curiosity_rejected_low_novelty() {
        let p = SkillProposal::new("Curiosity", InsightKind::Curiosity {
            chain_hash: 0x01, novelty: 0.2,
        }, 0.8, 1000);
        assert!(matches!(AAM::new().review_skill(&p), AAMDecision::Rejected { .. }));
    }

    #[test]
    fn skill_proposal_curiosity_approved_high_novelty() {
        let p = SkillProposal::new("Curiosity", InsightKind::Curiosity {
            chain_hash: 0x01, novelty: 0.8,
        }, 0.7, 1000);
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_abstraction_needs_sources() {
        if skip() { return; }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = SkillProposal::new("Abstraction", InsightKind::Abstraction {
            abstract_chain: chain, source_hashes: vec![1, 2], variance: 0.3,
        }, 0.8, 1000);
        // Only 2 sources < 3 → pending
        assert!(matches!(AAM::new().review_skill(&p), AAMDecision::Pending { .. }));
    }

    #[test]
    fn skill_proposal_rejected_low_confidence() {
        let p = SkillProposal::new("Analogy", InsightKind::Analogy {
            result_chain: MolecularChain::empty(),
        }, 0.3, 1000);
        assert!(matches!(AAM::new().review_skill(&p), AAMDecision::Rejected { .. }));
    }

    #[test]
    fn aam_supersede_needs_high_confidence() {
        let p = DreamProposal {
            kind: ProposalKind::SupersedeQR {
                old_hash: 0x01, new_hash: 0x02,
                reason: "better data".to_string(),
            },
            confidence: 0.7, // < 0.8
            timestamp: 1000,
        };
        assert!(matches!(AAM::new().review(&p), AAMDecision::Rejected { .. }));

        let p2 = DreamProposal {
            kind: ProposalKind::SupersedeQR {
                old_hash: 0x01, new_hash: 0x02,
                reason: "better data".to_string(),
            },
            confidence: 0.85, // ≥ 0.8
            timestamp: 2000,
        };
        assert_eq!(AAM::new().review(&p2), AAMDecision::Approved);
    }
}
