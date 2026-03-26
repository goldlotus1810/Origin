// JIT: compile fib(n) → native x86-64 → run on CPU
// Emit raw machine code bytes → mmap executable → call

let code = [];

// fib(rdi) → rax
// cmp rdi, 2
push(code, 0x48); push(code, 0x83); push(code, 0xFF); push(code, 0x02);
// jge .recurse (+4 bytes: skip mov+ret = 3+1 = 4 bytes)
push(code, 0x7D); push(code, 0x04);
// mov rax, rdi; ret (base case: return n)
push(code, 0x48); push(code, 0x89); push(code, 0xF8);
push(code, 0xC3);
// .recurse: (offset 10)
// push rbx; push rbp (save + align)
push(code, 0x53);
push(code, 0x55);
// mov rbx, rdi
push(code, 0x48); push(code, 0x89); push(code, 0xFB);
// lea rdi, [rbx-1]
push(code, 0x48); push(code, 0x8D); push(code, 0x7B); push(code, 0xFF);
// call fib (offset 0) — rel32 = 0 - (19+5) = -24
push(code, 0xE8);
push(code, 0xE8); push(code, 0xFF); push(code, 0xFF); push(code, 0xFF);
// mov rbp, rax (save fib(n-1))
push(code, 0x48); push(code, 0x89); push(code, 0xC5);
// lea rdi, [rbx-2]
push(code, 0x48); push(code, 0x8D); push(code, 0x7B); push(code, 0xFE);
// call fib — rel32 = 0 - (31+5) = -36 = 0xDC
push(code, 0xE8);
push(code, 0xDC); push(code, 0xFF); push(code, 0xFF); push(code, 0xFF);
// add rax, rbp
push(code, 0x48); push(code, 0x01); push(code, 0xE8);
// pop rbp; pop rbx; ret
push(code, 0x5D);
push(code, 0x5B);
push(code, 0xC3);

emit "Code: " + to_string(len(code)) + " bytes";

// Allocate executable memory
let mem = __mmap_exec(4096);
emit "Exec: " + to_string(mem);

// Copy code to executable memory
let buf = __bytes_new(len(code));
let i = 0;
while i < len(code) {
    __bytes_set(buf, i, code[i]);
    let i = i + 1;
};
__memcpy_to(mem, buf, len(code));
emit "Copied";

// Benchmark: native fib(30)
let t0 = __time();
let r = __call_native(mem, 30);
let t1 = __time();
emit "NATIVE fib(30)=" + to_string(r) + " time=" + to_string(t1-t0) + "ms";

// Compare with interpreted
let t2 = __time();
fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); };
let r2 = fib(30);
let t3 = __time();
emit "INTERP fib(30)=" + to_string(r2) + " time=" + to_string(t3-t2) + "ms";

if r == 832040 { emit "PASS"; } else { emit "FAIL native=" + to_string(r); };
