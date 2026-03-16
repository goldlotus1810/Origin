//! # router — Agent Message Router
//!
//! Central dispatcher kết nối toàn bộ agent hierarchy:
//!   Worker → Chief → LeoAI → AAM → LeoAI (feedback loop)
//!
//! Trước đây: caller phải manually route messages giữa agents.
//! Giờ: `MessageRouter.tick()` pump tất cả — 1 call duy nhất.
//!
//! ```text
//! Worker.flush() → reports
//!   ↓ route by worker→chief registry
//! Chief.receive_frame() → IngestedReport
//!   ↓ drain_reports()
//! LeoAI.ingest() → proposals
//!   ↓ flush_outbox()
//! AAM.review() → decisions
//!   ↓ receive_aam_decision()
//! LeoAI ← Approved/Rejected
//!   ↓ if approved
//! UserAuthority.submit() → pending user confirmation
//! ```

extern crate alloc;
use alloc::vec::Vec;

use agents::chief::{Chief, ChiefKind};
use agents::leo::LeoAI;
use agents::worker::Worker;
use isl::address::ISLAddress;
use isl::message::{ISLMessage, MsgType};
use memory::proposal::{AAM, AAMDecision, DreamProposal, UserAuthority};

// ─────────────────────────────────────────────────────────────────────────────
// TickStats
// ─────────────────────────────────────────────────────────────────────────────

/// Statistics from a single tick() cycle.
#[derive(Debug, Clone, Default)]
pub struct TickStats {
    /// Worker reports routed to Chiefs
    pub worker_reports: u32,
    /// Chief reports routed to LeoAI
    pub chief_reports: u32,
    /// Proposals sent to AAM
    pub proposals_sent: u32,
    /// Proposals approved by AAM
    pub proposals_approved: u32,
    /// Proposals rejected by AAM
    pub proposals_rejected: u32,
    /// Proposals pending more evidence
    pub proposals_pending: u32,
    /// Proposals submitted to UserAuthority
    pub user_submissions: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// RouterStats
// ─────────────────────────────────────────────────────────────────────────────

/// Cumulative router statistics.
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    /// Total ticks processed
    pub ticks: u64,
    /// Total worker reports routed
    pub total_worker_reports: u64,
    /// Total chief reports routed
    pub total_chief_reports: u64,
    /// Total proposals to AAM
    pub total_proposals: u64,
    /// Total approved
    pub total_approved: u64,
    /// Total rejected
    pub total_rejected: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// MessageRouter
// ─────────────────────────────────────────────────────────────────────────────

/// Central message dispatcher for the agent hierarchy.
///
/// Hierarchy enforced:
///   ✅ Worker → Chief (via flush/receive_frame)
///   ✅ Chief → LeoAI (via drain_reports/ingest)
///   ✅ LeoAI → AAM (via flush_outbox/review)
///   ✅ AAM → LeoAI (via receive_aam_decision)
///   ❌ Worker → Worker (blocked by design)
///   ❌ AAM → Worker (blocked by design)
pub struct MessageRouter {
    /// AAM — stateless decision maker (tier 0)
    aam: AAM,
    /// UserAuthority — user confirmation for QR writes
    user_authority: UserAuthority,
    /// Cumulative stats
    stats: RouterStats,
    /// Pending writes from LeoAI (bytes to flush to disk)
    pending_writes: Vec<u8>,
}

impl MessageRouter {
    /// Create a new router with auto-approve enabled.
    pub fn new() -> Self {
        let mut ua = UserAuthority::new();
        ua.set_auto_approve(true);
        Self {
            aam: AAM::new(),
            user_authority: ua,
            stats: RouterStats::default(),
            pending_writes: Vec::new(),
        }
    }

    /// Run one tick of the message routing cycle.
    ///
    /// This is the SINGLE function needed to pump the entire agent hierarchy.
    /// Call after processing user input or on a timer.
    ///
    /// Flow: Workers → Chiefs → LeoAI → AAM → feedback
    pub fn tick(
        &mut self,
        workers: &mut [Worker],
        chiefs: &mut [Chief],
        leo: &mut LeoAI,
        ts: i64,
    ) -> TickStats {
        let mut tick = TickStats::default();

        // ── Phase 1: Workers → Chiefs ────────────────────────────────────────
        for worker in workers.iter_mut() {
            let reports = worker.flush();
            for report in reports {
                tick.worker_reports += 1;
                // Route to the Chief that registered this Worker
                let worker_key = report.frame.header.from.to_u32();
                let mut routed = false;
                for chief in chiefs.iter_mut() {
                    if chief.workers.contains_key(&worker_key) {
                        chief.receive_frame(report.frame.clone(), ts);
                        routed = true;
                        break;
                    }
                }
                // Fallback: send to General chief
                if !routed {
                    for chief in chiefs.iter_mut() {
                        if chief.kind == ChiefKind::General {
                            chief.receive_frame(report.frame.clone(), ts);
                            break;
                        }
                    }
                }
            }
        }

        // ── Phase 2: Chiefs → LeoAI ─────────────────────────────────────────
        for chief in chiefs.iter_mut() {
            let reports = chief.drain_reports();
            for report in reports {
                tick.chief_reports += 1;
                leo.ingest(report, ts);
            }
        }

        // ── Phase 3: LeoAI → AAM ────────────────────────────────────────────
        let outbox = leo.flush_outbox();
        for frame in &outbox {
            if frame.header.msg_type == MsgType::Propose {
                tick.proposals_sent += 1;
                // Extract proposal info from ISL frame
                let chain_hash = payload_to_hash(&frame.header.payload);
                let fire_count = if frame.body.len() >= 2 {
                    ((frame.body[0] as u32) << 8) | (frame.body[1] as u32)
                } else {
                    3
                };
                let confidence = if frame.body.len() >= 3 {
                    frame.body[2] as f32 / 255.0
                } else {
                    0.6
                };

                let proposal =
                    DreamProposal::promote_qr(chain_hash, fire_count, confidence, ts);
                let decision = self.aam.review(&proposal);

                match decision {
                    AAMDecision::Approved => {
                        tick.proposals_approved += 1;
                        // Send Approved back to LeoAI
                        let idx = leo.pending.len().saturating_sub(1) as u8;
                        let ack = ISLMessage {
                            from: ISLAddress::ROOT,
                            to: leo.addr,
                            msg_type: MsgType::Approved,
                            payload: [idx, 0, 0],
                        };
                        leo.receive_aam_decision(ack, ts);

                        // Submit to UserAuthority for final confirmation
                        self.user_authority.submit(proposal, ts);
                        tick.user_submissions += 1;
                    }
                    AAMDecision::Rejected { .. } => {
                        tick.proposals_rejected += 1;
                        let idx = leo.pending.len().saturating_sub(1) as u8;
                        let nack = ISLMessage {
                            from: ISLAddress::ROOT,
                            to: leo.addr,
                            msg_type: MsgType::Nack,
                            payload: [idx, 0, 0],
                        };
                        leo.receive_aam_decision(nack, ts);
                    }
                    AAMDecision::Pending { .. } => {
                        tick.proposals_pending += 1;
                    }
                }
            }
        }

        // ── Phase 4: Drain pending writes from LeoAI ─────────────────────────
        if leo.has_pending_writes() {
            self.pending_writes
                .extend_from_slice(&leo.drain_pending_writes());
        }

        // ── Phase 5: Dream if idle ───────────────────────────────────────────
        leo.try_dream_if_idle(ts);

        // ── Update cumulative stats ──────────────────────────────────────────
        self.stats.ticks += 1;
        self.stats.total_worker_reports += tick.worker_reports as u64;
        self.stats.total_chief_reports += tick.chief_reports as u64;
        self.stats.total_proposals += tick.proposals_sent as u64;
        self.stats.total_approved += tick.proposals_approved as u64;
        self.stats.total_rejected += tick.proposals_rejected as u64;

        tick
    }

    /// Get cumulative stats.
    pub fn stats(&self) -> &RouterStats {
        &self.stats
    }

    /// Access UserAuthority (for user confirmation of proposals).
    pub fn user_authority(&self) -> &UserAuthority {
        &self.user_authority
    }

    /// Mutable access to UserAuthority.
    pub fn user_authority_mut(&mut self) -> &mut UserAuthority {
        &mut self.user_authority
    }

    /// Drain pending writes (bytes to flush to disk).
    pub fn drain_pending_writes(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.pending_writes)
    }

    /// Has bytes waiting to be written to disk?
    pub fn has_pending_writes(&self) -> bool {
        !self.pending_writes.is_empty()
    }

    /// Summary string for display.
    pub fn summary(&self) -> alloc::string::String {
        alloc::format!(
            "Router ○\n\
             Ticks        : {}\n\
             Worker→Chief : {}\n\
             Chief→LeoAI  : {}\n\
             Proposals    : {} (approved: {}, rejected: {})\n\
             User pending : {}",
            self.stats.ticks,
            self.stats.total_worker_reports,
            self.stats.total_chief_reports,
            self.stats.total_proposals,
            self.stats.total_approved,
            self.stats.total_rejected,
            self.user_authority.pending_count(),
        )
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract chain_hash from ISL 3-byte payload.
fn payload_to_hash(payload: &[u8; 3]) -> u64 {
    ((payload[0] as u64) << 16) | ((payload[1] as u64) << 8) | (payload[2] as u64)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agents::worker::WorkerKind;

    fn addrs() -> (ISLAddress, ISLAddress, ISLAddress) {
        let aam = ISLAddress::new(0, 0, 0, 0);
        let leo = ISLAddress::new(0, 0, 0, 1);
        let chief = ISLAddress::new(0, 0, 0, 2);
        (aam, leo, chief)
    }

    #[test]
    fn router_new_defaults() {
        let r = MessageRouter::new();
        assert_eq!(r.stats.ticks, 0);
        assert!(r.user_authority.is_auto_approve());
    }

    #[test]
    fn tick_empty_no_crash() {
        let mut r = MessageRouter::new();
        let (aam, leo_addr, _) = addrs();
        let mut leo = LeoAI::new(leo_addr, aam);
        let mut workers: Vec<Worker> = Vec::new();
        let mut chiefs: Vec<Chief> = Vec::new();

        let stats = r.tick(&mut workers, &mut chiefs, &mut leo, 1000);
        assert_eq!(stats.worker_reports, 0);
        assert_eq!(stats.chief_reports, 0);
        assert_eq!(r.stats.ticks, 1);
    }

    #[test]
    fn tick_increments_ticks() {
        let mut r = MessageRouter::new();
        let (aam, leo_addr, _) = addrs();
        let mut leo = LeoAI::new(leo_addr, aam);
        let mut workers: Vec<Worker> = Vec::new();
        let mut chiefs: Vec<Chief> = Vec::new();

        r.tick(&mut workers, &mut chiefs, &mut leo, 1000);
        r.tick(&mut workers, &mut chiefs, &mut leo, 2000);
        r.tick(&mut workers, &mut chiefs, &mut leo, 3000);
        assert_eq!(r.stats.ticks, 3);
    }

    #[test]
    fn chief_domain_routing() {
        let (aam, leo_addr, chief_addr) = addrs();
        let worker_addr = ISLAddress::new(0, 0, 0, 10);

        let mut home_chief = Chief::new(chief_addr, aam, leo_addr, ChiefKind::Home);
        // Sensor → Home Chief = OK
        assert!(home_chief.register_worker(worker_addr, WorkerKind::Sensor as u8, 0));

        let mut vision_chief = Chief::new(
            ISLAddress::new(0, 0, 0, 3),
            aam,
            leo_addr,
            ChiefKind::Vision,
        );
        // Sensor → Vision Chief = rejected
        assert!(!vision_chief.register_worker(worker_addr, WorkerKind::Sensor as u8, 0));
    }

    #[test]
    fn router_summary() {
        let r = MessageRouter::new();
        let s = r.summary();
        assert!(s.contains("Router"), "summary: {}", s);
        assert!(s.contains("Ticks"), "summary: {}", s);
        assert!(s.contains("Worker→Chief"), "summary: {}", s);
    }

    #[test]
    fn pending_writes_lifecycle() {
        let mut r = MessageRouter::new();
        assert!(!r.has_pending_writes());
        r.pending_writes.extend_from_slice(&[0x01, 0x02, 0x03]);
        assert!(r.has_pending_writes());
        let bytes = r.drain_pending_writes();
        assert_eq!(bytes.len(), 3);
        assert!(!r.has_pending_writes());
    }

    #[test]
    fn payload_to_hash_encoding() {
        assert_eq!(payload_to_hash(&[0xAB, 0xCD, 0xEF]), 0xABCDEF);
        assert_eq!(payload_to_hash(&[0x00, 0x00, 0x00]), 0);
        assert_eq!(payload_to_hash(&[0xFF, 0xFF, 0xFF]), 0xFFFFFF);
    }

    #[test]
    fn user_authority_accessible() {
        let mut r = MessageRouter::new();
        assert!(r.user_authority().is_auto_approve());
        r.user_authority_mut().set_auto_approve(false);
        assert!(!r.user_authority().is_auto_approve());
    }
}
