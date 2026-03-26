// Test: matrix.ol — vector and matrix operations
let _t_ok = 1;
if vec_dot([1,2,3], [4,5,6]) == 32 { emit "PASS vec_dot"; } else { emit "FAIL: vec_dot expected 32"; };
if vec_norm([3,4]) == 5 { emit "PASS vec_norm"; } else { emit "FAIL: vec_norm expected 5"; };
let _t_va = vec_add([1,2,3], [4,5,6]); if _t_va[0] == 5 { if _t_va[1] == 7 { if _t_va[2] == 9 { emit "PASS vec_add"; } else { emit "FAIL: vec_add[2]"; }; } else { emit "FAIL: vec_add[1]"; }; } else { emit "FAIL: vec_add[0]"; };
let _t_vs = vec_sub([10,20,30], [1,2,3]); if _t_vs[0] == 9 { if _t_vs[1] == 18 { if _t_vs[2] == 27 { emit "PASS vec_sub"; } else { emit "FAIL: vec_sub[2]"; }; } else { emit "FAIL: vec_sub[1]"; }; } else { emit "FAIL: vec_sub[0]"; };
let _t_vk = vec_scale([1,2,3], 10); if _t_vk[0] == 10 { if _t_vk[1] == 20 { if _t_vk[2] == 30 { emit "PASS vec_scale"; } else { emit "FAIL: vec_scale[2]"; }; } else { emit "FAIL: vec_scale[1]"; }; } else { emit "FAIL: vec_scale[0]"; };
let _t_vc = vec_cross([1,0,0], [0,1,0]); if _t_vc[0] == 0 { if _t_vc[1] == 0 { if _t_vc[2] == 1 { emit "PASS vec_cross"; } else { emit "FAIL: vec_cross[2]"; }; } else { emit "FAIL: vec_cross[1]"; }; } else { emit "FAIL: vec_cross[0]"; };
let _t_id = mat_identity(2); let _t_m2 = mat_from_rows([[1,2],[3,4]]); let _t_mr = mat_mul(_t_id, _t_m2); if _t_mr.data[0] == 1 { if _t_mr.data[1] == 2 { if _t_mr.data[2] == 3 { if _t_mr.data[3] == 4 { emit "PASS mat_mul identity"; } else { emit "FAIL: mat_mul[3]"; }; } else { emit "FAIL: mat_mul[2]"; }; } else { emit "FAIL: mat_mul[1]"; }; } else { emit "FAIL: mat_mul[0]"; };
let _t_di = mat_from_rows([[1,0],[0,1]]); if mat_det(_t_di) == 1 { emit "PASS mat_det identity"; } else { emit "FAIL: mat_det identity expected 1"; };
let _t_d2 = mat_from_rows([[2,3],[1,4]]); if mat_det(_t_d2) == 5 { emit "PASS mat_det 2x2"; } else { emit "FAIL: mat_det 2x2 expected 5"; };
let _t_mt = mat_from_rows([[1,2],[3,4]]); let _t_tt = mat_transpose(_t_mt); if _t_tt.data[0] == 1 { if _t_tt.data[1] == 3 { if _t_tt.data[2] == 2 { if _t_tt.data[3] == 4 { emit "PASS mat_transpose"; } else { emit "FAIL: mat_transpose[3]"; }; } else { emit "FAIL: mat_transpose[2]"; }; } else { emit "FAIL: mat_transpose[1]"; }; } else { emit "FAIL: mat_transpose[0]"; };
let _t_ma = mat_add(mat_from_rows([[1,2],[3,4]]), mat_from_rows([[5,6],[7,8]])); if _t_ma.data[0] == 6 { if _t_ma.data[3] == 12 { emit "PASS mat_add"; } else { emit "FAIL: mat_add[3]"; }; } else { emit "FAIL: mat_add[0]"; };
emit "matrix tests done";
