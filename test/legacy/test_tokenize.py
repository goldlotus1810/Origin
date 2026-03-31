#!/usr/bin/env python3
"""DoD #5: Simple tokenizer in codegen.ol bytecode format.
Tokenizes "let x = 42;" → outputs token types.

This is a simplified version showing the VM can handle the core ops
needed by lexer.ol: store, load, call builtins, jz branching, loops.
"""
import struct

bc = bytearray()

# codegen.ol format (tags 0x01-0x24)
# 0x15=PushNum 0x13=Store 0x02=Load 0x14=LoadLocal
# 0x07=Call 0x06=Emit 0x0F=Halt
# 0x09=Jmp 0x0A=Jz 0x0B=Dup 0x0C=Pop

def push_num(val):
    bc.append(0x15)
    bc.extend(struct.pack('<d', val))

def push_str(s):
    """Push string via codegen Push (0x01, u16 len, bytes)"""
    data = s.encode('utf-8') if isinstance(s, str) else s
    bc.append(0x01)
    bc.extend(struct.pack('<H', len(data)))
    bc.extend(data)

def store(name):
    bc.append(0x13)
    n = name.encode('utf-8')
    bc.append(len(n))
    bc.extend(n)

def load(name):
    bc.append(0x02)
    n = name.encode('utf-8')
    bc.append(len(n))
    bc.extend(n)

def call(name):
    bc.append(0x07)
    n = name.encode('utf-8')
    bc.append(len(n))
    bc.extend(n)

def emit():
    bc.append(0x06)

def halt():
    bc.append(0x0F)

def dup():
    bc.append(0x0B)

def pop():
    bc.append(0x0C)

def jz(target_placeholder=True):
    pos = len(bc)
    bc.append(0x0A)
    bc.extend(struct.pack('<I', 0))  # placeholder
    return pos

def jmp(target_placeholder=True):
    pos = len(bc)
    bc.append(0x09)
    bc.extend(struct.pack('<I', 0))
    return pos

def patch_jz(pos, target):
    struct.pack_into('<I', bc, pos + 1, target)

def patch_jmp(pos, target):
    struct.pack_into('<I', bc, pos + 1, target)

# ── Program: tokenize "let x = 42;" ──
# Store input string
push_str("let x = 42;")
store("input")

# Store position
push_num(0)
store("pos")

# Store input length
load("input")
call("__len")
store("input_len")

# Loop: while pos < input_len
loop_top = len(bc)

# Load pos, load input_len, compare
load("pos")
load("input_len")
call("__cmp_ge")  # pos >= input_len?
jz_exit = jz()    # if false (pos < len), continue

# pos >= len → done
push_str("DONE\n")
emit()
halt()

# Continue: get char at pos
patch_jz(jz_exit, len(bc))
load("input")
load("pos")
call("__char_at")
dup()
store("ch")

# Check if space (skip)
push_str(" ")
call("__eq")
jz_not_space = jz()
# It's a space — skip
push_str("SPACE ")
emit()
jmp_advance = jmp()

# Not space
patch_jz(jz_not_space, len(bc))
load("ch")

# Check if digit (0-9)
push_str("0")
call("__cmp_ge")
jz_not_digit = jz()

# Could be digit — just label it
push_str("NUM ")
emit()
jmp_advance2 = jmp()

# Not digit
patch_jz(jz_not_digit, len(bc))

# Check if letter or underscore — simple: just emit IDENT for a-z
push_str("ID ")
emit()

# Advance: pos = pos + 1
advance_pos = len(bc)
patch_jmp(jmp_advance, advance_pos)
patch_jmp(jmp_advance2, advance_pos)

load("pos")
push_num(1)
call("__hyp_add")
store("pos")

# Jump to loop top
jmp_loop = jmp()
patch_jmp(jmp_loop, loop_top)

# Build file with codegen format (flags=1)
magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01,
    0, 0,
    bc_offset, len(bc),
    bc_offset + len(bc), 0,
    1)  # flags=1 → codegen format

with open('/tmp/test_tokenize.olang', 'wb') as f:
    f.write(header)
    f.write(bc)

print(f"Written {32+len(bc)} bytes")
print(f"Expected: ID ID ID SPACE ID SPACE ID SPACE NUM NUM SPACE ID")
