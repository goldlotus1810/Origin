# ucd

> Compile-time Unicode Character Database lookup: codepoint to Molecule bytes via static tables generated from UnicodeData.txt.

## Dependencies
- None (no_std, no external crates)

## Files
| File | Purpose |
|------|---------|
| build.rs | Reads UnicodeData.txt at compile time, derives 5D Molecule bytes (shape, relation, valence, arousal, time) for ~5400 codepoints across 4 Unicode groups (SDF, MATH, EMOTICON, MUSICAL), and generates static lookup tables |
| src/lib.rs | Public API for forward lookup (cp to UcdEntry), reverse lookup (hash to cp), bucket lookup (shape+relation to codepoints), and convenience accessors for individual dimensions |

## Key API
```rust
pub fn lookup(cp: u32) -> Option<&'static UcdEntry>
pub fn decode_hash(hash: u64) -> Option<u32>
pub fn bucket_cps(shape: u8, relation: u8) -> &'static [u32]
pub fn shape_of(cp: u32) -> u8
pub fn is_sdf_primitive(cp: u32) -> bool
```

## Rules
- 1: 5 Unicode groups = foundation. Do not add groups.
- 2: Unicode character name = node name. Do not rename.
- 4: All Molecules from encode_codepoint(cp) -- never hand-write.

## Test
```bash
cargo test -p ucd
```
