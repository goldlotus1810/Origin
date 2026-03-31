// stdlib/jit_compile.ol — JIT compilation framework
// jit_fib() → hardcoded native fib
// jit_auto(fn_name) → profile + compile + register (future: general)
// jit_warmup(fn_name, arg) → run interpreted, then auto-JIT if hot

// ── Native fib(n) → 42 bytes x86-64 ──
pub fn jit_fib() {
    let code = [];
    let _ = __push(code, 0x48); let _ = __push(code, 0x83);
    let _ = __push(code, 0xFF); let _ = __push(code, 0x02);
    let _ = __push(code, 0x7D); let _ = __push(code, 0x04);
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xF8);
    let _ = __push(code, 0xC3);
    let _ = __push(code, 0x53); let _ = __push(code, 0x55);
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xFB);
    let _ = __push(code, 0x48); let _ = __push(code, 0x8D);
    let _ = __push(code, 0x7B); let _ = __push(code, 0xFF);
    let _ = __push(code, 0xE8);
    let _ = __push(code, 0xE8); let _ = __push(code, 0xFF);
    let _ = __push(code, 0xFF); let _ = __push(code, 0xFF);
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xC5);
    let _ = __push(code, 0x48); let _ = __push(code, 0x8D);
    let _ = __push(code, 0x7B); let _ = __push(code, 0xFE);
    let _ = __push(code, 0xE8);
    let _ = __push(code, 0xDC); let _ = __push(code, 0xFF);
    let _ = __push(code, 0xFF); let _ = __push(code, 0xFF);
    let _ = __push(code, 0x48); let _ = __push(code, 0x01); let _ = __push(code, 0xE8);
    let _ = __push(code, 0x5D); let _ = __push(code, 0x5B); let _ = __push(code, 0xC3);
    return _jit_install(code);
}

// ── Native sum(n) → iterative loop ──
pub fn jit_sum() {
    let code = [];
    let _ = __push(code, 0x48); let _ = __push(code, 0x31); let _ = __push(code, 0xC0);
    let _ = __push(code, 0x48); let _ = __push(code, 0x31); let _ = __push(code, 0xC9);
    let _ = __push(code, 0x48); let _ = __push(code, 0x39); let _ = __push(code, 0xF9);
    let _ = __push(code, 0x7D); let _ = __push(code, 0x06);
    let _ = __push(code, 0x48); let _ = __push(code, 0x01); let _ = __push(code, 0xC8);
    let _ = __push(code, 0x48); let _ = __push(code, 0xFF); let _ = __push(code, 0xC1);
    let _ = __push(code, 0xEB); let _ = __push(code, 0xF4);
    let _ = __push(code, 0xC3);
    return _jit_install(code);
}

// ── Native factorial(n) ──
pub fn jit_fact() {
    let code = [];
    // if rdi <= 1: return 1
    let _ = __push(code, 0x48); let _ = __push(code, 0x83); let _ = __push(code, 0xFF); let _ = __push(code, 0x01);
    // jg +8 (skip mov rax,1 [7 bytes] + ret [1 byte] = 8)
    let _ = __push(code, 0x7F); let _ = __push(code, 0x08);
    // mov rax, 1; ret
    let _ = __push(code, 0x48); let _ = __push(code, 0xC7); let _ = __push(code, 0xC0);
    let _ = __push(code, 0x01); let _ = __push(code, 0x00); let _ = __push(code, 0x00); let _ = __push(code, 0x00);
    let _ = __push(code, 0xC3);
    // push rbx; mov rbx, rdi
    let _ = __push(code, 0x53);
    let _ = __push(code, 0x48); let _ = __push(code, 0x89); let _ = __push(code, 0xFB);
    // dec rdi; call self
    let _ = __push(code, 0x48); let _ = __push(code, 0xFF); let _ = __push(code, 0xCF);
    let _ = __push(code, 0xE8);
    // call at offset 0x15, target 0x00: rel32 = 0 - (0x15+5) = -26 = 0xE6
    let _ = __push(code, 0xE6); let _ = __push(code, 0xFF); let _ = __push(code, 0xFF); let _ = __push(code, 0xFF);
    // imul rax, rbx
    let _ = __push(code, 0x48); let _ = __push(code, 0x0F); let _ = __push(code, 0xAF); let _ = __push(code, 0xC3);
    // pop rbx; ret
    let _ = __push(code, 0x5B); let _ = __push(code, 0xC3);
    return _jit_install(code);
}

// ── Install code: alloc exec memory + copy ──
fn _jit_install(_ji_code) {
    let _ji_mem = __mmap_exec(4096);
    let _ji_buf = __bytes_new(__array_len(_ji_code));
    let _ji_i = 0;
    while _ji_i < __array_len(_ji_code) {
        __bytes_set(_ji_buf, _ji_i, __array_get(_ji_code, _ji_i));
        let _ji_i = _ji_i + 1;
    };
    __memcpy_to(_ji_mem, _ji_buf, __array_len(_ji_code));
    return _ji_mem;
}

// ── One-line JIT: compile + register + call ──
pub fn jit_run_fib(_jrf_n) {
    let _jrf_ptr = jit_fib();
    __jit_register("fib", _jrf_ptr);
    return __call_native(_jrf_ptr, _jrf_n);
}

pub fn jit_run_fact(_jrf_n) {
    let _jrf_ptr = jit_fact();
    __jit_register("fact", _jrf_ptr);
    return __call_native(_jrf_ptr, _jrf_n);
}
