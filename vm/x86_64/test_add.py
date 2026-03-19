#!/usr/bin/env python3
"""DoD #3: PushNum(2) + PushNum(3) + Call("__hyp_add") + Emit → "5"
   DoD #4: Countdown loop 3→0 using Jmp+Jz
"""
import struct

bc = bytearray()

# ── Test 1: 2 + 3 = 5 ──
# PushNum 2.0
bc.append(0x35)
bc.extend(struct.pack('<d', 2.0))

# PushNum 3.0
bc.append(0x35)
bc.extend(struct.pack('<d', 3.0))

# Call "__hyp_add"
name = b"__hyp_add"
bc.append(0x32)
bc.append(len(name))
bc.extend(name)

# Emit result (should be 5)
bc.append(0x02)

# Push newline and emit
bc.append(0x30)
bc.append(1)
bc.append(0x0A)  # \n
bc.append(0x02)

# ── Test 2: Countdown loop 3→0 ──
# Store counter: PushNum 3 → Store "n"
bc.append(0x35)
bc.extend(struct.pack('<d', 3.0))
bc.append(0x33)
bc.append(1)
bc.extend(b"n")

# Loop start:
loop_start = len(bc)

# Load "n"
bc.append(0x31)
bc.append(1)
bc.extend(b"n")

# Jz → exit (if n == 0, jump to halt)
jz_pos = len(bc)
bc.append(0x22)
bc.extend(struct.pack('<I', 0))  # placeholder

# Load "n" again, emit it
bc.append(0x31)
bc.append(1)
bc.extend(b"n")
bc.append(0x02)  # emit

# Push newline
bc.append(0x30)
bc.append(1)
bc.append(0x0A)
bc.append(0x02)

# n = n - 1: Load n, PushNum 1, Call __hyp_sub, Store n
bc.append(0x31)
bc.append(1)
bc.extend(b"n")
bc.append(0x35)
bc.extend(struct.pack('<d', 1.0))
bc.append(0x32)
name_sub = b"__hyp_sub"
bc.append(len(name_sub))
bc.extend(name_sub)
bc.append(0x33)
bc.append(1)
bc.extend(b"n")

# Jmp → loop_start
bc.append(0x21)
bc.extend(struct.pack('<I', loop_start))

# Halt (Jz target)
halt_pos = len(bc)
bc.append(0x07)

# Patch Jz
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

with open('/tmp/test_add.olang', 'wb') as f:
    f.write(header)
    f.write(bc)

print(f"Written {32 + len(bc)} bytes")
print(f"Expected output: 5\\n3\\n2\\n1\\n")
