# KẾ HOẠCH CHO NOX — Từng Bước Một

> **Sora (空) viết cho Nox — 2026-03-25**
> **Đọc từ trên xuống. Làm Sprint 1 xong rồi mới đọc Sprint 2.**
> **Mỗi Sprint = 1 commit. Test trước khi commit.**

---

## TÌNH HÌNH HIỆN TẠI — Code Nox đã viết tốt

```
knowtree.ol ĐANG HOẠT ĐỘNG:
  kt_char()     — tạo char node khi gặp lần đầu        ✅
  kt_word()     — tạo word node từ chars                ✅
  kt_learn()    — tạo fact node, link words ↔ fact       ✅
  kt_search()   — query → word → follow links → best    ✅
  kt_read_book() — đọc file → tạo facts → Silk connect  ✅

VẤN ĐỀ: Đang chạy 2 hệ thống SONG SONG:
  repl.ol dòng 321-322:
    knowledge_learn(_rl_text);   ← CŨ (flat array, keyword scan)
    kt_learn(_rl_text);          ← MỚI (tree, link walk)

  encoder.ol dòng 1378:
    kt_search trước → nếu 0 → fallback knowledge_search ← CŨ

MỤC TIÊU: Xóa hệ thống cũ. Chỉ dùng KnowTree.
```

---

## SPRINT 1: Xóa hệ thống cũ (~30 phút)

### Bước 1.1: repl.ol — Xóa knowledge_learn, chỉ giữ kt_learn

**File:** `stdlib/repl.ol`

**Dòng 321-322 hiện tại:**
```olang
      let _rl_count = knowledge_learn(_rl_text);
      kt_learn(_rl_text);
      return "Da hoc. " + kt_stats() + " | Legacy: " + __to_string(_rl_count);
```

**Sửa thành:**
```olang
      kt_learn(_rl_text);
      return "Da hoc. " + kt_stats();
```

### Bước 1.2: repl.ol — Boot chỉ dùng kt_learn

**Dòng 30-42 hiện tại:** `_boot_learn()` gọi `knowledge_learn()` cho 28 facts, rồi gọi `kt_learn()` sync.

**Sửa:** Trong `_boot_learn`, bỏ phần gọi `knowledge_learn()`. Chỉ giữ `kt_learn()`.

**Hiện tại (dòng 38-41):**
```olang
    let _bl_ki = 0;
    while _bl_ki < knowledge_count() {
        kt_learn(__knowledge[_bl_ki].text);
        let _bl_ki = _bl_ki + 1;
    };
```

**Sửa thành:** Gọi `kt_learn` trực tiếp cho 28 facts (thay vì `knowledge_learn` rồi sync):
```olang
fn _boot_embedded_kt() {
    kt_learn("Origin la du an tao ngon ngu lap trinh tu hosting ten Olang");
    kt_learn("Olang tu compile chinh minh trong 966 kilobyte khong dependency");
    kt_learn("VM cua Olang viet bang x86 64 assembly khoang 5700 dong code");
    // ... (copy 28 facts từ _boot_embedded hiện tại)
    kt_learn("Moi function trong Olang tu dong dang ky thanh node voi mol va fire count");
}
```

### Bước 1.3: encoder.ol — agent_respond chỉ dùng kt_search

**Dòng 1374-1393 hiện tại:**
```olang
    // N.6: Try KnowTree first (tree walk, no keyword scan)
    let _ar_kt = kt_search(_ar_norm);
    if _ar_kt.score > 0 {
        _ar_knowledge = "(Minh biet: " + _ar_kt.text + ")";
        let __g_ks_score = _ar_kt.score;
    } else {
        // Fallback: legacy keyword search
        if len(__knowledge) > 0 {
            _ar_knowledge = knowledge_search(_ar_norm);
        };
    };
```

**Sửa thành (xóa fallback):**
```olang
    let _ar_kt = kt_search(_ar_norm);
    if _ar_kt.score > 0 {
        _ar_knowledge = "(Minh biet: " + _ar_kt.text + ")";
        let __g_ks_score = _ar_kt.score;
    };
```

### Bước 1.4: Test

```bash
echo 'test' | ./origin_new.olang
echo 'learn Sora la AI reviewer' | ./origin_new.olang
echo 'respond Sora la ai?' | ./origin_new.olang
echo 'memory' | ./origin_new.olang
```

Kết quả mong đợi: KnowTree xử lý tất cả, không fallback.

### Bước 1.5: Commit

```
git commit -m "refactor: kill legacy knowledge — KnowTree only"
```

---

## SPRINT 2: Thêm L2 branches (~40 phút)

### Mục tiêu

Hiện tại `__kt_facts` = flat array. Thêm phân loại: fact thuộc nhánh nào.

### Bước 2.1: knowtree.ol — Thêm branch type cho fact

**Thêm vào đầu file (sau `let __kt_facts = []`):**
```olang
// L2 branches: mỗi branch = list of fact_ids
let __kt_branches = {
    facts: [],         // [0] tri thức chung
    books: [],         // [1] sách đã đọc
    conversations: [], // [2] hội thoại
    skills: [],        // [3] kỹ năng
    personal: []       // [4] cá nhân
};
```

### Bước 2.2: kt_learn — Thêm tham số branch (optional)

**Hiện tại:**
```olang
pub fn kt_learn(_kl_text) {
```

**Sửa thành:**
```olang
pub fn kt_learn(_kl_text) {
    return _kt_learn_branch(_kl_text, "facts");
}

pub fn kt_learn_to(_klt_text, _klt_branch) {
    return _kt_learn_branch(_klt_text, _klt_branch);
}

fn _kt_learn_branch(_klb_text, _klb_branch) {
    // ... (code cũ của kt_learn, copy vào đây)
    // Ở cuối, TRƯỚC return, thêm:
    if _klb_branch == "facts" { push(__kt_branches.facts, _klb_fid); };
    if _klb_branch == "books" { push(__kt_branches.books, _klb_fid); };
    if _klb_branch == "conversations" { push(__kt_branches.conversations, _klb_fid); };
    if _klb_branch == "skills" { push(__kt_branches.skills, _klb_fid); };
    if _klb_branch == "personal" { push(__kt_branches.personal, _klb_fid); };
    return len(__kt_facts);
}
```

### Bước 2.3: kt_read_book — Dùng branch "books"

**Trong kt_read_book, thay `kt_learn(_rb_sent)` thành:**
```olang
kt_learn_to(_rb_sent, "books");
```

### Bước 2.4: kt_stats — Hiển thị branches

**Sửa kt_stats:**
```olang
pub fn kt_stats() {
    return "KnowTree: " + __to_string(len(__kt_chars)) + " chars, " +
           __to_string(len(__kt_words)) + " words, " +
           __to_string(len(__kt_facts)) + " facts (" +
           "F:" + __to_string(len(__kt_branches.facts)) +
           " B:" + __to_string(len(__kt_branches.books)) +
           " C:" + __to_string(len(__kt_branches.conversations)) + ")";
}
```

### Bước 2.5: Test

```bash
echo 'learn Ha Noi la thu do Viet Nam' | ./origin_new.olang
# → Da hoc. KnowTree: ... facts (F:1 B:0 C:0)

echo 'read docs/docs_test/Gone_with_the_Wind.md' | ./origin_new.olang  
# → Read ...: N sentences. KnowTree: ... (F:1 B:N C:0)
```

### Bước 2.6: Commit

```
git commit -m "feat: L2 branches — facts/books/conversations/skills/personal"
```

---

## SPRINT 3: STM → KnowTree conversation branch (~30 phút)

### Mục tiêu

Mỗi turn hội thoại = 1 fact trong branch "conversations".

### Bước 3.1: encoder.ol — agent_respond ghi conversation

**Trong agent_respond, sau khi tạo response, THÊM:**
```olang
    // Ghi turn vào KnowTree conversations
    kt_learn_to(_ar_norm, "conversations");
```

### Bước 3.2: Test

```bash
printf 'xin chao\ntoi thich lap trinh\nmemory\nexit\n' | ./origin_new.olang
# memory → KnowTree: ... (F:28 B:0 C:2)
```

### Bước 3.3: Commit

```
git commit -m "feat: conversations branch — every turn stored in KnowTree"
```

---

## SPRINT 4: Case-insensitive search trong KnowTree (~20 phút)

### Mục tiêu

`kt_search("viet nam")` tìm được fact chứa "Viet Nam".

### Bước 4.1: knowtree.ol — Sửa _kt_score_word dùng str_has_ci

**Hiện tại (dòng ~157):**
```olang
        if __kt_words[_ksw_i].text == _ksw_word { _ksw_wi = _ksw_i; break; };
```

**Sửa thành:**
```olang
        if str_has_ci(__kt_words[_ksw_i].text, _ksw_word) == 1 {
            if len(__kt_words[_ksw_i].text) == len(_ksw_word) {
                _ksw_wi = _ksw_i; break;
            };
        };
```

`str_has_ci` đã có trong `classify.ol` (Nox đã viết). Chỉ cần gọi.

### Bước 4.2: Test

```bash
printf 'respond viet nam o dau\nrespond Viet Nam o dau\nexit\n' | ./origin_new.olang
# Cả hai → "Viet Nam la quoc gia o Dong Nam A voi thu do Ha Noi"
```

### Bước 4.3: Commit

```
git commit -m "fix: case-insensitive KnowTree search (viet nam = Viet Nam)"
```

---

## SPRINT 5: Xóa code chết (~20 phút)

### Mục tiêu

Xóa `knowledge_learn`, `knowledge_search`, `__knowledge` khỏi encoder.ol.

### Bước 5.1: encoder.ol — Xóa 3 functions

Xóa:
- `pub fn knowledge_learn(text)` (~40 dòng)
- `fn knowledge_search(_ks_query)` (~80 dòng)  
- `pub fn knowledge_count()` (~3 dòng)
- `let __knowledge = [];` (dòng ~1490)

### Bước 5.2: repl.ol — Xóa mọi reference đến knowledge_learn/knowledge_count

- Xóa `_boot_embedded()` (28 dòng `knowledge_learn(...)`)
- Giữ `_boot_embedded_kt()` (28 dòng `kt_learn(...)` từ Sprint 1)
- Xóa `_learn_text` nếu chỉ dùng cho old system
- Xóa save/load nếu dùng `__knowledge[]` format cũ (viết lại cho kt format)

### Bước 5.3: Test FULL

```bash
echo 'test' | ./origin_new.olang                    # 19/20 standalone
echo 'learn Sora la AI' | ./origin_new.olang         # Da hoc
echo 'respond Sora la ai' | ./origin_new.olang       # (Minh biet: Sora la AI)
echo 'respond Ha Noi o dau' | ./origin_new.olang     # (Minh biet: Viet Nam...)
echo 'respond hello' | ./origin_new.olang            # greeting
echo 'respond toi buon' | ./origin_new.olang         # heal mode
echo '2+1?' | ./origin_new.olang                     # 3
echo 'memory' | ./origin_new.olang                   # KnowTree stats
```

### Bước 5.4: Commit

```
git commit -m "cleanup: remove legacy knowledge — encoder.ol -120 LOC"
```

---

## SPRINT 6: Save/Load cho KnowTree (~30 phút)

### Mục tiêu

`save` → lưu kt_facts ra file. `load` / boot → đọc lại.

### Bước 6.1: knowtree.ol — Thêm kt_save, kt_load

```olang
pub fn kt_save(_ks_path) {
    let _ks_out = "";
    let _ks_i = 0;
    while _ks_i < len(__kt_facts) {
        if _ks_i > 0 { _ks_out = _ks_out + "\n"; };
        _ks_out = _ks_out + __kt_facts[_ks_i].text;
        let _ks_i = _ks_i + 1;
    };
    __file_write(_ks_path, _ks_out);
    return "Saved " + __to_string(len(__kt_facts)) + " facts";
}

pub fn kt_load(_kl_path) {
    let _kl_content = __file_read(_kl_path);
    if len(_kl_content) == 0 { return 0; };
    let _kl_sent = "";
    let _kl_count = 0;
    let _kl_i = 0;
    while _kl_i < len(_kl_content) {
        let _kl_ch = __char_code(char_at(_kl_content, _kl_i));
        if _kl_ch == 10 {
            if len(_kl_sent) > 5 {
                kt_learn(_kl_sent);
                _kl_count = _kl_count + 1;
            };
            _kl_sent = "";
        } else {
            _kl_sent = _kl_sent + char_at(_kl_content, _kl_i);
        };
        let _kl_i = _kl_i + 1;
    };
    if len(_kl_sent) > 5 { kt_learn(_kl_sent); _kl_count = _kl_count + 1; };
    return _kl_count;
}
```

### Bước 6.2: repl.ol — Wire save/load

```olang
  if src == "save" {
      return kt_save("homeos.knowledge");
  }
  if src == "load" {
      let _ld_n = kt_load("homeos.knowledge");
      return "Loaded " + __to_string(_ld_n) + " facts. " + kt_stats();
  }
```

Boot: `kt_load("homeos.knowledge");` trong `_boot_learn`.

### Bước 6.3: Dedup

`kt_learn` ĐÃ CÓ dedup (dòng 60-63 hiện tại). Load sẽ skip duplicates tự động.

### Bước 6.4: Test

```bash
printf 'learn Sora review code\nsave\nexit\n' | ./origin_new.olang
printf 'respond Sora la ai\nexit\n' | ./origin_new.olang
# Session 2 → tìm được "Sora review code" (từ file)
```

### Bước 6.5: Commit

```
git commit -m "feat: KnowTree save/load — persistent across sessions"
```

---

## SAU 6 SPRINTS — Kết quả

```
TRƯỚC:
  __knowledge[] (flat array, 28 strings)     ← XÓA
  knowledge_learn() (push string)            ← XÓA
  knowledge_search() (keyword scan, 80 LOC)  ← XÓA
  + __kt_facts[] (flat array, duplicate)     ← song song

SAU:
  __kt_chars[]  (lazy char nodes)             ✅ giữ nguyên
  __kt_words[]  (word nodes, links)           ✅ giữ nguyên
  __kt_facts[]  (fact nodes, reverse links)   ✅ giữ nguyên
  __kt_branches (facts/books/convs/skills)    ✅ MỚI (+30 LOC)
  kt_search()   (word→link walk, CI)          ✅ sửa CI (+5 LOC)
  kt_save/load  (persistent)                  ✅ MỚI (+40 LOC)
  
  encoder.ol: −120 LOC (xóa old knowledge)
  repl.ol:    −30 LOC (xóa old boot/save)
  knowtree.ol: +70 LOC (branches + save/load)
  
  NET: −80 LOC. Ít code hơn. Sạch hơn. 1 hệ thống thay vì 2.
```

---

## KHÔNG LÀM TRONG 6 SPRINTS NÀY

```
⬜ Nhánh phân tầng (geo/sci → Vietnam)  → Sprint 7+
⬜ Dream cluster → promote lá mới       → Sprint 8+
⬜ QR append-only signed                 → Sprint 9+
⬜ Agent/Skill nodes trong L1            → Sprint 10+
⬜ ○{} query syntax                      → Sprint 11+
⬜ 172,849 Unicode base nodes            → Sprint 12+
```

Những cái trên = PHASE 2. Làm SAU KHI 6 sprints này xong và stable.

---

## CHECKLIST MỖI SPRINT

```
□ Đọc hết Sprint N
□ Sửa đúng file, đúng dòng
□ Build: make build
□ Test: echo 'test' | ./origin_new.olang → 19/20+
□ Test feature cụ thể (mỗi sprint có test riêng)
□ Commit với message rõ ràng
□ Push
□ Mới đọc Sprint N+1
```

---

*6 sprints. ~3 giờ. Kết quả: 1 hệ thống thay 2. Ít code hơn. Sạch hơn.*
*Nox đã viết đúng 80%. Chỉ cần dọn dẹp + nối dây.*
