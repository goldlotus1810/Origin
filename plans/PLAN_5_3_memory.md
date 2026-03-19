# PLAN 5.3 — Memory Optimization

**Phụ thuộc:** Phase 3 DONE
**Mục tiêu:** Arena allocator, zero-copy strings, Molecule pool → giảm memory + GC pressure
**Tham chiếu:** `vm/x86_64/vm_x86_64.S` (r15 bump allocator)

---

## Bối cảnh

```
HIỆN TẠI:
  r15 = bump allocator, KHÔNG BAO GIỜ free
  Mỗi conversation turn: allocate strings, chains → r15 tăng mãi
  16 MB heap → hết sau ~10,000 turns (ước tính)

SAU PLAN 5.3:
  Arena per-turn: allocate tự do trong turn, FREE HẾT cuối turn
  Zero-copy: string slice thay vì copy
  Molecule pool: reuse 5-byte slots thay vì allocate mới
  → Memory stable, không tăng vô hạn
```

---

## Tasks

### 5.3.1 — Arena allocator (~100 LOC ASM)

```asm
;; Arena = region of memory, reset bằng 1 pointer move

arena_state:
    .quad 0     ;; arena_base (start of current arena)
    .quad 0     ;; arena_ptr  (current allocation point)
    .quad 0     ;; arena_end  (end of arena)

arena_init:
    ;; mmap 1 MB for arena
    mov     $9, %rax                ;; sys_mmap
    xor     %rdi, %rdi              ;; addr = NULL
    mov     $0x100000, %rsi         ;; 1 MB
    mov     $3, %rdx                ;; PROT_READ|PROT_WRITE
    mov     $0x22, %r10             ;; MAP_PRIVATE|MAP_ANONYMOUS
    mov     $-1, %r8                ;; fd = -1
    xor     %r9, %r9                ;; offset = 0
    syscall
    ;; Save base + ptr + end
    lea     arena_state(%rip), %rcx
    mov     %rax, (%rcx)            ;; arena_base
    mov     %rax, 8(%rcx)           ;; arena_ptr = base
    add     $0x100000, %rax
    mov     %rax, 16(%rcx)          ;; arena_end
    ret

arena_alloc:
    ;; rdi = size → rax = pointer
    lea     arena_state(%rip), %rcx
    mov     8(%rcx), %rax           ;; arena_ptr
    add     %rdi, %rax
    cmp     16(%rcx), %rax          ;; > arena_end?
    ja      arena_oom
    mov     %rax, 8(%rcx)           ;; advance ptr
    sub     %rdi, %rax              ;; return start
    ret

arena_reset:
    ;; Reset = 1 instruction! O(1)
    lea     arena_state(%rip), %rcx
    mov     (%rcx), %rax            ;; arena_ptr = arena_base
    mov     %rax, 8(%rcx)
    ret
```

**Tích hợp vào VM:**
```
Mỗi process_text() call:
  1. arena_init() (lần đầu) hoặc arena_reset() (subsequent)
  2. VM chạy → tất cả alloc dùng arena_alloc()
  3. Kết quả cần giữ (STM, QR) → copy sang persistent heap (r15)
  4. arena_reset() → free hết temporary data
```

### 5.3.2 — Zero-copy strings (~50 LOC)

```
HIỆN TẠI:
  substr(s, start, len) → allocate + memcpy = O(n)

SAU:
  substr(s, start, len) → (s.ptr + start, len) = O(1), no allocation

Thay đổi cần thiết:
  - Stack entry [ptr, len] ĐÃ hỗ trợ (ptr = interior pointer)
  - NHƯNG: cần đảm bảo source string sống đủ lâu
  - Arena model: tất cả string trong cùng arena → ok vì reset cùng lúc
  - Cross-arena reference: copy khi promote sang persistent heap
```

### 5.3.3 — Molecule pool (~80 LOC)

```
Molecule = 5 bytes. Allocation overhead > data size nếu malloc mỗi cái.

Pool:
  - Pre-allocate slab: 4096 Molecule slots = 4096 × 8 bytes = 32 KB
    (8 bytes per slot: 5 bytes Molecule + 3 bytes padding/metadata)
  - Free list: linked list of available slots
  - Alloc: pop free list → O(1)
  - Free: push free list → O(1)
  - Grow: mmap thêm slab khi hết

Integrate:
  - encode_codepoint() → pool_alloc() thay vì heap_alloc()
  - LCA result → pool_alloc()
  - evolve() result → pool_alloc()
```

### 5.3.4 — Persistent heap compaction (optional)

```
r15 (persistent heap) vẫn bump-only cho simplicity.
Compaction CHỈ chạy khi:
  - heap_used > 80% capacity
  - During Dream cycle (offline, không block user)

Compaction strategy:
  - Mark: walk Registry + STM + Silk → mark reachable
  - Sweep: compact live data → reset r15
  - Update: fix pointers in Registry + Silk
  - Ước tính: ~200 LOC ASM, phức tạp → defer sang sau
```

---

## Rào cản

```
1. Zero-copy + arena = dangling pointer risk
   → Giải pháp: strict lifetime rules
   → Within turn: all pointers valid (arena alive)
   → Cross turn: MUST copy to persistent heap
   → VM enforce: arena_reset() only at turn boundary

2. Persistent data (STM, QR) needs survive arena_reset()
   → Giải pháp: explicit promote() function
   → promote(arena_ptr, len) → copy to r15 heap → return new ptr
   → STM.push() calls promote() internally

3. Pool fragmentation
   → Fixed-size slots → NO fragmentation (Molecule = always 5 bytes)
```

---

## Test Plan

```
Test 1: Arena — alloc 10000 × 100 bytes → reset → alloc again → no OOM
Test 2: Zero-copy substr — verify content correct, no extra allocation
Test 3: Molecule pool — alloc/free 1000 molecules → pool size stable
Test 4: Turn simulation — 1000 turns × 100 allocs each → memory stable
Test 5: Promote — arena data → persistent heap → survives arena_reset
```

---

## Definition of Done

- [ ] Arena allocator (mmap + bump + reset)
- [ ] VM integration: arena per turn
- [ ] Zero-copy substr
- [ ] Molecule pool (slab allocator)
- [ ] promote() for persistent data
- [ ] Test: memory stable over 1000 turns

## Ước tính: 3-5 ngày
