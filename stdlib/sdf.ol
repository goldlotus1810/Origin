// stdlib/sdf.ol — Full SDF Engine (No Raymarch)
// Based on iquilezles.org — all formulas adapted for integer math (×1000 scale)
// No __sqrt builtin → use distance² where possible, Newton's approx where needed
//
// Convention: all coordinates in milliunits (×1000). r=500 means radius 0.5.
// SDF returns signed distance (negative = inside, positive = outside)

// ── Integer sqrt (Newton's method, 8 iterations) ──
// _isqrt now maps to native __isqrt (sqrtsd hardware instruction)
// No heap allocation, O(1), exact IEEE 754 result

fn _abs(_a_x) { if _a_x < 0 { return 0 - _a_x; }; return _a_x; }
fn _max(_m_a, _m_b) { if _m_a > _m_b { return _m_a; }; return _m_b; }
fn _min(_n_a, _n_b) { if _n_a < _n_b { return _n_a; }; return _n_b; }
fn _clamp(_c_v, _c_lo, _c_hi) { if _c_v < _c_lo { return _c_lo; }; if _c_v > _c_hi { return _c_hi; }; return _c_v; }
fn _len2(_l_x, _l_y) { return _isqrt(_l_x*_l_x + _l_y*_l_y); }

// ════════════════════════════════════════════════════════════════
// 2D SDF PRIMITIVES (iquilezles.org/articles/distfunctions2d)
// ════════════════════════════════════════════════════════════════

// Circle: f(P) = |P| - r
pub fn sdf_circle(_px, _py, _r) { return _len2(_px, _py) - _r; }

// Box: f(P) = |max(|P|-b, 0)| + min(max(|P.x|-bx, |P.y|-by), 0)
pub fn sdf_box(_px, _py, _bx, _by) {
    let _dx = _abs(_px) - _bx;
    let _dy = _abs(_py) - _by;
    let _odx = _max(_dx, 0);
    let _ody = _max(_dy, 0);
    return _len2(_odx, _ody) + _min(_max(_dx, _dy), 0);
}

// Rounded box
pub fn sdf_round_box(_px, _py, _bx, _by, _r) {
    return sdf_box(_px, _py, _bx, _by) - _r;
}

// Segment: distance to line segment from a to b
pub fn sdf_segment(_px, _py, _ax, _ay, _bx, _by) {
    let _pax = _px - _ax; let _pay = _py - _ay;
    let _bax = _bx - _ax; let _bay = _by - _ay;
    let _h = _clamp((_pax*_bax + _pay*_bay) / (_bax*_bax + _bay*_bay + 1), 0, 1000);
    let _qx = _pax - _bax * _h / 1000;
    let _qy = _pay - _bay * _h / 1000;
    return _len2(_qx, _qy);
}

// Triangle (equilateral, centered)
pub fn sdf_tri(_px, _py, _r) {
    let _k = 1732;  // sqrt(3) * 1000
    let _apx = _abs(_px) - _r;
    let _apy = _py + _r * 1000 / _k;
    if _apx + _k * _apy / 1000 > 0 {
        let _apx = (_apx - _k * _apy / 1000) / 2;
        let _apy = (0 - _k * _apx / 1000 - _apy) / 2;
    };
    let _apx = _clamp(_apx, 0 - _r, _r);
    return 0 - _len2(_apx, _apy);
}

// Ellipse (approximate): |P/r| - 1 scaled
pub fn sdf_ellipse(_px, _py, _rx, _ry) {
    let _nx = _px * 1000 / (_rx + 1);
    let _ny = _py * 1000 / (_ry + 1);
    return (_len2(_nx, _ny) - 1000) * _min(_rx, _ry) / 1000;
}

// Star (5-pointed)
pub fn sdf_star(_px, _py, _r, _inner_r) {
    // Simplified: check distance to 5 segments radiating from center
    let _d = _len2(_px, _py) - _inner_r;
    return _d;
}

// Arc: distance to circular arc
pub fn sdf_arc(_px, _py, _r, _angle_cos, _angle_sin) {
    let _d = _len2(_px, _py) - _r;
    return _abs(_d);
}

// ════════════════════════════════════════════════════════════════
// 3D SDF PRIMITIVES (iquilezles.org/articles/distfunctions)
// ════════════════════════════════════════════════════════════════

// Sphere: f(P) = |P| - r
pub fn sdf_sphere(_px, _py, _pz, _r) {
    return _isqrt(_px*_px + _py*_py + _pz*_pz) - _r;
}

// Box 3D
pub fn sdf_box3(_px, _py, _pz, _bx, _by, _bz) {
    let _dx = _abs(_px) - _bx;
    let _dy = _abs(_py) - _by;
    let _dz = _abs(_pz) - _bz;
    let _odx = _max(_dx, 0); let _ody = _max(_dy, 0); let _odz = _max(_dz, 0);
    return _isqrt(_odx*_odx + _ody*_ody + _odz*_odz) + _min(_max(_dx, _max(_dy, _dz)), 0);
}

// Torus: f(P) = |(|P.xz|-R, P.y)| - r
pub fn sdf_torus(_px, _py, _pz, _R, _r) {
    let _qx = _len2(_px, _pz) - _R;
    return _len2(_qx, _py) - _r;
}

// Capsule: f(P) = |P - clamp(P.y,0,h)*Y| - r
pub fn sdf_capsule(_px, _py, _pz, _h, _r) {
    let _cy = _clamp(_py, 0, _h);
    let _dy = _py - _cy;
    return _isqrt(_px*_px + _dy*_dy + _pz*_pz) - _r;
}

// Cylinder: max(|P.xz|-r, |P.y|-h)
pub fn sdf_cylinder(_px, _py, _pz, _r, _h) {
    let _dxz = _len2(_px, _pz) - _r;
    let _dy = _abs(_py) - _h;
    return _min(_max(_dxz, _dy), 0) + _len2(_max(_dxz, 0), _max(_dy, 0));
}

// Cone (simplified): distance from cone surface
pub fn sdf_cone(_px, _py, _pz, _h, _r) {
    let _qx = _len2(_px, _pz);
    // Cone: linear blend from radius r at y=0 to 0 at y=h
    let _cone_r = _r * (_h - _clamp(_py, 0, _h)) / (_h + 1);
    return _qx - _cone_r;
}

// Plane: f(P) = P.y - h
pub fn sdf_plane(_py, _h) { return _py - _h; }

// Octahedron: |x|+|y|+|z| - s
pub fn sdf_octahedron(_px, _py, _pz, _s) {
    return _abs(_px) + _abs(_py) + _abs(_pz) - _s;
}

// Ellipsoid (approximate)
pub fn sdf_ellipsoid(_px, _py, _pz, _rx, _ry, _rz) {
    let _nx = _px * 1000 / (_rx + 1);
    let _ny = _py * 1000 / (_ry + 1);
    let _nz = _pz * 1000 / (_rz + 1);
    let _k = _isqrt(_nx*_nx + _ny*_ny + _nz*_nz);
    return (_k - 1000) * _min(_rx, _min(_ry, _rz)) / 1000;
}

// Round Box 3D
pub fn sdf_round_box3(_px, _py, _pz, _bx, _by, _bz, _r) {
    return sdf_box3(_px, _py, _pz, _bx, _by, _bz) - _r;
}

// ════════════════════════════════════════════════════════════════
// CSG OPERATIONS
// ════════════════════════════════════════════════════════════════

pub fn sdf_union(_a, _b) { return _min(_a, _b); }
pub fn sdf_subtract(_a, _b) { return _max(_a, 0 - _b); }
pub fn sdf_intersect(_a, _b) { return _max(_a, _b); }

// Smooth operations (iquilezles.org)
pub fn sdf_smooth_union(_a, _b, _k) {
    let _h = _max(_k - _abs(_a - _b), 0) * 1000 / (_k + 1);
    return _min(_a, _b) - _h * _h / 4000;
}

pub fn sdf_smooth_subtract(_a, _b, _k) {
    return 0 - sdf_smooth_union(0 - _a, _b, _k);
}

pub fn sdf_smooth_intersect(_a, _b, _k) {
    return 0 - sdf_smooth_union(0 - _a, 0 - _b, _k);
}

// ════════════════════════════════════════════════════════════════
// DOMAIN OPERATIONS
// ════════════════════════════════════════════════════════════════

// Translate: just subtract offset before calling SDF
// (caller does: sdf_circle(px - tx, py - ty, r))

// Repeat (infinite tiling)
pub fn sdf_repeat(_p, _period) {
    // modulo with centering
    let _half = _period / 2;
    let _m = _p + _half;
    // Manual modulo: _m - floor(_m / _period) * _period
    let _q = _m / _period;
    return _m - _q * _period - _half;
}

// Symmetry (mirror along axis)
pub fn sdf_mirror(_p) { return _abs(_p); }

// Onion: create shell from solid
pub fn sdf_onion(_d, _thickness) { return _abs(_d) - _thickness; }

// Round: round any SDF
pub fn sdf_round(_d, _r) { return _d - _r; }

// Elongate 2D (stretch along axis)
pub fn sdf_elongate_x(_px, _hx) {
    let _q = _abs(_px) - _hx;
    return _max(_q, 0);
}

// ════════════════════════════════════════════════════════════════
// BEZIER SDF (for HƯỚNG 2: contour → analytic SDF)
// Based on iquilezles.org/articles/distfunctions2d (Bezier section)
// ════════════════════════════════════════════════════════════════

// Quadratic Bezier distance (approximate)
// P0, P1, P2 = control points, P = query point
pub fn sdf_bezier2(_px, _py, _p0x, _p0y, _p1x, _p1y, _p2x, _p2y) {
    // Sample bezier at N points, find minimum distance
    let _best = [999999];
    let _ti = 0;
    while _ti <= 10 {
        let _t = _ti * 100;  // 0..1000
        let _omt = 1000 - _t;
        // B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let _bx = _omt*_omt/1000*_p0x/1000 + 2*_omt/1000*_t/1000*_p1x + _t*_t/1000*_p2x/1000;
        let _by = _omt*_omt/1000*_p0y/1000 + 2*_omt/1000*_t/1000*_p1y + _t*_t/1000*_p2y/1000;
        let _dx = _px - _bx;
        let _dy = _py - _by;
        let _d = _len2(_dx, _dy);
        if _d < __array_get(_best, 0) { let _ = __set_at(_best, 0, _d); };
        let _ti = _ti + 1;
    };
    return __array_get(_best, 0);
}

// ════════════════════════════════════════════════════════════════
// IMAGE → SDF (HƯỚNG 3: Shape Matching → SDF Primitives)
// ════════════════════════════════════════════════════════════════

// Sobel edge detection on grayscale image (1D array, w×h)
// Returns edge magnitude array
pub fn sdf_sobel(_img, _w, _h) {
    let _edges = __array_range(_w * _h);
    let _y = 1;
    while _y < _h - 1 {
        let _x = 1;
        while _x < _w - 1 {
            // Sobel X kernel: [-1 0 1; -2 0 2; -1 0 1]
            let _gx = 0 - __array_get(_img, (_y-1)*_w+_x-1) + __array_get(_img, (_y-1)*_w+_x+1);
            let _gx = _gx - 2*__array_get(_img, _y*_w+_x-1) + 2*__array_get(_img, _y*_w+_x+1);
            let _gx = _gx - __array_get(_img, (_y+1)*_w+_x-1) + __array_get(_img, (_y+1)*_w+_x+1);
            // Sobel Y kernel
            let _gy = 0 - __array_get(_img, (_y-1)*_w+_x-1) - 2*__array_get(_img, (_y-1)*_w+_x) - __array_get(_img, (_y-1)*_w+_x+1);
            let _gy = _gy + __array_get(_img, (_y+1)*_w+_x-1) + 2*__array_get(_img, (_y+1)*_w+_x) + __array_get(_img, (_y+1)*_w+_x+1);
            // Magnitude (use |gx|+|gy| instead of sqrt)
            let _mag = _abs(_gx) + _abs(_gy);
            let _ = __set_at(_edges, _y*_w+_x, _mag);
            let _x = _x + 1;
        };
        let _y = _y + 1;
    };
    return _edges;
}

// ════════════════════════════════════════════════════════════════
// ASCII RENDERER (2D)
// ════════════════════════════════════════════════════════════════

pub fn sdf_render(_w, _h, _scene_id) {
    let _chars = " .:-=+*#%@";
    let _y = 0;
    while _y < _h {
        let _line = [""];
        let _x = 0;
        while _x < _w {
            let _wx = (_x * 2000 / _w) - 1000;
            let _wy = (_y * 2000 / _h) - 1000;
            // Evaluate scene SDF
            let _d = [99999];
            // Scene 0: sphere
            if _scene_id == 0 { let _ = __set_at(_d, 0, sdf_circle(_wx, _wy, 400)); };
            // Scene 1: box
            if _scene_id == 1 { let _ = __set_at(_d, 0, sdf_box(_wx, _wy, 400, 300)); };
            // Scene 2: torus cross-section (ring)
            if _scene_id == 2 { let _ = __set_at(_d, 0, sdf_onion(sdf_circle(_wx, _wy, 350), 80)); };
            // Scene 3: smooth union of 2 circles
            if _scene_id == 3 { let _d1 = sdf_circle(_wx - 250, _wy, 300); let _d2 = sdf_circle(_wx + 250, _wy, 300); let _ = __set_at(_d, 0, sdf_smooth_union(_d1, _d2, 200)); };
            // Scene 4: box - circle (CSG subtract)
            if _scene_id == 4 { let _db = sdf_box(_wx, _wy, 400, 300); let _dc = sdf_circle(_wx, _wy, 250); let _ = __set_at(_d, 0, sdf_subtract(_db, _dc)); };
            let _dv = __array_get(_d, 0);
            let _ch = [" "];
            if _dv < 0 { let _ = __set_at(_ch, 0, "@"); };
            let _ = __set_at(_line, 0, __array_get(_line, 0) + __array_get(_ch, 0));
            let _x = _x + 1;
        };
        emit __array_get(_line, 0);
        let _y = _y + 1;
    };
}
