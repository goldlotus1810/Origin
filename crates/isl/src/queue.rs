//! # queue — ISLQueue
//!
//! Hàng đợi xử lý message theo priority.
//!
//! Emergency/Tick → ưu tiên cao
//! Còn lại → FIFO

extern crate alloc;
use alloc::collections::VecDeque;
use crate::message::ISLMessage;

// ─────────────────────────────────────────────────────────────────────────────
// ISLQueue
// ─────────────────────────────────────────────────────────────────────────────

const MAX_QUEUE: usize = 256;

/// Hàng đợi message theo priority.
#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct ISLQueue {
    urgent:  VecDeque<ISLMessage>, // Emergency + Tick
    normal:  VecDeque<ISLMessage>, // còn lại
}

impl ISLQueue {
    pub fn new() -> Self { Self::default() }

    /// Thêm message vào queue.
    /// Trả false nếu queue đầy.
    pub fn push(&mut self, msg: ISLMessage) -> bool {
        if msg.msg_type.is_urgent() {
            if self.urgent.len() >= MAX_QUEUE { return false; }
            self.urgent.push_back(msg);
        } else {
            if self.normal.len() >= MAX_QUEUE { return false; }
            self.normal.push_back(msg);
        }
        true
    }

    /// Lấy message tiếp theo — urgent trước, normal sau.
    pub fn pop(&mut self) -> Option<ISLMessage> {
        self.urgent.pop_front().or_else(|| self.normal.pop_front())
    }

    /// Số message trong queue.
    pub fn len(&self) -> usize { self.urgent.len() + self.normal.len() }

    /// Queue rỗng?
    pub fn is_empty(&self) -> bool { self.urgent.is_empty() && self.normal.is_empty() }

    /// Drain tất cả messages.
    pub fn drain(&mut self) -> alloc::vec::Vec<ISLMessage> {
        let mut out = alloc::vec::Vec::new();
        while let Some(m) = self.pop() { out.push(m); }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MsgType;
    use crate::address::ISLAddress;

    fn addr(i: u8) -> ISLAddress { ISLAddress::new(0, 0, 0, i) }

    #[test]
    fn urgent_before_normal() {
        let mut q = ISLQueue::new();
        q.push(ISLMessage::new(addr(1), addr(0), MsgType::Text));
        q.push(ISLMessage::new(addr(1), addr(0), MsgType::Query));
        q.push(ISLMessage::emergency(addr(1), 0x01));

        let first = q.pop().unwrap();
        assert_eq!(first.msg_type, MsgType::Emergency, "Emergency trước");
    }

    #[test]
    fn fifo_within_priority() {
        let mut q = ISLQueue::new();
        q.push(ISLMessage::new(addr(1), addr(0), MsgType::Text));
        q.push(ISLMessage::new(addr(2), addr(0), MsgType::Query));

        let a = q.pop().unwrap();
        let b = q.pop().unwrap();
        assert_eq!(a.from, addr(1), "FIFO: Text trước");
        assert_eq!(b.from, addr(2), "FIFO: Query sau");
    }

    #[test]
    fn empty_pop_returns_none() {
        let mut q = ISLQueue::new();
        assert!(q.pop().is_none());
    }

    #[test]
    fn len_and_drain() {
        let mut q = ISLQueue::new();
        for i in 0..5 {
            q.push(ISLMessage::tick(addr(i), i));
        }
        assert_eq!(q.len(), 5);
        let drained = q.drain();
        assert_eq!(drained.len(), 5);
        assert!(q.is_empty());
    }
}
