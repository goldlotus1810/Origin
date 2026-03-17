# isl

> ISL (Internal Signaling Language) address and message system — 4-byte addressing and 12-byte binary messages for HomeOS inter-component communication.

## Dependencies
- olang

## Files
| File | Purpose |
|------|---------|
| lib.rs | Crate root; re-exports `address`, `message`, `codec`, `queue` modules (`#![no_std]`) |
| address.rs | `ISLAddress` (4 bytes: layer/group/subgroup/index) and `ISLAllocator` for collision-free allocation |
| message.rs | `ISLMessage` (12 bytes), `ISLFrame` (header + extended body), `MsgType` enum with 13 message types |
| codec.rs | `ISLCodec` — encode/decode with XOR checksum; AES-256-GCM key field reserved for future encryption |
| queue.rs | `ISLQueue` — priority queue that processes urgent messages (Emergency/Tick) before normal FIFO |

## Key API
```rust
// 4-byte address
pub const fn ISLAddress::new(layer: u8, group: u8, subgroup: u8, index: u8) -> Self;
pub fn ISLAddress::to_bytes(self) -> [u8; 4];
pub fn ISLAddress::from_bytes(b: [u8; 4]) -> Self;

// 12-byte message
pub fn ISLMessage::new(from: ISLAddress, to: ISLAddress, msg_type: MsgType) -> Self;
pub fn ISLMessage::to_bytes(self) -> [u8; 12];
pub fn ISLMessage::from_bytes(b: &[u8]) -> Option<Self>;

// Codec with checksum
pub fn ISLCodec::encode(&self, msg: &ISLMessage) -> Vec<u8>;
pub fn ISLCodec::decode(&self, bytes: &[u8]) -> Result<ISLMessage, ISLError>;

// Priority queue
pub fn ISLQueue::push(&mut self, msg: ISLMessage) -> bool;
pub fn ISLQueue::pop(&mut self) -> Option<ISLMessage>;
```

## Rules
- `#![no_std]` — uses `alloc` only
- ISLMessage base is always 12 bytes (95.7% smaller than equivalent JSON)
- ISLAllocator enforces unique addresses via auto-incrementing counters per (layer, group, subgroup) namespace
- Emergency and Tick messages are classified as urgent and dequeued first
- ISLFrame extends messages with a variable-length body (up to 65535 bytes)
- Checksum is XOR-based; AES-256-GCM encryption is reserved but not yet implemented

## Test
```bash
cargo test -p isl
```
