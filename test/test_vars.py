#!/usr/bin/env python3
"""Test: Store/Load variables + control flow (Jz)
Program: let x = 42.0; emit x; if x → emit "yes\n"; halt
"""
import struct

bc = bytearray()

# PushNum 42.0
bc.append(0x35)
bc.extend(struct.pack('<d', 42.0))

# Store "x"
bc.append(0x33)
name = b"x"
bc.append(len(name))
bc.extend(name)

# Load "x"
bc.append(0x31)
bc.append(len(name))
bc.extend(name)

# Emit (should print 42)
bc.append(0x02)

# Load "x" again for Jz test
bc.append(0x31)
bc.append(len(name))
bc.extend(name)

# Jz → skip_target (if x == 0, jump past "yes")
jz_pos = len(bc)
bc.append(0x22)
bc.extend(struct.pack('<I', 0))  # placeholder target

# Push "yes\n" and emit
msg = b"yes\n"
bc.append(0x30)
bc.append(len(msg))
bc.extend(msg)
bc.append(0x02)

# Halt
halt_pos = len(bc)
bc.append(0x07)

# Patch Jz target to halt_pos
struct.pack_into('<I', bc, jz_pos + 1, halt_pos)

# Build file
magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01,
    0, 0,
    bc_offset, len(bc),
    bc_offset + len(bc), 0,
    0)

with open('/tmp/test_vars.olang', 'wb') as f:
    f.write(header)
    f.write(bc)

print(f"Written {32 + len(bc)} bytes, Jz target={halt_pos}")
