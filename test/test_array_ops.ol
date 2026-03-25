// Test: Array operations — create, push, get, set, len

let a = [];
__push(a, 10);
__push(a, 20);
__push(a, 30);

let ok = 1;

// Length
if __array_len(a) != 3 { let ok = 0; emit "FAIL len"; };

// Get
if __array_get(a, 0) != 10 { let ok = 0; emit "FAIL get 0"; };
if __array_get(a, 2) != 30 { let ok = 0; emit "FAIL get 2"; };

// Set
__set_at(a, 1, 99);
if __array_get(a, 1) != 99 { let ok = 0; emit "FAIL set"; };

// Array literal
let b = [5, 10, 15, 20];
if __array_len(b) != 4 { let ok = 0; emit "FAIL literal len"; };
if __array_get(b, 3) != 20 { let ok = 0; emit "FAIL literal get"; };

if ok { emit "PASS"; } else { emit "FAIL"; };
