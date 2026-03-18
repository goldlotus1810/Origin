// ─── stdlib/test.ol ─────────────────────────────────────────────────────────
// Test framework for Olang: assert functions + test runner.
//
// Usage:
//   use test;
//
//   fn test_addition() {
//       assert_eq(1 + 1, 2);
//   }
//
//   fn test_string() {
//       let s = "hello";
//       assert_eq(str_len(s), 5);
//       assert_true(str_contains(s, "ell"));
//   }
//
//   // Run: each test fn called, failures reported.
// ────────────────────────────────────────────────────────────────────────────

mod test;

// Assert two values are equal. Panics with message if not.
pub fn assert_eq(a, b) {
    __assert_eq(a, b);
}

// Assert two values are NOT equal. Panics with message if equal.
pub fn assert_ne(a, b) {
    __assert_ne(a, b);
}

// Assert value is truthy (non-empty). Panics if empty/falsy.
pub fn assert_true(val) {
    __assert_true(val);
}

// Assert value is falsy (empty). Panics if non-empty.
pub fn assert_false(val) {
    if val {
        panic(f"assert_false failed: value is truthy");
    }
}

// Assert that a string contains a substring.
pub fn assert_contains(haystack, needle) {
    if str_contains(haystack, needle) {
        // pass
    } else {
        panic(f"assert_contains failed: '{haystack}' does not contain '{needle}'");
    }
}

// Assert a number is approximately equal (within epsilon).
pub fn assert_approx(a, b, epsilon) {
    let diff = a - b;
    if diff < 0 {
        let diff = neg(diff);
    }
    if diff > epsilon {
        panic(f"assert_approx failed: {a} != {b} (diff={diff}, epsilon={epsilon})");
    }
}
