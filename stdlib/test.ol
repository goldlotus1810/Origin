// stdlib/test.ol — Olang Test Framework (CUT.3)

let __test_pass = 0;
let __test_fail = 0;
let __test_total = 0;

pub fn assert_eq(_te_a, _te_e, _te_n) {
    let __test_total = __test_total + 1;
    if _te_a == _te_e {
        let __test_pass = __test_pass + 1;
    } else {
        let __test_fail = __test_fail + 1;
        emit "  FAIL: " + _te_n;
    };
}

pub fn assert_true(_tt_v, _tt_n) {
    let __test_total = __test_total + 1;
    if _tt_v {
        let __test_pass = __test_pass + 1;
    } else {
        let __test_fail = __test_fail + 1;
        emit "  FAIL: " + _tt_n;
    };
}

pub fn test_summary() {
    if __test_fail == 0 {
        emit "ALL PASS: " + __to_string(__test_pass) + "/" + __to_string(__test_total);
    } else {
        emit "FAILED: " + __to_string(__test_fail) + "/" + __to_string(__test_total);
    };
    return __test_fail;
}

pub fn run_all_tests() {
    let __test_pass = 0;
    let __test_fail = 0;
    let __test_total = 0;
    test_core();
    test_features();
    return test_summary();
}

fn test_core() {
    assert_eq(1 + 2, 3, "add");
    assert_eq(10 - 3, 7, "sub");
    assert_eq(4 * 5, 20, "mul");
    assert_eq(10 / 2, 5, "div");
    assert_eq(__floor(3.7), 3, "floor");
    assert_eq(__ceil(3.2), 4, "ceil");
    assert_eq(len("hello"), 5, "strlen");
    assert_eq(__to_string(42), "42", "tostr");
    let a = [1,2,3];
    assert_eq(len(a), 3, "arrlen");
    assert_eq(a[0], 1, "arrget");
}

fn test_features() {
    // While
    let s = 0; let i = 0;
    while i < 5 { s = s + i; i = i + 1; };
    assert_eq(s, 10, "while");
    // For-in
    let t = 0;
    for x in [10,20,30] { t = t + x; };
    assert_eq(t, 60, "forin");
    // Try/catch
    let c = 0;
    try { __throw("e"); } catch { c = 1; };
    assert_eq(c, 1, "trycatch");
    // Dict
    let d = { x: 42 };
    assert_eq(d.x, 42, "dict");
    // SHA-256
    assert_eq(len(__sha256("abc")), 64, "sha256len");
}
