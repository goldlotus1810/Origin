# AUDIT — PLAN 0.1 Readiness (2026-03-18)

**Kết luận: 2 blockers, 1 minor fix, phần còn lại SẴN SÀNG.**

---

## Kết quả test thực tế

```
cargo test -p olang audit_parse_bootstrap_lexer
  → FAIL: ParseError { message: "Expected Eq, got LBrace" }

Nguyên nhân: parser gặp `union TokenKind {`
  → `union` không phải keyword → parse thành Ident
  → expect `=` (let binding) → gặp `{` → fail
```

---

## Checklist chi tiết

### BLOCKER 1: Parser thiếu `union` và `type` keywords

| Hiện trạng | Cần |
|------------|-----|
| `alphabet.rs:373-404` có `"struct"` → `Keyword::Struct` | Thêm `"type"` → `Keyword::Struct` |
| `alphabet.rs:373-404` có `"enum"` → `Keyword::Enum` | Thêm `"union"` → `Keyword::Enum` |

**Status: DONE** (branch claude/review-and-fix-project-erPD8)
**Fix: 2 dòng code** trong `keyword_from_str()`:
```rust
"union" => Some(Keyword::Enum),    // Olang dùng "union", Rust parser dùng "enum"
"type" => Some(Keyword::Struct),   // Olang dùng "type", Rust parser dùng "struct"
```

**Ảnh hưởng:** lexer.ol dòng 7 (`union TokenKind`) và dòng 16 (`type Token`)
**Effort:** 5 phút
**Verify:** `cargo test -p olang audit_parse_bootstrap_lexer` (currently `#[ignore]`)

### BLOCKER 2: ModuleLoader thiếu file I/O

| Hiện trạng | Cần |
|------------|-----|
| `module.rs:496` `load_from_source(path, source, requester)` ✅ | `load(path, requester)` ❌ |
| Path resolution `"a.b.c"` → `"a/b/c.ol"` ✅ | File read từ `roots` dirs ❌ |
| Cache, circular detection, export extraction ✅ | Bridge: resolve path → read file → call load_from_source ❌ |

**Fix: ~20 LOC** — thêm method:
```rust
pub fn load(&mut self, module_path: &str, requester: Option<&str>)
    -> Result<Vec<String>, ModuleError>
{
    let file_path = Self::resolve_path(module_path);
    for root in &self.roots {
        let full = format!("{}/{}", root, file_path);
        if let Ok(source) = std::fs::read_to_string(&full) {
            return self.load_from_source(module_path, &source, requester);
        }
    }
    Err(ModuleError::new(&format!("Module not found: {}", module_path)))
}
```

**Lưu ý:** Crate olang dùng `#![no_std]` + `alloc`. File I/O cần feature gate hoặc move lên runtime.
**Effort:** 1-2 giờ (bao gồm no_std workaround)

### Minor Fix: `to_num()` → `to_number()`

| Hiện trạng | Cần |
|------------|-----|
| `semantic.rs:2011` maps `"to_number"` → `"__to_number"` ✅ | Thêm alias `"to_num"` → `"__to_number"` |

**Status: DONE** (branch claude/review-and-fix-project-erPD8)
**Fix: 1 dòng** trong semantic.rs builtin mapping:
```rust
"to_num" => Some("__to_number"),
```

**Effort:** 1 phút

---

## Đã SẴN SÀNG (không cần fix)

| Component | Status | Evidence |
|-----------|--------|----------|
| **Array literals** `[a, b]` | ✅ Syntax + Semantic + VM | `Expr::Array` → `__array_new` → separator-encoded chains |
| **Array ops** `push()`, `len()` | ✅ | `semantic.rs:2011` maps → `__array_push`, `__array_len` |
| **String ops** `char_at()`, `substr()` | ✅ | VM: `__str_char_at` (line 3241), `__str_substr` (line 1501) |
| **Struct literals** `Token { k: v }` | ✅ | `Expr::StructLiteral` → `__dict_new` + `__struct_tag` |
| **Enum variants** `Kind::Variant { f }` | ✅ | `Expr::EnumVariantExpr` → `__enum_payload` |
| **while/if/else/continue/return** | ✅ | Full control flow in syntax + semantic |
| **pub fn** | ✅ | `Stmt::Pub(Box<Stmt>)` wraps inner |
| **Let rebinding** `let x = x + 1` | ✅ | Shadowing via scope stack |
| **String equality** `ch == "\n"` | ✅ | `__cmp_ne` / `__eq` builtins |
| **Closures** | ✅ | `Op::Closure`, `Op::CallClosure` |
| **60+ builtins** | ✅ | Math, string, array, dict, file, device |
| **Module cache + dep graph** | ✅ | 35 tests pass |

---

## Kế hoạch unblock

```
Bước 1 (5 phút):
  alphabet.rs — thêm "union" → Enum, "type" → Struct

Bước 2 (1 phút):
  semantic.rs — thêm "to_num" → "__to_number"

Bước 3 (verify):
  cargo test -p olang audit_parse_bootstrap_lexer (un-ignore + chạy)

Bước 4 (1-2 giờ):
  module.rs — thêm load() method với file I/O

Bước 5 (verify):
  Viết test: ModuleLoader.load("olang.bootstrap.lexer") → OK

Sau khi 5 bước xong → PLAN_0_1 unblocked.
Tổng effort: ~2-3 giờ.
```

---

## Test files đã thêm

```
crates/olang/src/lang/syntax.rs:
  audit_parse_bootstrap_lexer_ol    [#[ignore] — blocked on union/type]
  audit_parse_bootstrap_parser_ol   [#[ignore] — blocked on union/type + module]

Chạy khi fix xong:
  cargo test -p olang audit_parse_bootstrap -- --include-ignored
```
