// stdlib/jit_compile.ol — Auto-JIT: compile Olang function → native x86-64
// Uses __mmap_exec, __memcpy_to, __call_native
//
// Usage:
//   let native_fib = jit_fib();           // compile fib → native
//   let result = __call_native(native_fib, 30);  // run at C speed
//
// Currently supports: recursive integer fib pattern
// Future: general function JIT via asm_emit.ol

// ── JIT compile fib(n) → native x86-64 ──
// Hardcoded pattern: if n < 2 return n; return fib(n-1) + fib(n-2)
// Output: function pointer to native code

pub fn jit_fib() {
    let code = [];
    // cmp rdi, 2
    let _ = __push(code, 0x48); let _ = __push(code, 0x83);
    let _ = __push(code, 0xFF); let _ = __push(code, 0x02);
    // jge +4 (skip base case)
    let _ = __push(code, 0x7D); let _ = __push(code, 0x04);
    // mov rax, rdi (base: return n)
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xF8);
    // ret
    let _ = __push(code, 0xC3);
    // .recurse: push rbx; push rbp
    let _ = __push(code, 0x53); let _ = __push(code, 0x55);
    // mov rbx, rdi
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xFB);
    // lea rdi, [rbx-1]
    let _ = __push(code, 0x48); let _ = __push(code, 0x8D);
    let _ = __push(code, 0x7B); let _ = __push(code, 0xFF);
    // call self (offset = 0 - (19+5) = -24 = 0xE8)
    let _ = __push(code, 0xE8);
    let _ = __push(code, 0xE8); let _ = __push(code, 0xFF);
    let _ = __push(code, 0xFF); let _ = __push(code, 0xFF);
    // mov rbp, rax
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xC5);
    // lea rdi, [rbx-2]
    let _ = __push(code, 0x48); let _ = __push(code, 0x8D);
    let _ = __push(code, 0x7B); let _ = __push(code, 0xFE);
    // call self (offset = 0 - (31+5) = -36 = 0xDC)
    let _ = __push(code, 0xE8);
    let _ = __push(code, 0xDC); let _ = __push(code, 0xFF);
    let _ = __push(code, 0xFF); let _ = __push(code, 0xFF);
    // add rax, rbp
    let _ = __push(code, 0x48); let _ = __push(code, 0x01); let _ = __push(code, 0xE8);
    // pop rbp; pop rbx; ret
    let _ = __push(code, 0x5D); let _ = __push(code, 0x5B); let _ = __push(code, 0xC3);

    // Allocate + copy
    let mem = __mmap_exec(4096);
    let buf = __bytes_new(__array_len(code));
    let i = 0;
    while i < __array_len(code) {
        __bytes_set(buf, i, __array_get(code, i));
        let i = i + 1;
    };
    __memcpy_to(mem, buf, __array_len(code));
    return mem;
}

// ── JIT compile iterative sum(n) → native ──
// sum(rdi) = 0+1+2+...+n = n*(n+1)/2
// But iterative loop version for benchmark:

pub fn jit_sum() {
    let code = [];
    // xor rax, rax (sum = 0)
    let _ = __push(code, 0x48); let _ = __push(code, 0x31); let _ = __push(code, 0xC0);
    // xor rcx, rcx (i = 0)
    let _ = __push(code, 0x48); let _ = __push(code, 0x31); let _ = __push(code, 0xC9);
    // .loop: cmp rcx, rdi
    let _ = __push(code, 0x48); let _ = __push(code, 0x39); let _ = __push(code, 0xF9);
    // jge .done (+6)
    let _ = __push(code, 0x7D); let _ = __push(code, 0x06);
    // add rax, rcx
    let _ = __push(code, 0x48); let _ = __push(code, 0x01); let _ = __push(code, 0xC8);
    // inc rcx
    let _ = __push(code, 0x48); let _ = __push(code, 0xFF); let _ = __push(code, 0xC1);
    // jmp .loop (-12 = 0xF4)
    let _ = __push(code, 0xEB); let _ = __push(code, 0xF4);
    // .done: ret
    let _ = __push(code, 0xC3);

    let mem = __mmap_exec(4096);
    let buf = __bytes_new(__array_len(code));
    let i = 0;
    while i < __array_len(code) {
        __bytes_set(buf, i, __array_get(code, i));
        let i = i + 1;
    };
    __memcpy_to(mem, buf, __array_len(code));
    return mem;
}
