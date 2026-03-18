// stdlib/bytes.ol — Byte manipulation builtins for Olang
// Wraps low-level byte operations.

pub fn to_bytes(val) { val.to_bytes() }
pub fn from_bytes(b) { b.from_bytes() }
pub fn byte_len(b) { b.byte_len() }
pub fn get_u8(b, offset) { b.get_u8(offset) }
pub fn set_u8(b, offset, val) { b.set_u8(offset, val) }
pub fn get_u16_be(b, offset) { b.get_u16_be(offset) }
pub fn set_u16_be(b, offset, val) { b.set_u16_be(offset, val) }
pub fn get_u32_be(b, offset) { b.get_u32_be(offset) }
pub fn set_u32_be(b, offset, val) { b.set_u32_be(offset, val) }
