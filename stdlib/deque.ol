// stdlib/deque.ol — Double-ended queue for Olang
// Supports push/pop from both ends.

pub fn new() { Deque() }
pub fn push_back(q, val) { q.push_back(val) }
pub fn push_front(q, val) { q.push_front(val) }
pub fn pop_front(q) { q.pop_front() }
pub fn pop_back(q) { q.pop_back() }
pub fn peek_front(q) { q.peek_front() }
pub fn peek_back(q) { q.peek_back() }
pub fn len(q) { q.len() }
