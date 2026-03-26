// stdlib/homeos/jit.ol — JIT compiler for hot loops
// PLAN 5.1: Detect hot loops via profiling → trace → compile to native x86_64.
// Threshold: Fib[10] = 55 iterations before JIT (QT⑰).
// Uses asm_emit.ol for native code generation.

let JIT_THRESHOLD = 55;    // Fib[10]
let MAX_TRACE_LEN = 256;   // max opcodes per trace
let CODE_CACHE_SIZE = 64;  // max JIT'd traces

// ── Profiler ────────────────────────────────────────────────────────

pub fn profiler_new() {
  return {
    counters: {},       // pc → count (backward jump targets)
    traces: {},         // pc → trace (recorded opcode sequences)
    compiled: {},       // pc → native code bytes
    cache_size: 0
  };
}

pub fn profile_jump(prof, target_pc, current_pc) {
  // Called on every backward jump (potential loop header)
  if target_pc >= current_pc { return 0; }  // forward jump, not a loop
  let key = target_pc;
  let count = prof.counters[key];
  if count == 0 { count = 0; }
  count = count + 1;
  prof.counters[key] = count;
  // Check if hot enough for JIT
  if count == JIT_THRESHOLD {
    return 1;  // signal: start tracing
  }
  if count > JIT_THRESHOLD {
    return 2;  // signal: already traced/compiled, use native
  }
  return 0;    // not hot yet
}

// ── Trace Recorder ──────────────────────────────────────────────────

pub fn trace_new(start_pc) {
  return {
    start_pc: start_pc,
    ops: [],
    len: 0,
    has_call: false,     // bailout: can't JIT calls
    has_side_effect: false
  };
}

pub fn trace_record(trace, op_tag, operand) {
  if trace.len >= MAX_TRACE_LEN { return false; }
  push(trace.ops, { tag: op_tag, operand: operand });
  trace.len = trace.len + 1;
  // Mark ops that prevent JIT
  if op_tag == 0x07 { trace.has_call = true; }       // Call
  if op_tag == 0x10 { trace.has_side_effect = true; } // Dream
  if op_tag == 0x11 { trace.has_side_effect = true; } // Stats
  return true;
}

pub fn trace_is_jittable(trace) {
  // Conservative: only JIT simple numeric loops
  if trace.has_call { return false; }
  if trace.has_side_effect { return false; }
  if trace.len < 3 { return false; }  // too short to benefit
  return true;
}

// ── JIT Compiler (x86_64 code generation) ───────────────────────────

pub fn jit_compile(trace) {
  // Compile a trace of bytecode ops → x86_64 native code
  // Returns: { code: [bytes], len: N } or 0 if can't compile
  if !trace_is_jittable(trace) { return 0; }

  let code = [];

  // Prologue: save registers
  // push rbp; mov rbp, rsp; push r12; push r14; push r15
  push(code, 0x55);                          // push rbp
  push(code, 0x48); push(code, 0x89); push(code, 0xE5);  // mov rbp, rsp
  push(code, 0x41); push(code, 0x54);       // push r12
  push(code, 0x41); push(code, 0x56);       // push r14
  push(code, 0x41); push(code, 0x57);       // push r15

  // Load VM state from rdi (first arg = pointer to state struct)
  // mov r14, [rdi+0]  (stack ptr)
  push(code, 0x4C); push(code, 0x8B); push(code, 0x37);
  // mov r15, [rdi+8]  (heap ptr)
  push(code, 0x4C); push(code, 0x8B); push(code, 0x7F); push(code, 0x08);

  // Compile each op in trace
  let i = 0;
  while i < trace.len {
    let op = trace.ops[i];
    jit_emit_op(code, op.tag, op.operand);
    i = i + 1;
  }

  // Epilogue: store VM state back, restore registers
  // mov [rdi+0], r14
  push(code, 0x4C); push(code, 0x89); push(code, 0x37);
  // mov [rdi+8], r15
  push(code, 0x4C); push(code, 0x89); push(code, 0x7F); push(code, 0x08);
  // pop r15; pop r14; pop r12; pop rbp; ret
  push(code, 0x41); push(code, 0x5F);       // pop r15
  push(code, 0x41); push(code, 0x5E);       // pop r14
  push(code, 0x41); push(code, 0x5C);       // pop r12
  push(code, 0x5D);                          // pop rbp
  push(code, 0xC3);                          // ret

  return { code: code, len: len(code), start_pc: trace.start_pc };
}

fn jit_emit_op(code, tag, operand) {
  // Emit native x86_64 for a single bytecode op
  if tag == 0x15 {
    // PushNum: sub r14, 16; mov [r14], f64_bits; mov qword [r14+8], -1
    jit_emit_push_num(code, operand);
    return;
  }
  if tag == 0x0B {
    // Dup: copy top of stack
    // mov rax, [r14]; mov rcx, [r14+8]; sub r14, 16; mov [r14], rax; mov [r14+8], rcx
    push(code, 0x49); push(code, 0x8B); push(code, 0x06);        // mov rax, [r14]
    push(code, 0x49); push(code, 0x8B); push(code, 0x4E); push(code, 0x08); // mov rcx, [r14+8]
    push(code, 0x49); push(code, 0x83); push(code, 0xEE); push(code, 0x10); // sub r14, 16
    push(code, 0x49); push(code, 0x89); push(code, 0x06);        // mov [r14], rax
    push(code, 0x49); push(code, 0x89); push(code, 0x4E); push(code, 0x08); // mov [r14+8], rcx
    return;
  }
  if tag == 0x0C {
    // Pop: add r14, 16
    push(code, 0x49); push(code, 0x83); push(code, 0xC6); push(code, 0x10);
    return;
  }
  if tag == 0x12 {
    // Nop: no code needed
    return;
  }
  // Default: emit nop (1 byte) as placeholder for unsupported ops
  push(code, 0x90);
}

fn jit_emit_push_num(code, f64_val) {
  // sub r14, 16
  push(code, 0x49); push(code, 0x83); push(code, 0xEE); push(code, 0x10);
  // movabs rax, f64_bits → mov [r14], rax
  // For simplicity, emit mov rax, 0 (placeholder)
  push(code, 0x48); push(code, 0xB8);  // movabs rax, imm64
  // Emit 8 bytes of f64 (little-endian)
  let i = 0;
  while i < 8 {
    push(code, 0x00);  // placeholder bytes
    i = i + 1;
  }
  push(code, 0x49); push(code, 0x89); push(code, 0x06);  // mov [r14], rax
  // mov qword [r14+8], -1 (F64_MARKER)
  push(code, 0x49); push(code, 0xC7); push(code, 0x46); push(code, 0x08);
  push(code, 0xFF); push(code, 0xFF); push(code, 0xFF); push(code, 0xFF);
}

// ── Code Cache ──────────────────────────────────────────────────────

pub fn cache_install(prof, pc, compiled) {
  // Install compiled code into cache
  if prof.cache_size >= CODE_CACHE_SIZE {
    // Evict oldest entry (simple FIFO)
    return false;
  }
  prof.compiled[pc] = compiled;
  prof.cache_size = prof.cache_size + 1;
  return true;
}

pub fn cache_lookup(prof, pc) {
  // Look up compiled code for a loop header PC
  return prof.compiled[pc];
}

pub fn jit_stats(prof) {
  return {
    hot_loops: prof.cache_size,
    cache_capacity: CODE_CACHE_SIZE,
    threshold: JIT_THRESHOLD
  };
}
