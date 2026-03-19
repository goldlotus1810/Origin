//! Integration: ISL messaging ↔ Agent hierarchy

use isl::address::ISLAddress;
use isl::message::{ISLMessage, MsgType};
use isl::queue::ISLQueue;

#[test]
fn isl_address_roundtrip() {
    let addr = ISLAddress::new(1, 2, 3, 4);
    let bytes = addr.to_bytes();
    let decoded = ISLAddress::from_bytes(bytes);
    assert_eq!(decoded.layer, 1);
    assert_eq!(decoded.group, 2);
    assert_eq!(decoded.subgroup, 3);
    assert_eq!(decoded.index, 4);
}

#[test]
fn isl_address_different() {
    let a1 = ISLAddress::new(0, 0, 0, 1);
    let a2 = ISLAddress::new(0, 0, 0, 2);
    assert_ne!(a1.to_bytes(), a2.to_bytes());
}

#[test]
fn isl_message_roundtrip() {
    let chief = ISLAddress::new(1, 1, 0, 0);
    let worker = ISLAddress::new(2, 1, 1, 0);
    let msg = ISLMessage::new(chief, worker, MsgType::Text);
    let bytes = msg.to_bytes();
    let decoded = ISLMessage::from_bytes(&bytes).expect("must decode");
    assert_eq!(decoded.from.layer, chief.layer);
    assert_eq!(decoded.to.layer, worker.layer);
    assert_eq!(decoded.msg_type, MsgType::Text);
}

#[test]
fn isl_message_size() {
    assert_eq!(ISLMessage::SIZE, 12);
}

#[test]
fn isl_queue_urgent_before_normal() {
    let mut queue = ISLQueue::new();
    let a = ISLAddress::new(0, 0, 0, 0);
    let b = ISLAddress::new(1, 1, 0, 0);

    queue.push(ISLMessage::new(a, b, MsgType::Text));
    queue.push(ISLMessage::new(a, b, MsgType::Emergency));

    let first = queue.pop().expect("must not be empty");
    assert_eq!(first.msg_type, MsgType::Emergency, "urgent must come first");
    let second = queue.pop().expect("must have second");
    assert_eq!(second.msg_type, MsgType::Text);
}

#[test]
fn isl_queue_tick_is_urgent() {
    let mut queue = ISLQueue::new();
    let a = ISLAddress::new(0, 0, 0, 0);
    let b = ISLAddress::new(1, 0, 0, 0);

    queue.push(ISLMessage::new(a, b, MsgType::Learn));
    queue.push(ISLMessage::new(a, b, MsgType::Tick));

    let first = queue.pop().unwrap();
    assert_eq!(first.msg_type, MsgType::Tick, "Tick must come first");
}

#[test]
fn tier_addresses() {
    let aam = ISLAddress::new(0, 0, 0, 0);
    assert_eq!(aam.layer, 0, "AAM is tier 0");
    let chief = ISLAddress::new(1, 1, 0, 0);
    assert_eq!(chief.layer, 1, "Chief is tier 1");
    let worker = ISLAddress::new(2, 1, 1, 0);
    assert_eq!(worker.layer, 2, "Worker is tier 2");
}

#[test]
fn isl_queue_drain() {
    let mut queue = ISLQueue::new();
    let a = ISLAddress::new(0, 0, 0, 0);
    queue.push(ISLMessage::new(a, a, MsgType::Text));
    queue.push(ISLMessage::new(a, a, MsgType::Emergency));
    queue.pop();
    queue.pop();
    assert!(queue.pop().is_none(), "queue must be empty");
}
