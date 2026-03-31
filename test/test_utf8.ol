// Test: UTF-8 decoding

let ok = 1;

// ASCII: 'A' = U+0041 = 65
let cp_a = __utf8_cp("A", 0);
if cp_a != 65 { let ok = 0; emit "FAIL A"; emit cp_a; };

// 2-byte: 'é' = U+00E9 = 233
let cp_e = __utf8_cp("é", 0);
if cp_e != 233 { let ok = 0; emit "FAIL e-acute"; emit cp_e; };

// 2-byte: 'đ' = U+0111 = 273
let cp_d = __utf8_cp("đ", 0);
if cp_d != 273 { let ok = 0; emit "FAIL d-stroke"; emit cp_d; };

// UTF-8 byte lengths
if __utf8_len("A", 0) != 1 { let ok = 0; emit "FAIL len A"; };
if __utf8_len("é", 0) != 2 { let ok = 0; emit "FAIL len e"; };

if ok { emit "PASS"; } else { emit "FAIL"; };
