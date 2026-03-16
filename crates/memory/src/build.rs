//! # build — Draft Zone + Hypothesis Testing
//!
//! L8 Build layer: staging area cho draft concepts.
//! Mỗi draft = chain + confidence + evidence count.
//! Promote khi confidence ≥ 0.90 (QT18 Honesty).
//! Discard bằng SupersedeQR — KHÔNG xóa (append-only QT10).
//!
//! Pipeline:
//!   draft "X causes Y" → DraftEntry
//!   evidence arrives   → fire_count++, confidence update
//!   verify             → check Silk paths + fire count
//!   promote            → DreamProposal → AAM → QR

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use olang::molecular::MolecularChain;
use silk::edge::EmotionTag;

use crate::proposal::DreamProposal;

// ─────────────────────────────────────────────────────────────────────────────
// DraftStatus
// ─────────────────────────────────────────────────────────────────────────────

/// Trạng thái của một draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftStatus {
    /// Đang trong draft zone — chưa đủ evidence
    Active,
    /// Đã promote thành QR proposal
    Promoted,
    /// Bị supersede/discard (vẫn giữ trong zone, append-only)
    Superseded,
}

// ─────────────────────────────────────────────────────────────────────────────
// DraftEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Một hypothesis trong draft zone.
#[derive(Debug, Clone)]
pub struct DraftEntry {
    /// MolecularChain đại diện hypothesis
    pub chain: MolecularChain,
    /// Mô tả hypothesis (human readable)
    pub description: String,
    /// Evidence count (mỗi lần reinforce → +1)
    pub fire_count: u32,
    /// Confidence score [0.0, 1.0]
    pub confidence: f32,
    /// Emotion context khi tạo
    pub emotion: EmotionTag,
    /// Trạng thái
    pub status: DraftStatus,
    /// Timestamp tạo
    pub created_at: i64,
    /// Timestamp cập nhật cuối
    pub updated_at: i64,
}

impl DraftEntry {
    /// Tạo draft mới.
    pub fn new(chain: MolecularChain, description: &str, confidence: f32, ts: i64) -> Self {
        Self {
            chain,
            description: String::from(description),
            fire_count: 1,
            confidence: confidence.clamp(0.0, 1.0),
            emotion: EmotionTag::NEUTRAL,
            status: DraftStatus::Active,
            created_at: ts,
            updated_at: ts,
        }
    }

    /// Reinforce: thêm evidence cho hypothesis.
    pub fn reinforce(&mut self, additional_confidence: f32, ts: i64) {
        self.fire_count += 1;
        // Confidence tăng dần với diminishing returns
        let boost = additional_confidence * (1.0 - self.confidence) * 0.5;
        self.confidence = (self.confidence + boost).clamp(0.0, 1.0);
        self.updated_at = ts;
    }

    /// Weaken: counter-evidence.
    pub fn weaken(&mut self, penalty: f32, ts: i64) {
        self.confidence = (self.confidence - penalty).clamp(0.0, 1.0);
        self.updated_at = ts;
    }

    /// Đủ điều kiện promote? (QT18: Honesty ≥ 0.90)
    pub fn is_promotable(&self) -> bool {
        self.status == DraftStatus::Active && self.confidence >= 0.90 && self.fire_count >= 5
    }

    /// Mark as promoted.
    pub fn promote(&mut self, ts: i64) {
        self.status = DraftStatus::Promoted;
        self.updated_at = ts;
    }

    /// Mark as superseded (append-only — not deleted).
    pub fn supersede(&mut self, ts: i64) {
        self.status = DraftStatus::Superseded;
        self.updated_at = ts;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BuildZone
// ─────────────────────────────────────────────────────────────────────────────

/// Draft zone: sandbox cho hypothesis testing.
///
/// Mọi draft sống ở đây cho đến khi:
///   promote → confidence ≥ 0.90 + fire ≥ 5 → DreamProposal → AAM
///   supersede → counter-evidence → mark superseded (không xóa)
pub struct BuildZone {
    /// Tất cả draft entries (append-only)
    entries: Vec<DraftEntry>,
    /// Promote threshold (default: 0.90 — QT18 Honesty)
    promote_threshold: f32,
    /// Min fire count to promote
    min_fire: u32,
}

impl BuildZone {
    /// Tạo BuildZone mới.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            promote_threshold: 0.90,
            min_fire: 5,
        }
    }

    /// Add a draft hypothesis.
    pub fn draft(
        &mut self,
        chain: MolecularChain,
        description: &str,
        confidence: f32,
        ts: i64,
    ) -> usize {
        let entry = DraftEntry::new(chain, description, confidence, ts);
        self.entries.push(entry);
        self.entries.len() - 1
    }

    /// Reinforce a draft with new evidence.
    pub fn reinforce(&mut self, index: usize, additional_confidence: f32, ts: i64) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            if entry.status == DraftStatus::Active {
                entry.reinforce(additional_confidence, ts);
                return true;
            }
        }
        false
    }

    /// Weaken a draft with counter-evidence.
    pub fn weaken(&mut self, index: usize, penalty: f32, ts: i64) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            if entry.status == DraftStatus::Active {
                entry.weaken(penalty, ts);
                return true;
            }
        }
        false
    }

    /// Check which drafts are ready for promotion.
    pub fn promotable(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.status == DraftStatus::Active
                    && e.confidence >= self.promote_threshold
                    && e.fire_count >= self.min_fire
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Promote a draft → DreamProposal for AAM review.
    pub fn promote(&mut self, index: usize, ts: i64) -> Option<DreamProposal> {
        let entry = self.entries.get_mut(index)?;
        if entry.status != DraftStatus::Active {
            return None;
        }
        entry.promote(ts);

        Some(DreamProposal::promote_qr(
            entry.chain.chain_hash(),
            entry.fire_count,
            entry.confidence,
            ts,
        ))
    }

    /// Supersede a draft (mark as replaced — append-only, not deleted).
    pub fn supersede(&mut self, index: usize, ts: i64) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            if entry.status == DraftStatus::Active {
                entry.supersede(ts);
                return true;
            }
        }
        false
    }

    /// Get a draft by index.
    pub fn get(&self, index: usize) -> Option<&DraftEntry> {
        self.entries.get(index)
    }

    /// Count active drafts.
    pub fn active_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.status == DraftStatus::Active)
            .count()
    }

    /// Count total entries (including superseded).
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// All active drafts.
    pub fn active(&self) -> Vec<(usize, &DraftEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.status == DraftStatus::Active)
            .collect()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let active = self.active_count();
        let promoted = self.entries.iter().filter(|e| e.status == DraftStatus::Promoted).count();
        let superseded = self
            .entries
            .iter()
            .filter(|e| e.status == DraftStatus::Superseded)
            .count();
        alloc::format!(
            "BuildZone: {} active, {} promoted, {} superseded (total: {})",
            active,
            promoted,
            superseded,
            self.entries.len()
        )
    }
}

impl Default for BuildZone {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ConsolidationPhase — Temporal scheduling
// ─────────────────────────────────────────────────────────────────────────────

/// Circadian phases for temporal consolidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsolidationPhase {
    /// Active learning: new inputs → STM, co-activate Silk
    Day,
    /// Idle consolidation: cluster STM, reinforce BuildZone drafts
    Dusk,
    /// Deep consolidation: Dream cycle, promote/supersede drafts
    Night,
    /// Review: scan BuildZone, prepare promote candidates
    Dawn,
}

/// Temporal consolidation scheduler.
///
/// Tracks system idle time and triggers appropriate phases:
///   idle < 60s      → Day (active)
///   60s ≤ idle < 5m → Dusk (light consolidation)
///   5m ≤ idle < 30m → Night (deep dream)
///   after Night     → Dawn (review)
pub struct ConsolidationScheduler {
    /// Current phase
    pub phase: ConsolidationPhase,
    /// Timestamp of last user activity
    last_activity: i64,
    /// Timestamp of last phase transition
    last_transition: i64,
    /// Number of Dream cycles completed in current Night
    dreams_this_night: u32,
    /// Max dreams per night (avoid infinite dreaming)
    max_dreams: u32,
}

impl ConsolidationScheduler {
    /// Create new scheduler.
    pub fn new(ts: i64) -> Self {
        Self {
            phase: ConsolidationPhase::Day,
            last_activity: ts,
            last_transition: ts,
            dreams_this_night: 0,
            max_dreams: 5,
        }
    }

    /// Record user activity (resets idle timer).
    pub fn activity(&mut self, ts: i64) {
        self.last_activity = ts;
        if self.phase != ConsolidationPhase::Day {
            self.phase = ConsolidationPhase::Day;
            self.last_transition = ts;
            self.dreams_this_night = 0;
        }
    }

    /// Update phase based on idle time.
    /// Returns true if phase changed.
    pub fn tick(&mut self, ts: i64) -> bool {
        let idle_secs = (ts - self.last_activity) / 1000; // ms → seconds
        let old_phase = self.phase;

        self.phase = match idle_secs {
            0..=59 => ConsolidationPhase::Day,
            60..=299 => ConsolidationPhase::Dusk,
            300..=1799 => {
                if self.dreams_this_night < self.max_dreams {
                    ConsolidationPhase::Night
                } else {
                    ConsolidationPhase::Dawn // done dreaming
                }
            }
            _ => ConsolidationPhase::Dawn, // > 30 min idle
        };

        if self.phase != old_phase {
            self.last_transition = ts;
            old_phase != self.phase // return true if changed
        } else {
            false
        }
    }

    /// Record a dream cycle completion.
    pub fn dream_completed(&mut self) {
        self.dreams_this_night += 1;
    }

    /// Should trigger Dream cycle?
    pub fn should_dream(&self) -> bool {
        self.phase == ConsolidationPhase::Night && self.dreams_this_night < self.max_dreams
    }

    /// Should scan BuildZone for promotions?
    pub fn should_review(&self) -> bool {
        self.phase == ConsolidationPhase::Dawn
    }

    /// Idle duration in seconds.
    pub fn idle_secs(&self, ts: i64) -> i64 {
        (ts - self.last_activity) / 1000
    }

    /// Current phase name.
    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            ConsolidationPhase::Day => "Day",
            ConsolidationPhase::Dusk => "Dusk",
            ConsolidationPhase::Night => "Night",
            ConsolidationPhase::Dawn => "Dawn",
        }
    }
}

impl Default for ConsolidationScheduler {
    fn default() -> Self {
        Self::new(0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    fn make_chain() -> MolecularChain {
        if skip() {
            MolecularChain::empty()
        } else {
            olang::encoder::encode_codepoint(0x1F525) // 🔥
        }
    }

    // ── DraftEntry ──────────────────────────────────────────────────────────

    #[test]
    fn draft_entry_new() {
        let chain = make_chain();
        let entry = DraftEntry::new(chain, "fire causes heat", 0.5, 1000);
        assert_eq!(entry.fire_count, 1);
        assert!((entry.confidence - 0.5).abs() < 0.01);
        assert_eq!(entry.status, DraftStatus::Active);
    }

    #[test]
    fn draft_reinforce_increases_confidence() {
        let chain = make_chain();
        let mut entry = DraftEntry::new(chain, "test", 0.5, 1000);
        let before = entry.confidence;
        entry.reinforce(0.3, 2000);
        assert!(entry.confidence > before, "Reinforce increases confidence");
        assert_eq!(entry.fire_count, 2);
    }

    #[test]
    fn draft_reinforce_diminishing_returns() {
        let chain = make_chain();
        let mut entry = DraftEntry::new(chain, "test", 0.8, 1000);
        entry.reinforce(0.5, 2000);
        let high_conf = entry.confidence;
        // At high confidence, boost is smaller
        assert!(high_conf < 0.95, "Diminishing returns: {}", high_conf);
    }

    #[test]
    fn draft_weaken_decreases_confidence() {
        let chain = make_chain();
        let mut entry = DraftEntry::new(chain, "test", 0.7, 1000);
        entry.weaken(0.3, 2000);
        assert!((entry.confidence - 0.4).abs() < 0.01);
    }

    #[test]
    fn draft_promotable_threshold() {
        let chain = make_chain();
        let mut entry = DraftEntry::new(chain, "test", 0.95, 1000);
        // fire_count = 1, need ≥ 5
        assert!(!entry.is_promotable(), "fire=1 < 5");
        for i in 0..4 {
            entry.reinforce(0.1, 2000 + i * 100);
        }
        assert!(entry.fire_count >= 5);
        assert!(entry.is_promotable(), "Now promotable");
    }

    // ── BuildZone ───────────────────────────────────────────────────────────

    #[test]
    fn build_zone_draft_and_count() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        zone.draft(chain, "test hypothesis", 0.5, 1000);
        assert_eq!(zone.active_count(), 1);
        assert_eq!(zone.total_count(), 1);
    }

    #[test]
    fn build_zone_reinforce() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        let idx = zone.draft(chain, "test", 0.5, 1000);
        assert!(zone.reinforce(idx, 0.3, 2000));
        let entry = zone.get(idx).unwrap();
        assert_eq!(entry.fire_count, 2);
    }

    #[test]
    fn build_zone_promote() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        let idx = zone.draft(chain, "test", 0.95, 1000);
        // Reinforce to fire_count ≥ 5
        for i in 0..5 {
            zone.reinforce(idx, 0.1, 2000 + i * 100);
        }
        let promotable = zone.promotable();
        assert!(!promotable.is_empty());

        let proposal = zone.promote(idx, 3000);
        assert!(proposal.is_some());
        assert_eq!(zone.get(idx).unwrap().status, DraftStatus::Promoted);
        assert_eq!(zone.active_count(), 0);
    }

    #[test]
    fn build_zone_supersede_not_delete() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        let idx = zone.draft(chain, "wrong hypothesis", 0.3, 1000);
        assert!(zone.supersede(idx, 2000));
        // Still in zone (append-only)
        assert_eq!(zone.total_count(), 1);
        assert_eq!(zone.active_count(), 0);
        assert_eq!(zone.get(idx).unwrap().status, DraftStatus::Superseded);
    }

    #[test]
    fn build_zone_weaken() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        let idx = zone.draft(chain, "test", 0.7, 1000);
        assert!(zone.weaken(idx, 0.5, 2000));
        let entry = zone.get(idx).unwrap();
        assert!((entry.confidence - 0.2).abs() < 0.01);
    }

    #[test]
    fn build_zone_summary() {
        let mut zone = BuildZone::new();
        let c = make_chain();
        zone.draft(c.clone(), "a", 0.5, 1000);
        zone.draft(c.clone(), "b", 0.5, 1000);
        zone.supersede(1, 2000);
        let s = zone.summary();
        assert!(s.contains("1 active"), "summary: {}", s);
        assert!(s.contains("1 superseded"), "summary: {}", s);
    }

    #[test]
    fn build_zone_cannot_reinforce_superseded() {
        let mut zone = BuildZone::new();
        let chain = make_chain();
        let idx = zone.draft(chain, "test", 0.5, 1000);
        zone.supersede(idx, 2000);
        assert!(!zone.reinforce(idx, 0.3, 3000), "Cannot reinforce superseded");
    }

    // ── ConsolidationScheduler ──────────────────────────────────────────────

    #[test]
    fn scheduler_starts_day() {
        let sched = ConsolidationScheduler::new(0);
        assert_eq!(sched.phase, ConsolidationPhase::Day);
    }

    #[test]
    fn scheduler_transitions_to_dusk() {
        let mut sched = ConsolidationScheduler::new(0);
        // 120 seconds idle
        let changed = sched.tick(120_000);
        assert!(changed);
        assert_eq!(sched.phase, ConsolidationPhase::Dusk);
    }

    #[test]
    fn scheduler_transitions_to_night() {
        let mut sched = ConsolidationScheduler::new(0);
        // 10 minutes idle
        let changed = sched.tick(600_000);
        assert!(changed);
        assert_eq!(sched.phase, ConsolidationPhase::Night);
        assert!(sched.should_dream());
    }

    #[test]
    fn scheduler_activity_resets_to_day() {
        let mut sched = ConsolidationScheduler::new(0);
        sched.tick(600_000); // Night
        sched.activity(601_000);
        assert_eq!(sched.phase, ConsolidationPhase::Day);
        assert!(!sched.should_dream());
    }

    #[test]
    fn scheduler_max_dreams() {
        let mut sched = ConsolidationScheduler::new(0);
        sched.tick(600_000); // Night
        for _ in 0..5 {
            sched.dream_completed();
        }
        sched.tick(600_000);
        // Max dreams reached → Dawn
        assert_eq!(sched.phase, ConsolidationPhase::Dawn);
        assert!(!sched.should_dream());
        assert!(sched.should_review());
    }

    #[test]
    fn scheduler_idle_secs() {
        let sched = ConsolidationScheduler::new(1000);
        assert_eq!(sched.idle_secs(61_000), 60);
    }

    #[test]
    fn scheduler_phase_names() {
        let mut sched = ConsolidationScheduler::new(0);
        assert_eq!(sched.phase_name(), "Day");
        sched.tick(120_000);
        assert_eq!(sched.phase_name(), "Dusk");
        sched.tick(600_000);
        assert_eq!(sched.phase_name(), "Night");
    }
}
