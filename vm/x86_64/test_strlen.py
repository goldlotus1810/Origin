#!/usr/bin/env python3
"""Test __len and __char_at with codegen format"""
import struct

bc = bytearray()

def push_str(s):
    data = s.encode('utf-8')
    bc.append(0x01); bc.extend(struct.pack('<H', len(data))); bc.extend(data)
def push_num(v):
    bc.append(0x15); bc.extend(struct.pack('<d', v))
def store(n):
    bc.append(0x13); n=n.encode(); bc.append(len(n)); bc.extend(n)
def load(n):
    bc.append(0x02); n=n.encode(); bc.append(len(n)); bc.extend(n)
def call(n):
    bc.append(0x07); n=n.encode(); bc.append(len(n)); bc.extend(n)
def emit():
    bc.append(0x06)
def halt():
    bc.append(0x0F)
def dup():
    bc.append(0x0B)

# Store "hello" as input
push_str("hello")
store("s")

# Get length
load("s")
call("__len")
emit()  # should print 5

# Get char at 1
load("s")
push_num(1)
call("__char_at")
emit()  # should print 'e'

# Newline
push_str("\n")
emit()

halt()

magic = b'\xe2\x97\x8b\x4c'
bc_offset = 32
header = struct.pack('<4sBB II II II H',
    magic, 0x05, 0x01, 0, 0,
    bc_offset, len(bc), bc_offset+len(bc), 0, 1)

with open('/tmp/test_strlen.olang', 'wb') as f:
    f.write(header); f.write(bc)
print(f"Expected: 5e")
