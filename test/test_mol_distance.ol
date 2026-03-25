// Test: Molecule distance — 5D Euclidean similarity
// distance = sqrt(dS^2 + dR^2 + dV^2 + dA^2 + dT^2) normalized
// similarity = 1.0 - distance, clamped [0, 1]

let ok = 1;

// Same molecule = distance 0, similarity 1
let m1 = __mol_pack(5, 3, 4, 2, 1);
let m2 = __mol_pack(5, 3, 4, 2, 1);

fn mol_dist(a, b) {
    let ds = (__mol_s(a) - __mol_s(b)) / 15;
    let dr = (__mol_r(a) - __mol_r(b)) / 15;
    let dv = (__mol_v(a) - __mol_v(b)) / 7;
    let da = (__mol_a(a) - __mol_a(b)) / 7;
    let dt = (__mol_t(a) - __mol_t(b)) / 3;
    let sum = ds*ds + dr*dr + dv*dv + da*da + dt*dt;
    return sum;
};

fn mol_sim(a, b) {
    let d = mol_dist(a, b);
    let s = 1 - d;
    if s < 0 { return 0; };
    return s;
};

// Identical = similarity ~1.0
let sim_same = mol_sim(m1, m2);
if sim_same != 1 { let ok = 0; emit "FAIL same mol sim"; emit sim_same; };

// Opposite extremes = low similarity
let m_low = __mol_pack(0, 0, 0, 0, 0);
let m_high = __mol_pack(15, 15, 7, 7, 3);
let dist_opp = mol_dist(m_low, m_high);
// Distance should be large (>= 3)
if dist_opp < 3 { let ok = 0; emit "FAIL opposite not distant enough"; emit dist_opp; };

// Close molecules = high similarity
let m_a = __mol_pack(5, 3, 4, 2, 1);
let m_b = __mol_pack(6, 3, 4, 2, 1);  // only S differs by 1
let sim_close = mol_sim(m_a, m_b);
if sim_close < 0.9 { let ok = 0; emit "FAIL close not similar enough"; emit sim_close; };

if ok { emit "PASS"; } else { emit "FAIL"; };
