#!/usr/bin/env python3
"""Test codegen.ol bytecode format (tags 0x01-0x24).
Program: PushNum(2) + PushNum(3) + Call("__hyp_add") + Emit + Halt
Expected: 5
"""
import struct

bc = bytearray()

# codegen format:
# 0x15 = PushNum, 0x07 = Call, 0x06 = Emit, 0x0F = Halt
# 0x13 = Store, 0x02 = Load

# PushNum 2.0
bc.append(0x15)
bc.extend(struct.pack('<d', 2.0))

# PushNum 3.0
bc.append(0x15)
bc.extend(struct.pack('<d', 3.0))

# Call "__hyp_add"
name = b"__hyp_add"
bc.append(0x07)
bc.append(len(name))
bc.extend(name)

# Emit
bc.append(0x06)

# Push newline via Store/Load trick? No, use Push (0x01)
# Actually Push in codegen format is [0x01, len_u16_LE, data]
# Let's just emit a newline as a chain
nl = b"\n"
bc.append(0x01)
bc.extend(struct.pack('<H', len(nl)))
bc.extend(nl)
bc.append(0x06)  # emit

# Halt
bc.append(0x0F)

# Build file with flags=1 (codegen format)
magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01,
    0, 0,
    bc_offset, len(bc),
    bc_offset + len(bc), 0,
    1)  # flags=1 → codegen format

with open('/tmp/test_codegen.olang', 'wb') as f:
    f.write(header)
    f.write(bc)

print(f"Written {32+len(bc)} bytes (codegen format)")
print(f"Expected: 5")
