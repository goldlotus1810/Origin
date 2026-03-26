// stdlib/typecheck.ol — Formal type checking + contract verification
// Inspired by Jolie's mathematically-defined execution semantics
//
// Usage:
//   fn add(a, b) { return a + b; };
//   assert_type(add(1, 2), "number");
//   contract("add", fn(r) { return r > 0; }, add(3, 4));
//
// Type system: Num, Str, Array, Dict, Fn, Bool, Nil

// ── Type checking ──

// __type_of now returns u16 molecules: "number", "string", "array", "dict", "closure", "nil"

pub fn type_name(_tn_val) {
    let _tn_t = __type_of(_tn_val);
    if _tn_t == "number" { return "Num"; };
    if _tn_t == "string" { return "Str"; };
    if _tn_t == "array" { return "Array"; };
    if _tn_t == "dict" { return "Dict"; };
    if _tn_t == "closure" { return "Fn"; };
    return "Unknown";
}

pub fn is_num(_in_v) { return __type_of(_in_v) == "number"; }
pub fn is_str(_is_v) { return __type_of(_is_v) == "string"; }
pub fn is_array(_ia_v) { return __type_of(_ia_v) == "array"; }
pub fn is_dict(_id_v) { return __type_of(_id_v) == "dict"; }
pub fn is_fn(_if_v) { return __type_of(_if_v) == "closure"; }

// ── Type assertion ──

pub fn assert_type(_at_val, _at_expected) {
    let _at_actual = type_name(_at_val);
    if _at_actual != _at_expected {
        __throw("Type error: expected " + _at_expected + ", got " + _at_actual);
    };
    return _at_val;
}

// ── Contract verification (Jolie-style formal semantics) ──
// A contract is a predicate that must hold for a computation result.
// contract(name, predicate, value) → value if predicate(value) is true, throws otherwise

pub fn contract(_ct_name, _ct_pred, _ct_val) {
    if _ct_pred(_ct_val) != 1 {
        __throw("Contract violation: " + _ct_name + " failed for value " + __to_string(_ct_val));
    };
    return _ct_val;
}

// ── Pre/post conditions ──
// requires(condition, msg) → throws if condition is false
// ensures(condition, msg) → throws if condition is false

pub fn requires(_rq_cond, _rq_msg) {
    if _rq_cond != 1 {
        __throw("Precondition failed: " + _rq_msg);
    };
    return 1;
}

pub fn ensures(_en_cond, _en_msg) {
    if _en_cond != 1 {
        __throw("Postcondition failed: " + _en_msg);
    };
    return 1;
}

// ── Typed function wrapper ──
// typed_fn(param_types, return_type, fn) → wrapper that checks types at call time
// param_types = ["Num", "Str", ...], return_type = "Num"

pub fn typed_fn(_tf_ptypes, _tf_rtype, _tf_fn) {
    // Return a wrapper closure that checks argument types
    // Note: Olang closures capture by reference
    return fn(_tf_a1) {
        // Single-arg version (most common)
        assert_type(_tf_a1, _tf_ptypes[0]);
        let _tf_result = _tf_fn(_tf_a1);
        assert_type(_tf_result, _tf_rtype);
        return _tf_result;
    };
}

// ── Invariant checking ──
// invariant(name, predicate) → returns a checker function
// Usage: let check_positive = invariant("positive", fn(x) { return x > 0; });
//        check_positive(42);  // ok
//        check_positive(-1);  // throws

pub fn invariant(_iv_name, _iv_pred) {
    return fn(_iv_val) {
        return contract(_iv_name, _iv_pred, _iv_val);
    };
}

// ── Formal specification patterns ──

// spec: Define a formal specification for a function
// spec(name, precondition, postcondition, implementation)
// Returns a checked wrapper

pub fn spec(_sp_name, _sp_pre, _sp_post, _sp_impl) {
    return fn(_sp_arg) {
        requires(_sp_pre(_sp_arg), _sp_name + " precondition");
        let _sp_result = _sp_impl(_sp_arg);
        ensures(_sp_post(_sp_result), _sp_name + " postcondition");
        return _sp_result;
    };
}

// ── Array type checking ──

pub fn assert_array_of(_aao_arr, _aao_type) {
    assert_type(_aao_arr, "Array");
    let _aao_i = 0;
    while _aao_i < len(_aao_arr) {
        assert_type(_aao_arr[_aao_i], _aao_type);
        _aao_i = _aao_i + 1;
    };
    return _aao_arr;
}

// ── Numeric constraints ──

pub fn assert_positive(_ap_v) {
    assert_type(_ap_v, "Num");
    if _ap_v <= 0 { __throw("Expected positive number, got " + __to_string(_ap_v)); };
    return _ap_v;
}

pub fn assert_range(_ar_v, _ar_lo, _ar_hi) {
    assert_type(_ar_v, "Num");
    if _ar_v < _ar_lo { __throw("Value " + __to_string(_ar_v) + " below minimum " + __to_string(_ar_lo)); };
    if _ar_v > _ar_hi { __throw("Value " + __to_string(_ar_v) + " above maximum " + __to_string(_ar_hi)); };
    return _ar_v;
}
