//! # concurrency — Concurrency Model cho HomeOS
//!
//! ## CAP Tradeoff
//!
//! HomeOS chọn: **AP** (Availability + Partition tolerance)
//!   - Consistency: eventual (Silk edges converge qua Hebbian decay)
//!   - Availability: luôn phản hồi (degrade to in-memory nếu disk fail)
//!   - Partition: Workers hoạt động độc lập, sync qua ISL khi reconnect
//!
//! ## Concurrency Primitives
//!
//! Vì `no_std`:
//!   - Không có Mutex, RwLock, channel
//!   - Thay vào: single-threaded event loop + ISL message queue
//!   - Worker isolation: mỗi Worker = 1 process/task riêng
//!   - Chief coordination: ISL urgent queue cho Emergency
//!
//! ## Consensus
//!
//! AAM = single-point consensus:
//!   - Proposals từ Dream/Worker → AAM review → Approve/Reject
//!   - Không cần distributed consensus (AAM là ý thức duy nhất)
//!   - Worker conflict: Chief resolve, escalate lên AAM nếu cần
//!
//! ## Data Race Prevention
//!
//! - origin.olang: append-only → no write conflict
//! - Registry: rebuild từ file → deterministic
//! - Silk: co_activate idempotent (same inputs → same output)
//! - STM: single-writer (LearningLoop owns it)

/// Trạng thái sync giữa Chief và Workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    /// Worker đang online, ISL connected.
    Connected,
    /// Worker offline — hoạt động độc lập.
    Partitioned,
    /// Worker vừa reconnect — đang sync.
    Syncing,
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Last-write-wins (dùng timestamp).
    LastWriteWins,
    /// Merge cả hai (append-only → không mất data).
    MergeBoth,
    /// Escalate lên AAM (dùng cho security decisions).
    EscalateAAM,
}

/// Metadata cho ISL sync session.
#[derive(Debug, Clone)]
pub struct SyncSession {
    /// Worker ISL address.
    pub worker_addr: [u8; 4],
    /// Trạng thái hiện tại.
    pub state: SyncState,
    /// Timestamp bắt đầu sync.
    pub started_at: i64,
    /// Số records đã sync.
    pub records_synced: u32,
    /// Conflict strategy cho session này.
    pub strategy: ConflictStrategy,
}

impl SyncSession {
    /// Tạo session mới.
    pub fn new(worker_addr: [u8; 4], ts: i64) -> Self {
        Self {
            worker_addr,
            state: SyncState::Syncing,
            started_at: ts,
            records_synced: 0,
            strategy: ConflictStrategy::MergeBoth,
        }
    }

    /// Đánh dấu sync hoàn tất.
    pub fn complete(&mut self) {
        self.state = SyncState::Connected;
    }

    /// Đánh dấu mất kết nối.
    pub fn partition(&mut self) {
        self.state = SyncState::Partitioned;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_session_lifecycle() {
        let mut session = SyncSession::new([0x01, 0x02, 0x03, 0x04], 1000);
        assert_eq!(session.state, SyncState::Syncing);

        session.records_synced = 42;
        session.complete();
        assert_eq!(session.state, SyncState::Connected);

        session.partition();
        assert_eq!(session.state, SyncState::Partitioned);
    }

    #[test]
    fn conflict_strategy_default() {
        let session = SyncSession::new([0; 4], 0);
        assert_eq!(session.strategy, ConflictStrategy::MergeBoth);
    }
}
