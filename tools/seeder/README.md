# seeder

> CLI tool that seeds `origin.olang` with L0 nodes from Unicode 18.0 codepoints, creating the initial knowledge base.

## Dependencies
- ucd
- olang (with `std` feature)

## Files
| File | Purpose |
|------|---------|
| src/main.rs | Seeds 34 L0 nodes (fire, water, earth, etc.) into `origin.olang` with QR signatures and multilingual aliases |
| src/multilang.rs | Additional binary for multilingual seeding |
| src/l2_data.rs | Additional binary for L2 data seeding |
| src/l3_topics.rs | Additional binary for L3 topic seeding |

## Key API
```rust
// main() pipeline:
// 1. For each (name, codepoint, aliases) in L0_NODES:
//    chain = encode_codepoint(cp)
//    qr    = QRSigner::sign_qr(&chain, ts)
//    writer.append_node(&chain, layer=0, is_qr=true, ts)
//    registry.insert(&chain, ...)
//    registry.register_alias(name, hash)
// 2. writer.as_bytes() → fs::write("origin.olang")
// 3. Verify roundtrip with OlangReader
```

## Rules
- No hardcoded chains — all molecular chains come from `encode_codepoint(cp)`
- No ISL hardcode or presets
- QR signatures are verified immediately after signing; failures abort the process
- File is written atomically then verified via roundtrip read
- Exits with code 1 if UCD table is empty or any node fails to write
- Each L0 node gets multilingual aliases (Vietnamese, English, French)

## Test
```bash
cargo test -p seeder
```
