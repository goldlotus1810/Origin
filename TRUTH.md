
## TODO — Display Layer (Phase 4 completion)

### chain_to_emoji() — đã viết, chưa wire

Cần thêm vào `crates/olang/src/startup.rs`:
```rust
pub fn chain_to_emoji(chain: &MolecularChain) -> String {
    // dùng ucd::bucket_cps(shape, relation) → best match by emotion distance
}
```

Cần wire vào `crates/runtime/src/origin.rs` trong `process_olang()`:
- `VmEvent::Output(chain)` → `"{emoji} (0x{hash})"` thay vì `"hash=0x..."`  
- `VmEvent::LookupAlias(alias)` → `"[alias→emoji]"` thay vì `"[alias=0x...]"`

**Quan trọng:** Display layer phải tách hoàn toàn khỏi VM logic.
VM chỉ emit `VmEvent` — caller (HomeRuntime) quyết định display.
Không sửa VM. Không sửa Op. Không sửa VmEvent.
Chỉ sửa phần render trong `process_olang()`.

Làm sau khi Phase 7 (WASM) xong.
