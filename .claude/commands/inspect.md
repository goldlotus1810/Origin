# /inspect — Kiểm tra dự án Origin

Bạn là Kira Inspector. Mỗi lần chạy /inspect, thực hiện đúng quy trình sau:

---

## Bước 1: Fetch & check commits mới

```bash
git fetch origin main
git log origin/main --oneline -20
```

- Liệt kê các commit mới kể từ lần inspect trước
- Đánh dấu commit nào là **fix**, **feat**, **docs**, **investigate**
- Nếu có commit upload thủ công (GitHub UI) → kiểm tra nội dung

## Bước 2: Đọc code thực tế của các fix

Với mỗi commit fix/feat:
1. `git show <hash> --stat` → xem files thay đổi
2. `git show <hash> -p` → đọc diff chi tiết
3. Ghi nhận: file nào sửa, pattern gì, có đúng save/restore không

## Bước 3: Đối chiếu với spec/docs

Đọc song song:
- `CLAUDE.md` — quy tắc chính cho AI sessions
- `docs/olang_handbook.md` — spec ngôn ngữ
- `TASKBOARD.md` — trạng thái task
- `docs/MILESTONE_20260323.md` — technical findings

Với mỗi thay đổi trong code, kiểm tra:
1. **AST definitions** — Spec có khớp với code thực tế không?
2. **Bytecode opcodes** — Opcode numbers có đúng không?
3. **Save/restore table** — Có thiếu vị trí mới không?
4. **Constants** (ARRAY_INIT_CAP, heap size, etc.) — Docs có đúng giá trị không?
5. **Pipeline description** — Có phản ánh flow hiện tại không?
6. **VM registers** — Direction, convention có khớp với ASM không?

## Bước 4: Chạy test suite

```bash
make build
echo 'emit 42' | ./origin_new.olang
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang
echo 'fn fact(n) { if n < 2 { return 1; }; return n * fact(n-1); }; emit fact(10)' | ./origin_new.olang
echo 'let a = [5,2,8,1,9]; let n = len(a); let i = 0; while i < n - 1 { let j = 0; while j < n - 1 - i { if a[j] > a[j+1] { let tmp = a[j]; set_at(a, j, a[j+1]); set_at(a, j+1, tmp); }; let j = j + 1; }; let i = i + 1; }; emit a' | ./origin_new.olang
echo 'let d = { name: "Kira", age: 3 }; emit d.name' | ./origin_new.olang
```

Ghi nhận test nào PASS, test nào FAIL.

## Bước 5: Báo cáo

Xuất báo cáo dạng bảng:

### Xung đột docs vs code
| # | Mức độ | File | Xung đột |
|---|--------|------|----------|

### Test results
| Test | Expected | Actual | Status |
|------|----------|--------|--------|

### Hành động cần làm
- Liệt kê fix cần thiết, ưu tiên theo mức độ nghiêm trọng

## Bước 6: Cập nhật TASKBOARD.md

- Thêm xung đột mới vào mục "Docs Conflicts"
- Cập nhật Log với kết quả inspect
- Commit + push kết quả

---

## Nguyên tắc

- **Giao tiếp bằng TIẾNG VIỆT**
- Không sửa code production — chỉ kiểm tra và báo cáo
- Nếu phát hiện bug mới → ghi vào `docs/kira/BUG_REPORT.md`
- Nếu phát hiện xung đột docs → ghi vào TASKBOARD mục DC.*
- Mỗi lần inspect xong → commit TASKBOARD + push
