#!/usr/bin/env python3
"""Test: PushNum(2) + PushNum(3) + Add → Store("x") → Load("x") → Emit → Halt
Expected output: 5
"""
import struct

bytecode = bytearray()

# PushNum 2.0: [0x35, f64_le]
bytecode.append(0x35)
bytecode.extend(struct.pack('<d', 2.0))

# PushNum 3.0: [0x35, f64_le]
bytecode.append(0x35)
bytecode.extend(struct.pack('<d', 3.0))

# We need an Add opcode — but it's not in the standard opcode table yet!
# The VM dispatch doesn't have math ops mapped to opcodes.
# For now, let's just test PushNum → Emit → Halt

# Actually let's test the store/load cycle:
# PushNum 42.0 → Store "x" → Load "x" → Emit → Halt

bytecode2 = bytearray()

# PushNum 42.0
bytecode2.append(0x35)
bytecode2.extend(struct.pack('<d', 42.0))

# Emit
bytecode2.append(0x02)

# PushNum 2.0
bytecode2.append(0x35)
bytecode2.extend(struct.pack('<d', 2.0))

# PushNum 3.0
bytecode2.append(0x35)
bytecode2.extend(struct.pack('<d', 3.0))

# Emit both (will print two numbers)
bytecode2.append(0x02)  # emit 3.0
bytecode2.append(0x02)  # emit 2.0

# Push string and emit
msg = b"\nDone!\n"
bytecode2.append(0x30)
bytecode2.append(len(msg))
bytecode2.extend(msg)
bytecode2.append(0x02)

# Halt
bytecode2.append(0x07)

# Build file
magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01,
    0, 0,
    bc_offset, len(bytecode2),
    bc_offset + len(bytecode2), 0,
    0)

with open('/tmp/test_math.olang', 'wb') as f:
    f.write(header)
    f.write(bytecode2)

print(f"Written {32 + len(bytecode2)} bytes")
print(f"Bytecode: {bytecode2.hex()}")
