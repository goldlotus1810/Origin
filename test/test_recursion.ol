// Test: Deep recursion — fibonacci, factorial, scope integrity

let ok = 1;

// Fibonacci
fn fib(n) {
    if n < 2 { return n; };
    return fib(n - 1) + fib(n - 2);
};

let f10 = fib(10);
if f10 != 55 { let ok = 0; emit "FAIL fib(10)"; emit f10; };

let f20 = fib(20);
if f20 != 6765 { let ok = 0; emit "FAIL fib(20)"; emit f20; };

// Factorial
fn fact(n) {
    if n <= 1 { return 1; };
    return n * fact(n - 1);
};

let f8 = fact(8);
if f8 != 40320 { let ok = 0; emit "FAIL fact(8)"; emit f8; };

if ok { emit "PASS"; } else { emit "FAIL"; };
