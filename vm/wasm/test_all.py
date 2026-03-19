#!/usr/bin/env python3
"""Generate test bytecodes for WASM VM (codegen.ol format)"""
import struct, os

def write_test(name, bc):
    path = f'/tmp/wasm_test_{name}.bin'
    with open(path, 'wb') as f:
        f.write(bc)
    return path

def push_str(bc, s):
    data = s.encode('utf-8') if isinstance(s, str) else s
    bc.append(0x01); bc.extend(struct.pack('<H', len(data))); bc.extend(data)

def push_num(bc, v):
    bc.append(0x15); bc.extend(struct.pack('<d', v))

def store(bc, n):
    bc.append(0x13); n = n.encode(); bc.append(len(n)); bc.extend(n)

def load(bc, n):
    bc.append(0x02); n = n.encode(); bc.append(len(n)); bc.extend(n)

def call(bc, n):
    bc.append(0x07); n = n.encode(); bc.append(len(n)); bc.extend(n)

def emit(bc): bc.append(0x06)
def halt(bc): bc.append(0x0F)
def dup(bc): bc.append(0x0B)
def pop(bc): bc.append(0x0C)

def jz(bc):
    pos = len(bc)
    bc.append(0x0A); bc.extend(struct.pack('<I', 0))
    return pos

def jmp(bc):
    pos = len(bc)
    bc.append(0x09); bc.extend(struct.pack('<I', 0))
    return pos

def patch(bc, pos, target):
    struct.pack_into('<I', bc, pos + 1, target)

# ── Test 1: Hello ──
bc = bytearray()
push_str(bc, "Hello from WASM VM!\n")
emit(bc)
halt(bc)
write_test('hello', bc)

# ── Test 2: Math 2+3=5 ──
bc = bytearray()
push_num(bc, 2.0)
push_num(bc, 3.0)
call(bc, "__hyp_add")
store(bc, "result")
load(bc, "result")
emit(bc)
halt(bc)
write_test('math', bc)

# ── Test 3: Store/Load ──
bc = bytearray()
push_str(bc, "stored_value")
store(bc, "x")
load(bc, "x")
emit(bc)
push_str(bc, "\n")
emit(bc)
halt(bc)
write_test('vars', bc)

# ── Test 4: Countdown loop 3→1 ──
bc = bytearray()
push_num(bc, 3.0)
store(bc, "n")

loop_top = len(bc)
load(bc, "n")
dup(bc)
jz_pos = jz(bc)

# emit n
emit(bc)

# n = n - 1
load(bc, "n")
push_num(bc, 1.0)
call(bc, "__hyp_sub")
store(bc, "n")

jmp_pos = jmp(bc)
patch(bc, jmp_pos, loop_top)

halt_pos = len(bc)
patch(bc, jz_pos, halt_pos)
pop(bc)  # pop the 0 from jz
halt(bc)
write_test('loop', bc)

# ── Test 5: Comparison ──
bc = bytearray()
push_num(bc, 5.0)
push_num(bc, 3.0)
call(bc, "__cmp_gt")
jz_pos = jz(bc)

push_str(bc, "5>3=true\n")
emit(bc)
jmp_pos = jmp(bc)

patch(bc, jz_pos, len(bc))
push_str(bc, "5>3=false\n")
emit(bc)

patch(bc, jmp_pos, len(bc))
halt(bc)
write_test('cmp', bc)

print("Generated 5 test files:")
for name in ['hello', 'math', 'vars', 'loop', 'cmp']:
    print(f"  /tmp/wasm_test_{name}.bin")
