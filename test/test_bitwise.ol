// Test: Bitwise operations

let ok = 1;

// OR: 0b0101 | 0b0011 = 0b0111 = 7
if __bit_or(5, 3) != 7 { let ok = 0; emit "FAIL or"; };

// AND: 0b0111 & 0b0011 = 0b0011 = 3
if __bit_and(7, 3) != 3 { let ok = 0; emit "FAIL and"; };

// XOR: 0b0101 ^ 0b0011 = 0b0110 = 6
if __bit_xor(5, 3) != 6 { let ok = 0; emit "FAIL xor"; };

// SHL: 1 << 4 = 16
if __bit_shl(1, 4) != 16 { let ok = 0; emit "FAIL shl"; };

// SHR: 16 >> 2 = 4
if __bit_shr(16, 2) != 4 { let ok = 0; emit "FAIL shr"; };

// Combined: pack/unpack u16 via bitwise
let s = 5;
let r = 3;
let packed = __bit_or(__bit_shl(s, 4), r);
// 5 << 4 | 3 = 80 | 3 = 83
if packed != 83 { let ok = 0; emit "FAIL packed"; emit packed; };

// Extract back
let s2 = __bit_shr(packed, 4);
let r2 = __bit_and(packed, 15);
if s2 != 5 { let ok = 0; emit "FAIL unpack s"; };
if r2 != 3 { let ok = 0; emit "FAIL unpack r"; };

if ok { emit "PASS"; } else { emit "FAIL"; };
