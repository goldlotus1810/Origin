#!/usr/bin/env python3
"""Minimal loop test: count from 3 to 0, emit each"""
import struct

bc = bytearray()

# PushNum 3.0
bc.append(0x35)
bc.extend(struct.pack('<d', 3.0))
# Store "n"
bc.append(0x33)
bc.append(1)
bc.extend(b"n")

loop_start = len(bc)

# Load "n"
bc.append(0x31)
bc.append(1)
bc.extend(b"n")

# Dup (for Jz check)
bc.append(0x04)

# Jz → halt
jz_pos = len(bc)
bc.append(0x22)
bc.extend(struct.pack('<I', 0))  # placeholder

# Emit (the dup'd value)
bc.append(0x02)

# Load n, PushNum 1, Call __hyp_sub, Store n
bc.append(0x31); bc.append(1); bc.extend(b"n")
bc.append(0x35); bc.extend(struct.pack('<d', 1.0))
bc.append(0x32); name = b"__hyp_sub"; bc.append(len(name)); bc.extend(name)
bc.append(0x33); bc.append(1); bc.extend(b"n")

# Jmp loop_start
bc.append(0x21)
bc.extend(struct.pack('<I', loop_start))

halt_pos = len(bc)
# Pop the 0 from Jz
bc.append(0x07)  # Halt

struct.pack_into('<I', bc, jz_pos + 1, halt_pos)

magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01, 0, 0,
    bc_offset, len(bc), bc_offset + len(bc), 0, 0)

with open('/tmp/test_loop.olang', 'wb') as f:
    f.write(header)
    f.write(bc)

print(f"Written {32+len(bc)} bytes, loop_start={loop_start}, halt={halt_pos}")
print(f"Bytecode: {bc.hex()}")
