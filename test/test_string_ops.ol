// Test: String operations — split, join, contains, sort

let ok = 1;

// Split
let parts = split("hello,world,olang", ",");
if __array_len(parts) != 3 { let ok = 0; emit "FAIL split len"; };

// Join
let joined = join(["a", "b", "c"], "-");
if joined != "a-b-c" { let ok = 0; emit "FAIL join"; emit joined; };

// Contains
if contains("hello world", "world") != 1 { let ok = 0; emit "FAIL contains true"; };
if contains("hello", "xyz") != 0 { let ok = 0; emit "FAIL contains false"; };

// Sort numbers
let sorted = sort([5, 1, 4, 2, 3]);
if __array_get(sorted, 0) != 1 { let ok = 0; emit "FAIL sort[0]"; };
if __array_get(sorted, 4) != 5 { let ok = 0; emit "FAIL sort[4]"; };

// Trim
let trimmed = __str_trim("  hello  ");
if trimmed != "hello" { let ok = 0; emit "FAIL trim"; emit trimmed; };

if ok { emit "PASS"; } else { emit "FAIL"; };
