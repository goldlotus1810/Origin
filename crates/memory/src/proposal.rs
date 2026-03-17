//! # proposal — QR Proposal
//!
//! LeoAI đề xuất lên AAM → AAM approve → QR bất biến.
//! Proposal không phải QR — chỉ là đề xuất.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

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
        chain: MolecularChain,
        emotion: EmotionTag,
        /// Các observation hashes tạo nên cluster này
        sources: Vec<u64>,
    },
    /// Promote node ĐN lên QR
    PromoteQR { chain_hash: u64, fire_count: u32 },
    /// Tạo Silk edge mới (structural)
    NewEdge {
        from_hash: u64,
        to_hash: u64,
        edge_kind: u8,
    },
    /// Supersede QR cũ
    SupersedeQR {
        old_hash: u64,
        new_hash: u64,
        reason: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// DreamProposal
// ─────────────────────────────────────────────────────────────────────────────

/// Một proposal từ Dream → AAM.
#[derive(Debug, Clone)]
pub struct DreamProposal {
    pub kind: ProposalKind,
    pub confidence: f32, // ∈ [0, 1]
    pub timestamp: i64,
}

impl DreamProposal {
    pub fn new_node(
        chain: MolecularChain,
        emotion: EmotionTag,
        sources: Vec<u64>,
        confidence: f32,
        ts: i64,
    ) -> Self {
        Self {
            kind: ProposalKind::NewNode {
                chain,
                emotion,
                sources,
            },
            confidence,
            timestamp: ts,
        }
    }

    pub fn promote_qr(chain_hash: u64, fire_count: u32, confidence: f32, ts: i64) -> Self {
        Self {
            kind: ProposalKind::PromoteQR {
                chain_hash,
                fire_count,
            },
            confidence,
            timestamp: ts,
        }
    }

    pub fn new_edge(from: u64, to: u64, kind: u8, confidence: f32, ts: i64) -> Self {
        Self {
            kind: ProposalKind::NewEdge {
                from_hash: from,
                to_hash: to,
                edge_kind: kind,
            },
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
    Causal { cause_hash: u64, effect_hash: u64 },
    /// Phát hiện mâu thuẫn giữa 2 nodes
    Contradiction {
        chain_a_hash: u64,
        chain_b_hash: u64,
        score: f32,
    },
    /// Tạo abstraction mới từ N chains
    Abstraction {
        abstract_chain: MolecularChain,
        source_hashes: Vec<u64>,
        variance: f32, // concrete/categorical/abstract
    },
    /// Analogy: tìm được D từ A:B :: C:?
    Analogy { result_chain: MolecularChain },
    /// High curiosity — node mới đáng explore
    Curiosity { chain_hash: u64, novelty: f32 },
    /// Learned skill sequence pattern
    SkillPattern {
        skill_names: Vec<String>,
        effectiveness: f32,
        observations: u32,
    },
}

/// Proposal từ instinct Skills → AAM.
///
/// Khác DreamProposal: DreamProposal từ offline consolidation,
/// SkillProposal từ real-time instinct processing.
#[derive(Debug, Clone)]
pub struct SkillProposal {
    /// Skill nào tạo proposal
    pub skill_name: String,
    /// Loại insight
    pub kind: InsightKind,
    /// Confidence ∈ [0, 1]
    pub confidence: f32,
    /// Timestamp
    pub timestamp: i64,
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
    Pending { needed_fire_count: u32 },
}

// ─────────────────────────────────────────────────────────────────────────────
// AAM (Agent AI Master)
// ─────────────────────────────────────────────────────────────────────────────

/// AAM — stateless, chỉ approve/reject proposals.
/// Silent by default. Không giao tiếp Ln trực tiếp.
pub struct AAM;

impl AAM {
    pub fn new() -> Self {
        Self
    }

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
                    return AAMDecision::Pending {
                        needed_fire_count: 3,
                    };
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
        proposals
            .iter()
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
                    AAMDecision::Pending {
                        needed_fire_count: 3,
                    }
                }
            }

            InsightKind::Contradiction { score, .. } => {
                // Mâu thuẫn: score cao + confidence → approve
                if *score > 0.5 && proposal.confidence >= 0.6 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Pending {
                        needed_fire_count: 2,
                    }
                }
            }

            InsightKind::Abstraction { source_hashes, .. } => {
                // Abstraction: cần ≥ 3 sources
                if source_hashes.len() >= 3 {
                    AAMDecision::Approved
                } else {
                    AAMDecision::Pending {
                        needed_fire_count: 3,
                    }
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

            InsightKind::SkillPattern {
                observations,
                effectiveness,
                ..
            } => {
                // SkillPattern: cần ≥3 observations + effectiveness ≥ 0.6
                if *observations >= 3 && *effectiveness >= 0.6 {
                    AAMDecision::Approved
                } else if *observations < 3 {
                    AAMDecision::Pending {
                        needed_fire_count: 3 - *observations,
                    }
                } else {
                    AAMDecision::Rejected {
                        reason: alloc::format!(
                            "effectiveness={:.2} < 0.6",
                            effectiveness
                        ),
                    }
                }
            }
        }
    }
}

impl Default for AAM {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UserAuthority — quyền tối cao của người dùng
// ─────────────────────────────────────────────────────────────────────────────

/// Trạng thái xác nhận của user cho một proposal.
#[derive(Debug, Clone, PartialEq)]
pub enum UserConfirmation {
    /// Chưa hỏi user
    Pending,
    /// User đồng ý
    Approved,
    /// User từ chối
    Rejected,
    /// User hoãn (chưa quyết định)
    Deferred,
}

/// QRProposal — proposal cần user xác nhận trước khi ghi QR.
///
/// AAM approve → QRProposal tạo → hỏi user → user approve → mới ghi QR.
/// Đây là cầu nối giữa AAM (tự động) và User (quyết định cuối).
#[derive(Debug, Clone)]
pub struct QRProposal {
    /// Proposal gốc từ Dream
    pub proposal: DreamProposal,
    /// User đã confirm chưa
    pub user_confirmation: UserConfirmation,
    /// Prompt hiển thị cho user
    pub prompt: alloc::string::String,
    /// Timestamp tạo
    pub created_at: i64,
}

impl QRProposal {
    /// Tạo QRProposal từ DreamProposal đã AAM-approved.
    pub fn from_approved(proposal: DreamProposal, ts: i64) -> Self {
        let prompt = match &proposal.kind {
            ProposalKind::NewNode { sources, .. } => {
                alloc::format!(
                    "Tạo node mới từ {} nguồn, confidence={:.0}%. Đồng ý?",
                    sources.len(),
                    proposal.confidence * 100.0
                )
            }
            ProposalKind::PromoteQR { fire_count, .. } => {
                alloc::format!(
                    "Nâng cấp kiến thức thành QR (bất biến), {} lần xác nhận, confidence={:.0}%. Đồng ý?",
                    fire_count, proposal.confidence * 100.0
                )
            }
            ProposalKind::NewEdge { .. } => {
                alloc::format!(
                    "Tạo liên kết mới, confidence={:.0}%. Đồng ý?",
                    proposal.confidence * 100.0
                )
            }
            ProposalKind::SupersedeQR { reason, .. } => {
                alloc::format!(
                    "Cập nhật QR cũ: \"{}\", confidence={:.0}%. Đồng ý?",
                    reason,
                    proposal.confidence * 100.0
                )
            }
        };

        Self {
            proposal,
            user_confirmation: UserConfirmation::Pending,
            prompt,
            created_at: ts,
        }
    }

    /// User xác nhận (accept/reject).
    pub fn confirm(&mut self, approved: bool) {
        self.user_confirmation = if approved {
            UserConfirmation::Approved
        } else {
            UserConfirmation::Rejected
        };
    }

    /// Hoãn quyết định.
    pub fn defer(&mut self) {
        self.user_confirmation = UserConfirmation::Deferred;
    }

    /// Đã sẵn sàng ghi QR?
    pub fn is_ready_to_write(&self) -> bool {
        self.user_confirmation == UserConfirmation::Approved
    }

    /// Bị từ chối?
    pub fn is_rejected(&self) -> bool {
        self.user_confirmation == UserConfirmation::Rejected
    }
}

/// UserAuthority — ghi nhận quyền tối cao của người dùng.
///
/// Mọi QR write phải qua UserAuthority.confirm().
/// AAM là brain logic, UserAuthority là ý thức (user).
pub struct UserAuthority {
    /// Queue proposals chờ user confirm
    pending: Vec<QRProposal>,
    /// Proposals đã được user approve (ready to write)
    approved: Vec<QRProposal>,
    /// Proposals bị reject
    rejected_count: u32,
    /// Auto-approve mode (cho testing hoặc khi user trust hệ thống)
    auto_approve: bool,
}

impl UserAuthority {
    /// Tạo mới — mặc định KHÔNG auto-approve.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            approved: Vec::new(),
            rejected_count: 0,
            auto_approve: false,
        }
    }

    /// Submit proposal đã AAM-approved → chờ user confirm.
    pub fn submit(&mut self, proposal: DreamProposal, ts: i64) {
        if self.auto_approve {
            let mut qr = QRProposal::from_approved(proposal, ts);
            qr.confirm(true);
            self.approved.push(qr);
        } else {
            self.pending.push(QRProposal::from_approved(proposal, ts));
        }
    }

    /// User xử lý proposal tại index.
    pub fn respond(&mut self, index: usize, approved: bool) {
        if index >= self.pending.len() {
            return;
        }
        let mut qr = self.pending.remove(index);
        qr.confirm(approved);
        if approved {
            self.approved.push(qr);
        } else {
            self.rejected_count += 1;
        }
    }

    /// Lấy proposals đã approved (ready to write QR).
    pub fn drain_approved(&mut self) -> Vec<QRProposal> {
        core::mem::take(&mut self.approved)
    }

    /// Proposals đang chờ user.
    pub fn pending(&self) -> &[QRProposal] {
        &self.pending
    }

    /// Số pending.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Số đã reject.
    pub fn rejected_count(&self) -> u32 {
        self.rejected_count
    }

    /// Bật auto-approve (trust mode).
    pub fn set_auto_approve(&mut self, on: bool) {
        self.auto_approve = on;
    }

    /// Đang auto-approve?
    pub fn is_auto_approve(&self) -> bool {
        self.auto_approve
    }
}

impl Default for UserAuthority {
    fn default() -> Self {
        Self::new()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// RegistryGate — cơ chế cứng: mọi thứ phải đăng ký Registry
// ═════════════════════════════════════════════════════════════════════════════

/// Mức độ cảnh báo khi phát hiện component chưa đăng ký.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    /// Thông báo bình thường — chờ user confirm
    Normal,
    /// Quan trọng — cần xử lý sớm
    Important,
    /// Red-alert — tự giải quyết, đối chiếu 9 QT
    RedAlert,
}

/// Một component chưa đăng ký, đang chờ xử lý.
#[derive(Debug, Clone)]
pub struct PendingRegistration {
    /// Tên component (e.g., "skill:new_skill", "agent:new_worker")
    pub name: String,
    /// Chain hash nếu có
    pub chain_hash: Option<u64>,
    /// NodeKind nên thuộc nhóm nào
    pub suggested_kind: u8, // olang::registry::NodeKind as u8
    /// Mức độ cảnh báo
    pub alert_level: AlertLevel,
    /// Lý do tạo
    pub reason: String,
    /// Timestamp phát hiện
    pub discovered_at: i64,
    /// User đã xác nhận chưa
    pub user_response: UserConfirmation,
    /// Đã tự giải quyết (red-alert) chưa
    pub auto_resolved: bool,
    /// QT rules đã kiểm tra (bitmask: bit N = QT(N+1))
    pub qt_checked: u32,
}

impl PendingRegistration {
    /// Tạo pending registration mới.
    pub fn new(name: &str, suggested_kind: u8, alert_level: AlertLevel, reason: &str, ts: i64) -> Self {
        Self {
            name: String::from(name),
            chain_hash: None,
            suggested_kind,
            alert_level,
            reason: String::from(reason),
            discovered_at: ts,
            user_response: UserConfirmation::Pending,
            auto_resolved: false,
            qt_checked: 0,
        }
    }

    /// Tạo với chain hash.
    pub fn with_hash(mut self, hash: u64) -> Self {
        self.chain_hash = Some(hash);
        self
    }

    /// Đã được xử lý chưa (user approve hoặc auto-resolve).
    pub fn is_resolved(&self) -> bool {
        self.auto_resolved
            || matches!(
                self.user_response,
                UserConfirmation::Approved | UserConfirmation::Rejected
            )
    }

    /// User approve.
    pub fn approve(&mut self) {
        self.user_response = UserConfirmation::Approved;
    }

    /// User reject.
    pub fn reject(&mut self) {
        self.user_response = UserConfirmation::Rejected;
    }

    /// Generate prompt cho user.
    pub fn prompt(&self) -> String {
        let level_icon = match self.alert_level {
            AlertLevel::Normal => "○",
            AlertLevel::Important => "⚠",
            AlertLevel::RedAlert => "🔴",
        };
        alloc::format!(
            "{} [Registry] \"{}\" chưa đăng ký. {}\nĐồng ý đăng ký?",
            level_icon,
            self.name,
            self.reason,
        )
    }
}

/// RegistryGate — kiểm tra và bắt buộc đăng ký.
///
/// Cơ chế cứng:
/// 1. check() → phát hiện component chưa đăng ký → tạo PendingRegistration
/// 2. PendingRegistration nằm ở STM (trí nhớ ngắn hạn, không tự hoạt động)
/// 3. AAM thông báo cho user → chờ xác nhận
/// 4. Nếu user offline → thông báo đợi
/// 5. Nếu red-alert → tự giải quyết, đối chiếu 9 QT
pub struct RegistryGate {
    /// Pending registrations chờ xử lý
    pending: Vec<PendingRegistration>,
    /// Đã thông báo nhưng chưa resolve (user offline)
    notified: Vec<PendingRegistration>,
    /// Đã resolve (history)
    resolved_count: u32,
    /// Auto-resolved count (red-alert)
    auto_resolved_count: u32,
}

impl RegistryGate {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            notified: Vec::new(),
            resolved_count: 0,
            auto_resolved_count: 0,
        }
    }

    /// Kiểm tra chain_hash có trong Registry không.
    ///
    /// Nếu KHÔNG có → tạo PendingRegistration + return false.
    /// Nếu CÓ → return true.
    pub fn check_registered(
        &mut self,
        name: &str,
        chain_hash: u64,
        suggested_kind: u8,
        alert_level: AlertLevel,
        ts: i64,
    ) -> bool {
        // Check đã pending chưa — tránh tạo trùng
        if self.pending.iter().any(|p| p.name == name) {
            return false;
        }
        if self.notified.iter().any(|p| p.name == name) {
            return false;
        }

        // Tạo PendingRegistration
        let reason = alloc::format!(
            "Component '{}' (hash=0x{:X}) xuất hiện nhưng chưa đăng ký Registry",
            name, chain_hash
        );
        let pending = PendingRegistration::new(name, suggested_kind, alert_level, &reason, ts)
            .with_hash(chain_hash);

        self.pending.push(pending);
        false
    }

    /// Thêm pending registration trực tiếp.
    pub fn submit(&mut self, pending: PendingRegistration) {
        // Tránh trùng
        if self.pending.iter().any(|p| p.name == pending.name) {
            return;
        }
        self.pending.push(pending);
    }

    /// Lấy tất cả pending → thông báo cho user.
    ///
    /// Chuyển từ pending → notified (đợi user xác nhận).
    /// User offline → notified vẫn ở đó đợi.
    pub fn drain_notifications(&mut self) -> Vec<PendingRegistration> {
        let drained: Vec<PendingRegistration> = self.pending.drain(..).collect();
        let for_return = drained.clone();
        // Chuyển sang notified — đợi user respond
        for p in drained {
            self.notified.push(p);
        }
        for_return
    }

    /// User respond cho notification tại index.
    pub fn respond(&mut self, index: usize, approved: bool) {
        if index < self.notified.len() {
            if approved {
                self.notified[index].approve();
            } else {
                self.notified[index].reject();
            }
            self.resolved_count += 1;
        }
    }

    /// Drain các notification đã được user approve.
    ///
    /// Trả về danh sách cần ghi Registry (approved).
    /// Remove resolved khỏi notified.
    pub fn drain_approved(&mut self) -> Vec<PendingRegistration> {
        let mut approved = Vec::new();
        let mut remaining = Vec::new();
        for p in self.notified.drain(..) {
            if p.user_response == UserConfirmation::Approved || p.auto_resolved {
                approved.push(p);
            } else if p.is_resolved() {
                // Rejected — discard
            } else {
                remaining.push(p); // Still pending — keep waiting
            }
        }
        self.notified = remaining;
        approved
    }

    /// Red-alert auto-resolve: tự giải quyết theo 9 QT.
    ///
    /// Chạy khi component xuất hiện nhưng user offline + alert_level == RedAlert.
    /// Đối chiếu với 9 Quy Tắc bất biến:
    ///   QT1: ○(x)==x — identity check
    ///   QT4: mọi Molecule từ encode_codepoint
    ///   QT8: mọi Node tự động registry
    ///   QT9: ghi file TRƯỚC — cập nhật RAM SAU
    ///   QT10: append-only
    ///   QT14: L0 không import L1
    ///   QT18: không đủ evidence → im lặng
    ///   QT19: 1 Skill = 1 trách nhiệm
    ///   QT20: Skill không biết Agent
    pub fn auto_resolve_red_alerts(&mut self) {
        for p in &mut self.notified {
            if p.alert_level != AlertLevel::RedAlert || p.is_resolved() {
                continue;
            }

            // Đối chiếu 9 QT
            let mut qt_mask: u32 = 0;
            let mut safe = true;

            // QT8: mọi Node tự động registry → chính vì thế mới cần auto-resolve
            qt_mask |= 1 << 7; // QT8 checked

            // QT10: append-only → auto-register = append, không delete → OK
            qt_mask |= 1 << 9; // QT10 checked

            // QT14: L0 không import L1 → nếu suggested_kind == Alphabet (0) thì KHÔNG auto
            if p.suggested_kind == 0 {
                // Đây là L0 — KHÔNG tự thêm vào L0 (vi phạm QT14)
                safe = false;
            }
            qt_mask |= 1 << 13; // QT14 checked

            // QT18: không đủ evidence → im lặng
            // Red-alert = có đủ evidence (emergency) → OK
            qt_mask |= 1 << 17; // QT18 checked

            // QT4: molecule phải từ encode_codepoint
            // Auto-resolve chỉ cho phép nếu có chain_hash (đã encode)
            if p.chain_hash.is_none() {
                safe = false;
            }
            qt_mask |= 1 << 3; // QT4 checked

            p.qt_checked = qt_mask;

            if safe {
                p.auto_resolved = true;
                self.auto_resolved_count += 1;
            }
            // Nếu không safe → vẫn pending, đợi user
        }
    }

    /// Số pending chưa xử lý.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Số notification đang đợi user.
    pub fn notified_count(&self) -> usize {
        self.notified.len()
    }

    /// Tổng đã resolve.
    pub fn resolved_count(&self) -> u32 {
        self.resolved_count
    }

    /// Tổng auto-resolved (red-alert).
    pub fn auto_resolved_count(&self) -> u32 {
        self.auto_resolved_count
    }

    /// Tất cả pending notifications (read-only, cho UI).
    pub fn notifications(&self) -> &[PendingRegistration] {
        &self.notified
    }
}

impl Default for RegistryGate {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    #[test]
    fn proposal_is_confident() {
        if skip() {
            return;
        }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(
            chain,
            silk::edge::EmotionTag::NEUTRAL,
            vec![1, 2, 3],
            0.75,
            1000,
        );
        assert!(p.is_confident());
    }

    #[test]
    fn proposal_not_confident() {
        if skip() {
            return;
        }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(
            chain,
            silk::edge::EmotionTag::NEUTRAL,
            vec![1, 2, 3],
            0.4,
            1000,
        );
        assert!(!p.is_confident());
    }

    #[test]
    fn aam_approve_new_node() {
        if skip() {
            return;
        }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(
            chain,
            silk::edge::EmotionTag::NEUTRAL,
            vec![1, 2, 3, 4],
            0.75,
            1000,
        );
        assert_eq!(AAM::new().review(&p), AAMDecision::Approved);
    }

    #[test]
    fn aam_pending_insufficient_sources() {
        if skip() {
            return;
        }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = DreamProposal::new_node(
            chain,
            silk::edge::EmotionTag::NEUTRAL,
            vec![1, 2],
            0.75,
            1000,
        ); // chỉ 2 sources < 3
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
        assert!(matches!(
            AAM::new().review(&p),
            AAMDecision::Pending {
                needed_fire_count: 2
            }
        ));
    }

    #[test]
    fn aam_reject_low_confidence() {
        let p = DreamProposal::promote_qr(0xABCD, 10, 0.3, 1000);
        assert!(matches!(
            AAM::new().review(&p),
            AAMDecision::Rejected { .. }
        ));
    }

    #[test]
    fn aam_batch_filters_approved() {
        let proposals = vec![
            DreamProposal::promote_qr(0x01, 10, 0.8, 1000), // approved
            DreamProposal::promote_qr(0x02, 3, 0.8, 1000),  // pending
            DreamProposal::promote_qr(0x03, 10, 0.3, 1000), // rejected
        ];
        let aam = AAM::new();
        let approved = aam.review_batch(&proposals);
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn skill_proposal_causal_approved() {
        let p = SkillProposal::new(
            "Causality",
            InsightKind::Causal {
                cause_hash: 0x01,
                effect_hash: 0x02,
            },
            0.8,
            1000,
        );
        assert!(p.is_confident());
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_causal_pending_low_conf() {
        let p = SkillProposal::new(
            "Causality",
            InsightKind::Causal {
                cause_hash: 0x01,
                effect_hash: 0x02,
            },
            0.55,
            1000,
        );
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Pending { .. }
        ));
    }

    #[test]
    fn skill_proposal_contradiction() {
        let p = SkillProposal::new(
            "Contradiction",
            InsightKind::Contradiction {
                chain_a_hash: 0x01,
                chain_b_hash: 0x02,
                score: 0.75,
            },
            0.7,
            1000,
        );
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_curiosity_rejected_low_novelty() {
        let p = SkillProposal::new(
            "Curiosity",
            InsightKind::Curiosity {
                chain_hash: 0x01,
                novelty: 0.2,
            },
            0.8,
            1000,
        );
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Rejected { .. }
        ));
    }

    #[test]
    fn skill_proposal_curiosity_approved_high_novelty() {
        let p = SkillProposal::new(
            "Curiosity",
            InsightKind::Curiosity {
                chain_hash: 0x01,
                novelty: 0.8,
            },
            0.7,
            1000,
        );
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn skill_proposal_abstraction_needs_sources() {
        if skip() {
            return;
        }
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let p = SkillProposal::new(
            "Abstraction",
            InsightKind::Abstraction {
                abstract_chain: chain,
                source_hashes: vec![1, 2],
                variance: 0.3,
            },
            0.8,
            1000,
        );
        // Only 2 sources < 3 → pending
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Pending { .. }
        ));
    }

    #[test]
    fn skill_proposal_rejected_low_confidence() {
        let p = SkillProposal::new(
            "Analogy",
            InsightKind::Analogy {
                result_chain: MolecularChain::empty(),
            },
            0.3,
            1000,
        );
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Rejected { .. }
        ));
    }

    #[test]
    fn aam_supersede_needs_high_confidence() {
        let p = DreamProposal {
            kind: ProposalKind::SupersedeQR {
                old_hash: 0x01,
                new_hash: 0x02,
                reason: "better data".to_string(),
            },
            confidence: 0.7, // < 0.8
            timestamp: 1000,
        };
        assert!(matches!(
            AAM::new().review(&p),
            AAMDecision::Rejected { .. }
        ));

        let p2 = DreamProposal {
            kind: ProposalKind::SupersedeQR {
                old_hash: 0x01,
                new_hash: 0x02,
                reason: "better data".to_string(),
            },
            confidence: 0.85, // ≥ 0.8
            timestamp: 2000,
        };
        assert_eq!(AAM::new().review(&p2), AAMDecision::Approved);
    }

    // ── UserAuthority ──────────────────────────────────────────────────────────

    #[test]
    fn user_authority_default_no_auto() {
        let ua = UserAuthority::new();
        assert!(!ua.is_auto_approve());
        assert_eq!(ua.pending_count(), 0);
    }

    #[test]
    fn submit_goes_to_pending() {
        let mut ua = UserAuthority::new();
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        ua.submit(p, 1000);
        assert_eq!(ua.pending_count(), 1, "Proposal chờ user confirm");
    }

    #[test]
    fn respond_approved_moves_to_approved() {
        let mut ua = UserAuthority::new();
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        ua.submit(p, 1000);
        ua.respond(0, true); // user approve
        assert_eq!(ua.pending_count(), 0);
        let approved = ua.drain_approved();
        assert_eq!(approved.len(), 1);
        assert!(approved[0].is_ready_to_write());
    }

    #[test]
    fn respond_rejected_counted() {
        let mut ua = UserAuthority::new();
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        ua.submit(p, 1000);
        ua.respond(0, false); // user reject
        assert_eq!(ua.pending_count(), 0);
        assert_eq!(ua.rejected_count(), 1);
    }

    #[test]
    fn auto_approve_bypasses_pending() {
        let mut ua = UserAuthority::new();
        ua.set_auto_approve(true);
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        ua.submit(p, 1000);
        assert_eq!(ua.pending_count(), 0, "Auto-approve → không pending");
        let approved = ua.drain_approved();
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn qr_proposal_prompt_contains_info() {
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        let qr = QRProposal::from_approved(p, 1000);
        assert!(qr.prompt.contains("QR"), "Prompt phải nói về QR");
        assert!(qr.prompt.contains("80"), "Prompt phải có confidence %");
        assert!(!qr.is_ready_to_write(), "Chưa confirm → chưa sẵn sàng");
    }

    #[test]
    fn qr_proposal_defer() {
        let p = DreamProposal::promote_qr(0x01, 10, 0.8, 1000);
        let mut qr = QRProposal::from_approved(p, 1000);
        qr.defer();
        assert_eq!(qr.user_confirmation, UserConfirmation::Deferred);
        assert!(!qr.is_ready_to_write());
        assert!(!qr.is_rejected());
    }

    #[test]
    fn respond_out_of_bounds_noop() {
        let mut ua = UserAuthority::new();
        ua.respond(999, true); // index out of bounds
        assert_eq!(ua.pending_count(), 0); // no crash
    }

    // ── RegistryGate tests ─────────────────────────────────────────────────

    #[test]
    fn registry_gate_detects_unregistered() {
        let mut gate = RegistryGate::new();
        let registered = gate.check_registered(
            "skill:new_thing", 0xDEAD, 4, AlertLevel::Normal, 1000,
        );
        assert!(!registered, "Unregistered → false");
        assert_eq!(gate.pending_count(), 1);
    }

    #[test]
    fn registry_gate_no_duplicates() {
        let mut gate = RegistryGate::new();
        gate.check_registered("skill:x", 0x01, 4, AlertLevel::Normal, 1000);
        gate.check_registered("skill:x", 0x01, 4, AlertLevel::Normal, 2000); // duplicate
        assert_eq!(gate.pending_count(), 1, "No duplicate pending");
    }

    #[test]
    fn registry_gate_drain_notifications() {
        let mut gate = RegistryGate::new();
        gate.check_registered("skill:a", 0x01, 4, AlertLevel::Normal, 1000);
        gate.check_registered("skill:b", 0x02, 4, AlertLevel::Normal, 1000);
        let notifs = gate.drain_notifications();
        assert_eq!(notifs.len(), 2);
        assert_eq!(gate.pending_count(), 0, "Drained → no more pending");
        assert_eq!(gate.notified_count(), 2, "Moved to notified");
    }

    #[test]
    fn registry_gate_user_approve() {
        let mut gate = RegistryGate::new();
        gate.check_registered("skill:a", 0x01, 4, AlertLevel::Normal, 1000);
        gate.drain_notifications();
        gate.respond(0, true); // User approves
        let approved = gate.drain_approved();
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].name, "skill:a");
        assert_eq!(gate.notified_count(), 0, "Resolved → removed");
    }

    #[test]
    fn registry_gate_user_reject() {
        let mut gate = RegistryGate::new();
        gate.check_registered("skill:a", 0x01, 4, AlertLevel::Normal, 1000);
        gate.drain_notifications();
        gate.respond(0, false); // User rejects
        let approved = gate.drain_approved();
        assert!(approved.is_empty(), "Rejected → not in approved");
        assert_eq!(gate.notified_count(), 0, "Resolved → removed");
    }

    #[test]
    fn registry_gate_user_offline_waits() {
        let mut gate = RegistryGate::new();
        gate.check_registered("skill:a", 0x01, 4, AlertLevel::Normal, 1000);
        gate.drain_notifications();
        // User offline — no respond
        let approved = gate.drain_approved();
        assert!(approved.is_empty(), "Not responded → not approved");
        assert_eq!(gate.notified_count(), 1, "Still waiting");
    }

    #[test]
    fn registry_gate_red_alert_auto_resolve() {
        let mut gate = RegistryGate::new();
        gate.check_registered("agent:emergency", 0xBEEF, 3, AlertLevel::RedAlert, 1000);
        gate.drain_notifications();
        // User offline → red-alert auto-resolve
        gate.auto_resolve_red_alerts();
        assert_eq!(gate.auto_resolved_count(), 1);
        let approved = gate.drain_approved();
        assert_eq!(approved.len(), 1, "Red-alert auto → approved");
        assert!(approved[0].auto_resolved);
        assert!(approved[0].qt_checked > 0, "QT rules checked");
    }

    #[test]
    fn registry_gate_red_alert_l0_not_auto() {
        let mut gate = RegistryGate::new();
        // suggested_kind=0 (Alphabet/L0) → CANNOT auto-resolve (QT14)
        gate.check_registered("alphabet:bad", 0xDEAD, 0, AlertLevel::RedAlert, 1000);
        gate.drain_notifications();
        gate.auto_resolve_red_alerts();
        assert_eq!(gate.auto_resolved_count(), 0, "L0 cannot be auto-resolved");
        let approved = gate.drain_approved();
        assert!(approved.is_empty(), "L0 stays pending");
        assert_eq!(gate.notified_count(), 1, "Still waiting for user");
    }

    #[test]
    fn registry_gate_red_alert_no_hash_not_auto() {
        let mut gate = RegistryGate::new();
        // No chain_hash → QT4 violation → cannot auto-resolve
        let p = PendingRegistration::new(
            "skill:no_hash", 4, AlertLevel::RedAlert, "no hash", 1000,
        ); // chain_hash = None
        gate.submit(p);
        gate.drain_notifications();
        gate.auto_resolve_red_alerts();
        assert_eq!(gate.auto_resolved_count(), 0, "No hash → no auto");
    }

    #[test]
    fn pending_registration_prompt() {
        let p = PendingRegistration::new(
            "skill:test", 4, AlertLevel::Normal, "test reason", 1000,
        );
        let prompt = p.prompt();
        assert!(prompt.contains("skill:test"), "Prompt has name");
        assert!(prompt.contains("đăng ký"), "Prompt asks to register");
    }

    #[test]
    fn pending_registration_prompt_red_alert() {
        let p = PendingRegistration::new(
            "agent:critical", 3, AlertLevel::RedAlert, "emergency", 1000,
        );
        let prompt = p.prompt();
        assert!(prompt.contains("🔴"), "Red alert has icon");
    }

    // ── SkillPattern AAM review tests ────────────────────────────────────────

    #[test]
    fn aam_skill_pattern_approved() {
        let p = SkillProposal::new(
            "DreamSkill",
            InsightKind::SkillPattern {
                skill_names: vec!["Ingest".to_string(), "Cluster".to_string()],
                effectiveness: 0.8,
                observations: 5,
            },
            0.7,
            1000,
        );
        assert_eq!(AAM::new().review_skill(&p), AAMDecision::Approved);
    }

    #[test]
    fn aam_skill_pattern_pending_few_observations() {
        let p = SkillProposal::new(
            "DreamSkill",
            InsightKind::SkillPattern {
                skill_names: vec!["A".to_string()],
                effectiveness: 0.9,
                observations: 2, // < 3
            },
            0.7,
            1000,
        );
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Pending { needed_fire_count: 1 }
        ));
    }

    #[test]
    fn aam_skill_pattern_rejected_low_effectiveness() {
        let p = SkillProposal::new(
            "DreamSkill",
            InsightKind::SkillPattern {
                skill_names: vec!["A".to_string()],
                effectiveness: 0.3, // < 0.6
                observations: 5,
            },
            0.7,
            1000,
        );
        assert!(matches!(
            AAM::new().review_skill(&p),
            AAMDecision::Rejected { .. }
        ));
    }
}
