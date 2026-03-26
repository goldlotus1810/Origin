# inspector

> CLI tool that reads and verifies `origin.olang` files — the append-only ledger of HomeOS.

## Dependencies
- ucd
- olang (with `std` feature)

## Files
| File | Purpose |
|------|---------|
| src/main.rs | Reads `origin.olang`, parses all records via `OlangReader`, displays contents and runs integrity checks |

## Key API
```rust
// main() pipeline:
// 1. Read file bytes: fs::read(path)
// 2. Parse: OlangReader::new(&data) → reader
// 3. Parse all records: reader.parse_all() → ParsedData
// 4. Display:
//    - Node/Edge/Alias/QR counts
//    - Layer distribution (L0..L7)
//    - Sample nodes (first 10) with hash, molecule count, timestamp
//    - Sample aliases (first 10, excluding _qr_ internal)
// 5. Verify:
//    - All nodes have non-empty chains
//    - File offsets monotonically increasing (append-only invariant)
```

## Usage
```bash
# Inspect default origin.olang in current directory
cargo run -p inspector

# Inspect specific file
cargo run -p inspector -- path/to/origin.olang
```

## Output Example
```
Inspector ○  · origin.olang
File size: 4567 bytes
Created: 1000000 ns

── Contents ──────────────────────────────────
Nodes  : 34
Edges  : 12
Aliases: 102
QR     : 34

── Layer Distribution ───────────────────────
  L0: 34 nodes

── Verify ────────────────────────────────────
✓ All 34 nodes have non-empty chains
✓ File offsets monotonically increasing (append-only)
```

## Rules
- Read-only — never modifies the file
- Verifies append-only invariant (QT10): offsets must be monotonically increasing
- Displays QR vs non-QR (ĐN) distinction for each node

## Test
```bash
cargo test -p inspector
```
