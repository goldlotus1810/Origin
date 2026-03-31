// Test: Closures and higher-order functions

// 1. Lambda as value
let double = fn(x) { return x * 2; };
let r1 = double(21);

// 2. Pass lambda to function
fn apply(f, x) { return f(x); };
let r2 = apply(fn(x) { return x + 1; }, 41);

// 3. Map with lambda
let arr = [1, 2, 3];
let mapped = map(arr, fn(x) { return x * 10; });

// 4. Filter with lambda
let filtered = filter([1, 2, 3, 4, 5], fn(x) { return x > 3; });

// 5. Reduce
let total = reduce([1, 2, 3, 4], fn(a, b) { return a + b; });

let ok = 1;
if r1 != 42 { let ok = 0; emit "FAIL double"; };
if r2 != 42 { let ok = 0; emit "FAIL apply"; };
if __array_len(mapped) != 3 { let ok = 0; emit "FAIL map len"; };
if __array_get(mapped, 0) != 10 { let ok = 0; emit "FAIL map val"; };
if __array_len(filtered) != 2 { let ok = 0; emit "FAIL filter len"; };
if total != 10 { let ok = 0; emit "FAIL reduce"; };

if ok { emit "PASS"; } else { emit "FAIL"; };
