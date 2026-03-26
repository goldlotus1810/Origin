#!/usr/bin/env python3
"""Generate a test origin.olang binary with simple bytecode.
Test: Push "Hello from ASM VM!\n" → Emit → Halt
"""
import struct
import sys

# Bytecode
msg = b"Hello from ASM VM!\n"
bytecode = bytearray()
# Push chain: [0x30, len_u8, bytes...]
bytecode.append(0x30)
bytecode.append(len(msg))
bytecode.extend(msg)
# Emit: [0x02]
bytecode.append(0x02)
# Halt: [0x07]
bytecode.append(0x07)

# Header (32 bytes)
magic = "○LNG".encode('utf-8')  # ○ is 3 bytes UTF-8, so "○LNG" = 6 bytes
# Pad/truncate magic to 4 bytes? Actually the plan says 4B magic.
# Let's use raw bytes: 0xE2 0x97 0x8B 0x4C (first 4 bytes of "○L")
# Actually let's just use a simple 4-byte magic for testing
magic = b'\xe2\x97\x8b\x4c'  # "○L" truncated to 4 bytes

version = 0x05
arch = 0x01  # x86_64

vm_offset = 0  # no VM code section in test
vm_size = 0
bc_offset = 32  # right after header
bc_size = len(bytecode)
kn_offset = 32 + bc_size
kn_size = 0
flags = 0

header = struct.pack('<4sBB II II II H',
    magic, version, arch,
    vm_offset, vm_size,
    bc_offset, bc_size,
    kn_offset, kn_size,
    flags)

assert len(header) == 32, f"Header is {len(header)} bytes, expected 32"

outpath = sys.argv[1] if len(sys.argv) > 1 else '/tmp/test_hello.olang'
with open(outpath, 'wb') as f:
    f.write(header)
    f.write(bytecode)

print(f"Written {len(header) + len(bytecode)} bytes to {outpath}")
print(f"Bytecode: {bc_size} bytes at offset {bc_offset}")
print(f"Header: {header.hex()}")
print(f"Bytecode hex: {bytecode.hex()}")
