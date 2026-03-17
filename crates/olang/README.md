# olang

> Core types and operations: Molecule (5 bytes), MolecularChain, LCA, Registry, VM, and Compiler for the HomeOS knowledge graph.

## Dependencies
- ucd
- ed25519-dalek
- sha2

## Files
| File | Purpose |
|------|---------|
| molecular.rs | Molecule (5D coordinate), MolecularChain, similarity, FNV-1a chain_hash |
| encoder.rs | encode_codepoint(cp), encode_zwj_sequence, encode_flag -- the ONLY way to create chains |
| lca.rs | Weighted LCA engine: lca(), lca_weighted(), lca_many() with mode detection (>=60%) or weighted avg |
| registry.rs | Append-only ledger: insert, lookup_hash, lookup_name, register_alias, layer_rep, QR supersession |
| writer.rs | Binary writer for origin.olang (append-only) |
| reader.rs | Binary reader for origin.olang |
| qr.rs | QR (proven) node operations with ED25519 signing |
| ir.rs | Intermediate representation for the VM |
| vm.rs | Virtual machine executing IR |
| compiler.rs | Compiler from source to IR |
| log.rs | Append-only event log |
| startup.rs | Rebuild registry from origin.olang at startup |
| ling.rs | Linguistic modifiers (negation, amplifier, diminisher, contrast) |
| clone.rs | Node cloning operations |
| self_model.rs | Self-model introspection |
| separator.rs | Separator utilities |

## Key API
```rust
pub fn encode_codepoint(cp: u32) -> MolecularChain
pub fn encode_zwj_sequence(codepoints: &[u32]) -> MolecularChain
pub fn lca(a: &MolecularChain, b: &MolecularChain) -> MolecularChain
pub fn lca_weighted(pairs: &[(&MolecularChain, u32)]) -> MolecularChain
pub fn Registry::insert(&mut self, chain: &MolecularChain, layer: u8, file_offset: u64, created_at: i64, is_qr: bool) -> u64
```

## Rules
- 4: All Molecules from encode_codepoint(cp) -- NEVER hand-write.
- 5: All chains from LCA or UCD -- NEVER hand-write.
- 6: chain_hash is auto-generated. NEVER hand-write.
- 7: Parent chain = LCA(child chains).
- 8: Every Node created -> auto registry.
- 9: Write file FIRST -- update RAM AFTER.
- 10: Append-only -- NO DELETE, NO OVERWRITE.

## Test
```bash
cargo test -p olang
```
