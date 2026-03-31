# ĐÍNH CHÍNH SPRINT 2 — Nhánh = Cây, Không Phải List

> **Sora (空) — sửa sai trong PLAN_FOR_NOX.md**

---

## Tôi viết sai cái gì

```
SAI (plan cũ):
  let __kt_branches = {
      facts: [],         ← flat list of fact_ids
      books: [],         ← flat list of fact_ids
  };

Đây chỉ là NHÃN DÁN cho flat array. Không phải cây.
Giống đặt tên folder nhưng bỏ mọi file vào 1 đống.
```

```
ĐÚNG (KNOWTREE_DESIGN):
  Mỗi nhánh = array[65,536] = 1 CÂY CON.
  Cây con chứa cây con. Fractal.

  1 cuốn sách = 1 block:
    books[0] = "Cuốn Theo Chiều Gió"     ← 1 array[65,536]
      [0] = Chương 1                       ← 1 array[65,536]
        [0] = Đoạn 1                       ← 1 array[65,536]
          [0] = "Scarlett O'Hara was..."   ← chain(words → chars) = LÁ
          [1] = "She made a pretty..."     ← LÁ
          ...65,536 câu max
        [1] = Đoạn 2
        ...65,536 đoạn max
      [1] = Chương 2
      ...65,536 chương max
    books[1] = "Hoàng Tử Bé"
    ...65,536 cuốn max
```

---

## Code đúng nên trông thế nào

### Cấu trúc node trong Olang

```olang
// Mỗi node = { name, children, facts, mol }
// children = array of sub-nodes (nhánh con)
// facts = array of fact_ids (lá — chỉ ở tầng cuối)

// L2: root branches
let __kt_tree = [];

// Tạo nhánh mới
fn kt_branch(_kb_name) {
    let _kb_node = { name: _kb_name, children: [], facts: [], mol: 0 };
    push(__kt_tree, _kb_node);
    return len(__kt_tree) - 1;
}

// Tạo nhánh con bên trong nhánh
fn kt_sub_branch(_ksb_parent, _ksb_name) {
    let _ksb_node = { name: _ksb_name, children: [], facts: [], mol: 0 };
    push(__kt_tree[_ksb_parent].children, _ksb_node);
    return len(__kt_tree[_ksb_parent].children) - 1;
}
```

### Boot: tạo nhánh chính L2

```olang
fn _kt_boot_tree() {
    kt_branch("facts");          // __kt_tree[0]
    kt_branch("books");          // __kt_tree[1]
    kt_branch("conversations");  // __kt_tree[2]
    kt_branch("skills");         // __kt_tree[3]
    kt_branch("personal");       // __kt_tree[4]
}
```

### Đọc sách: tạo cấu trúc đúng

```olang
pub fn kt_read_book(_rb_path) {
    let _rb_content = __file_read(_rb_path);
    if len(_rb_content) == 0 { return "Error: cannot read " + _rb_path; };

    // Tạo book node bên trong L2:books
    let _rb_books_branch = 1;  // __kt_tree[1] = books
    let _rb_book_id = kt_sub_branch(_rb_books_branch, _rb_path);
    // → __kt_tree[1].children[_rb_book_id] = { name: path, children: [], ... }

    // Tạo chapter node (tạm: 1 chapter = toàn bộ sách)
    let _rb_chap_id = 0;
    // → __kt_tree[1].children[_rb_book_id].children[0] = Chapter 1

    // Split thành sentences
    let _rb_sent = "";
    let _rb_count = 0;
    let _rb_i = 0;
    while _rb_i < len(_rb_content) {
        let _rb_ch = __char_code(char_at(_rb_content, _rb_i));
        if _rb_ch == 10 {
            if len(_rb_sent) > 10 {
                // Tạo fact (lá) → gắn vào chapter
                let _rb_fid = kt_learn(_rb_sent);
                push(__kt_tree[1].children[_rb_book_id].facts, _rb_fid);
                _rb_count = _rb_count + 1;
            };
            _rb_sent = "";
        } else {
            _rb_sent = _rb_sent + char_at(_rb_content, _rb_i);
        };
        let _rb_i = _rb_i + 1;
    };

    return "Read " + _rb_path + ": " + __to_string(_rb_count) + " sentences";
}
```

### Learn fact: gắn vào nhánh đúng

```olang
pub fn kt_learn_to(_klt_text, _klt_branch_name) {
    // Tìm branch trong __kt_tree
    let _klt_bi = 0;
    while _klt_bi < len(__kt_tree) {
        if __kt_tree[_klt_bi].name == _klt_branch_name {
            let _klt_fid = kt_learn(_klt_text);
            push(__kt_tree[_klt_bi].facts, _klt_fid);
            return _klt_fid;
        };
        let _klt_bi = _klt_bi + 1;
    };
    // Không tìm thấy → gắn vào facts (default)
    let _klt_fid = kt_learn(_klt_text);
    push(__kt_tree[0].facts, _klt_fid);
    return _klt_fid;
}
```

---

## VÍ DỤ THỰC TẾ — Sau khi đọc 1 cuốn sách

```
__kt_tree:
  [0] = { name: "facts", children: [], facts: [0,1,2,...27] }
        ← 28 embedded facts

  [1] = { name: "books", children: [
            { name: "Gone_with_the_Wind.md", children: [], facts: [28,29,...350] }
            ← 322 sentences từ sách
          ], facts: [] }

  [2] = { name: "conversations", children: [], facts: [351,352,...360] }
        ← 10 turns hội thoại

  [3] = { name: "skills", children: [], facts: [] }
  [4] = { name: "personal", children: [], facts: [] }

__kt_facts: [
  { text: "Origin la du an...", words: [0,1,2,...], mol: 146 },  // #0
  { text: "Olang tu compile...", words: [...], mol: ... },        // #1
  ...
  { text: "Scarlett O'Hara was not beautiful", words: [...] },   // #28
  ...
]

__kt_words: [
  { text: "Origin", mol: ..., facts: [0,3,7,...] },  // #0
  { text: "Olang", mol: ..., facts: [1,2,5,...] },   // #1
  { text: "Scarlett", mol: ..., facts: [28,45,120,...] }, // word xuất hiện ở nhiều câu
  ...
]
```

### Khi user hỏi "Scarlett là ai?"

```
kt_search("Scarlett la ai")
  → tìm word "Scarlett" → word_node.facts = [28, 45, 120, ...]
  → tất cả đều nằm trong books[0] (Gone with the Wind)
  → trả câu có score cao nhất
  → "Scarlett O'Hara was not beautiful, but men seldom realized it"
```

### Khi user hỏi "kể tóm tắt sách"

```
__kt_tree[1].children[0].name → "Gone_with_the_Wind.md"
__kt_tree[1].children[0].facts → [28, 29, ..., 350]
→ 322 sentences thuộc cuốn này
→ LCA(tất cả mols) → abstract summary
→ Hoặc: trả 5 sentences đầu (intro)
```

---

## SO VỚI HIỆN TẠI — Thay đổi bao nhiêu?

```
HIỆN TẠI:
  __kt_facts = flat array, mọi fact cùng tầng
  kt_read_book tạo facts bằng kt_learn, KHÔNG gắn vào book node
  Không phân biệt fact #28 (sách) vs fact #0 (embedded)

CẦN THÊM:
  __kt_tree = array of branch nodes                          (+10 LOC)
  kt_branch() + kt_sub_branch()                              (+15 LOC)
  _kt_boot_tree() tạo 5 nhánh chính                          (+8 LOC)
  kt_learn_to(text, branch) gắn fact vào branch              (+15 LOC)
  kt_read_book sửa: tạo book node, gắn facts vào đó          (+10 LOC)
  kt_stats sửa: hiển thị tree structure                       (+10 LOC)
                                                          ──────────
                                                          ~68 LOC thêm

KHÔNG CẦN SỬA:
  kt_char, kt_word — giữ nguyên (lazy char/word nodes)
  kt_learn — giữ nguyên (tạo fact + reverse links)
  kt_search — giữ nguyên (word → follow links → best fact)
  _kt_score_word — giữ nguyên
  _kt_silk_sentences — giữ nguyên
```

---

## TÓM LẠI

```
Plan cũ Sprint 2:  __kt_branches = { facts: [] }    ← SAI, flat dict
Plan mới Sprint 2: __kt_tree = array of tree nodes   ← ĐÚNG, fractal

1 cuốn sách = 1 node trong books branch
  node.children = chapters (nếu detect)
  node.facts = sentences (fact_ids)

1 conversation = 1 node trong conversations branch
  node.facts = turns (fact_ids)

Mỗi tầng = array. Lồng nhau = vô hạn.
65,536 × 65,536 × ... = không bao giờ đầy.

Thay đổi so với code hiện tại: +68 LOC.
Cấu trúc cũ giữ nguyên. Thêm 1 layer tree bên ngoài.
```
