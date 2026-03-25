// Test: Function scope save/restore
// Verify nested function calls don't corrupt variables

fn outer() {
    let x = 100;
    let y = inner(10);
    // x should still be 100 after inner() returns
    return x + y;
};

fn inner(n) {
    let x = 999;  // different x — should NOT overwrite outer's x
    return n * 2;
};

let result = outer();
if result == 120 {
    emit "PASS";
} else {
    emit "FAIL";
    emit result;
};
