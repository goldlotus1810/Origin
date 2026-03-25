// Test: Molecule pack/unpack — 5D encoding [S:4][R:4][V:3][A:3][T:2]

let ok = 1;

// Pack: S=5, R=3, V=4, A=2, T=1
// Expected: 5*4096 + 3*256 + 4*32 + 2*4 + 1 = 20480 + 768 + 128 + 8 + 1 = 21385
let m = __mol_pack(5, 3, 4, 2, 1);
if m != 21385 { let ok = 0; emit "FAIL pack"; emit m; };

// Extract each dimension
if __mol_s(m) != 5 { let ok = 0; emit "FAIL S"; };
if __mol_r(m) != 3 { let ok = 0; emit "FAIL R"; };
if __mol_v(m) != 4 { let ok = 0; emit "FAIL V"; };
if __mol_a(m) != 2 { let ok = 0; emit "FAIL A"; };
if __mol_t(m) != 1 { let ok = 0; emit "FAIL T"; };

// Max values: S=15, R=15, V=7, A=7, T=3
let max_mol = __mol_pack(15, 15, 7, 7, 3);
if __mol_s(max_mol) != 15 { let ok = 0; emit "FAIL max S"; };
if __mol_r(max_mol) != 15 { let ok = 0; emit "FAIL max R"; };
if __mol_v(max_mol) != 7 { let ok = 0; emit "FAIL max V"; };
if __mol_a(max_mol) != 7 { let ok = 0; emit "FAIL max A"; };
if __mol_t(max_mol) != 3 { let ok = 0; emit "FAIL max T"; };

// Min values: all zeros
let zero_mol = __mol_pack(0, 0, 0, 0, 0);
if zero_mol != 0 { let ok = 0; emit "FAIL zero"; };

if ok { emit "PASS"; } else { emit "FAIL"; };
