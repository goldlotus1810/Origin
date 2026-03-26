// stdlib/test.ol — Olang Test Framework (CUT.3)

let __test_pass = [0];
let __test_fail = [0];
let __test_total = [0];

pub fn assert_eq(_te_a, _te_e, _te_n) {
    set_at(__test_total, 0, __array_get(__test_total, 0) + 1);
    if _te_a == _te_e {
        set_at(__test_pass, 0, __array_get(__test_pass, 0) + 1);
    } else {
        set_at(__test_fail, 0, __array_get(__test_fail, 0) + 1);
        emit "  FAIL: " + _te_n;
    };
}

pub fn assert_true(_tt_v, _tt_n) {
    set_at(__test_total, 0, __array_get(__test_total, 0) + 1);
    if _tt_v {
        set_at(__test_pass, 0, __array_get(__test_pass, 0) + 1);
    } else {
        set_at(__test_fail, 0, __array_get(__test_fail, 0) + 1);
        emit "  FAIL: " + _tt_n;
    };
}

pub fn test_summary() {
    let _ts_fail = __array_get(__test_fail, 0);
    let _ts_pass = __array_get(__test_pass, 0);
    let _ts_total = __array_get(__test_total, 0);
    if _ts_fail == 0 {
        emit "ALL PASS: " + __to_string(_ts_pass) + "/" + __to_string(_ts_total);
    } else {
        emit "FAILED: " + __to_string(_ts_fail) + "/" + __to_string(_ts_total);
    };
    return _ts_fail;
}

pub fn run_all_tests() {
    set_at(__test_pass, 0, 0);
    set_at(__test_fail, 0, 0);
    set_at(__test_total, 0, 0);
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
    // Marker test to verify new tests run
    emit "NEW_TESTS_RUNNING";
    assert_eq(2 + 2, 4, "extra_add");
    // a[expr] with BinOp (BUG-INDEX regression)
    let arr = [10,20,30];
    assert_eq(arr[0 + 1], 20, "idx_binop");
    assert_eq(arr[2 - 1], 20, "idx_sub");
    let j = 0;
    assert_eq(arr[j + 2], 30, "idx_var_add");
    // Bubble sort (BUG-SORT regression)
    let sa = [5,2,8,1,9]; let sn = 5; let si = 0;
    while si < sn - 1 { let sj = 0; while sj < sn - 1 - si { if sa[sj] > sa[sj + 1] { let tmp = sa[sj]; set_at(sa, sj, sa[sj + 1]); set_at(sa, sj + 1, tmp); }; sj = sj + 1; }; si = si + 1; };
    assert_eq(sa[0], 1, "sort_first");
    assert_eq(sa[4], 9, "sort_last");
    // Nested while
    let nw = 0; let ni = 0;
    while ni < 3 { let nj = 0; while nj < 3 { nw = nw + 1; nj = nj + 1; }; ni = ni + 1; };
    assert_eq(nw, 9, "nested_while");
    // String concat (interpolation tested via REPL)
    let iname = "world";
    assert_eq("hi " + iname, "hi world", "str_concat");
    // set_at
    let sa2 = [10,20,30];
    set_at(sa2, 1, 99);
    assert_eq(sa2[1], 99, "set_at");
    // min_val / max_val / sum (boot stdlib)
    assert_eq(min_val([5,2,8]), 2, "min_val");
    assert_eq(max_val([5,2,8]), 8, "max_val");
    assert_eq(sum([1,2,3,4]), 10, "sum");
}
