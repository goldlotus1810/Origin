// stdlib/homeos/terminal.ol — Terminal animation engine
//
// Uses __bytes_write("/dev/stdout") for ANSI escapes + __sleep(ms) for timing.
// ANSI control built byte-by-byte (Olang lexer has no \x escape).
//
// Animation frames use Unicode circle symbols from docs/O_SYMBOL_DN.txt:
//   ○ ◯ 〇 ◦ ⋅ ◌ ● ◉ ⦿ ◎ ⊚ ◐ ◑ ◒ ◓ ◔ ◕ ◜ ◝ ◟ ◞ ◠ ◡
//

// ── ANSI escape byte builder ──────────────────────────────────────────
// Since Olang lexer has no \x escape, we build ANSI strings via bytes.

// Write raw bytes to stdout: [ESC, '[', ...ascii codes...]
fn _term_write_esc(codes) {
    let _tw_buf = __bytes_new(64);
    __bytes_set(_tw_buf, 0, 27);    // ESC = 0x1B
    __bytes_set(_tw_buf, 1, 91);    // '[' = 0x5B
    let _tw_i = 2;
    let _tw_ci = 0;
    while _tw_ci < len(codes) {
        let _tw_c = codes[_tw_ci];
        __bytes_set(_tw_buf, _tw_i, _tw_c);
        let _tw_i = _tw_i + 1;
        let _tw_ci = _tw_ci + 1;
    };
    // Now write _tw_i bytes to stdout via syscall
    __bytes_write("/dev/stdout", _tw_buf, _tw_i);
};

// ── Cursor control ────────────────────────────────────────────────────

// Hide cursor: ESC[?25l
fn cursor_hide() {
    _term_write_esc([63, 50, 53, 108]);  // ?25l
};

// Show cursor: ESC[?25h
fn cursor_show() {
    _term_write_esc([63, 50, 53, 104]);  // ?25h
};

// Move cursor to column 1: ESC[G
fn cursor_home() {
    _term_write_esc([71]);  // G
};

// Clear line: ESC[2K
fn clear_line() {
    _term_write_esc([50, 75]);  // 2K
};

// Move up N lines: ESC[<N>A
fn cursor_up(n) {
    if n < 10 {
        _term_write_esc([48 + n, 65]);  // <digit>A
    };
};

// ── Color control ─────────────────────────────────────────────────────

// Set foreground color (30-37, 90-97): ESC[<code>m
fn fg_color(code) {
    // Convert number to ASCII digits
    let _fc_d1 = __floor(code / 10);
    let _fc_d0 = code - _fc_d1 * 10;
    _term_write_esc([48 + _fc_d1, 48 + _fc_d0, 109]);  // <d1><d0>m
};

// Reset all attributes: ESC[0m
fn fg_reset() {
    _term_write_esc([48, 109]);  // 0m
};

// Bold: ESC[1m
fn text_bold() {
    _term_write_esc([49, 109]);  // 1m
};

// Dim: ESC[2m
fn text_dim() {
    _term_write_esc([50, 109]);  // 2m
};

// ── Animation frame definitions ───────────────────────────────────────
// Based on docs/O_SYMBOL_DN.txt — circle-based status animations

// WORKING: breathing circle ○ ◯ 〇 ○ ◦ ⋅ ◦ ○ ◯ 〇 ◌ ⋅ ◦ ○
fn anim_working() {
    return ["○", "◯", "〇", "◯", "○", "◦", "⋅", "◦", "○", "◯", "〇", "◌", "⋅", "◦", "○"];
};

// LOADING: fill circle ○ ◯ ◔ ◑ ◕ ● ◉ ⦿ ◉ ◎ ⊚ ○
fn anim_loading() {
    return ["○", "◯", "◔", "◑", "◕", "●", "◉", "⬤", "⦿", "◉", "◎", "⊚", "○"];
};

// SEARCHING: radar sweep ○ ◜ ◠ ◝ ◞ ◡ ◟ ○
fn anim_searching() {
    return ["○", "◜", "◠", "◝", "◞", "◡", "◟", "○"];
};

// SUCCESS: radiant completion ○ ◯ ⦿ ❂ ⭕ ❂ ⦿ ◯ ○
fn anim_success() {
    return ["○", "◯", "⦿", "❂", "⭕", "❂", "⦿", "◯", "○"];
};

// ERROR: shake ○ ⧲ ⧳ ⊗ ⨷ ⊗ ⧳ ⧲ ○
fn anim_error() {
    return ["○", "⧲", "⧳", "⊗", "⨷", "⊗", "⧳", "⧲", "○"];
};

// WARNING: blink ○ ◌ ⊘ ☢ ⊘ ☢ ⊘ ◌ ○
fn anim_warning() {
    return ["○", "◌", "⊘", "☢", "⊘", "☢", "⊘", "◌", "○"];
};

// DOWNLOAD: flow down ○ ◌ ⧬ ◒ ⧭ ● ⧭ ◒ ⧬ ◌ ○
fn anim_download() {
    return ["○", "◌", "⧬", "◒", "⧭", "●", "⧭", "◒", "⧬", "◌", "○"];
};

// DATA IN: ○ ⇠ ◌ ⧂ ⦿ ◉ ⧂ ◌ ⇠ ○
fn anim_data_in() {
    return ["○", "⇠", "◌", "⧂", "⦿", "◉", "⧂", "◌", "⇠", "○"];
};

// DATA OUT: ○ ⇢ ⧃ ⦿ ◉ ⧃ ⇢ ○
fn anim_data_out() {
    return ["○", "⇢", "⧃", "⦿", "◉", "⧃", "⇢", "○"];
};

// DEBUGGING: scan ○ ⊚ ⦺ ⦹ ⨸ ⦹ ⦺ ⊚ ○
fn anim_debugging() {
    return ["○", "⊚", "⦺", "⦹", "⨸", "⦹", "⦺", "⊚", "○"];
};

// PAUSE: rest ○ ⊜ ◌ ◌ ⊜ ○
fn anim_pause() {
    return ["○", "⊜", "◌", "◌", "⊜", "○"];
};

// ── Animation runner ──────────────────────────────────────────────────

// Play animation once: show each frame with delay_ms between
fn anim_play(frames, delay_ms) {
    cursor_hide();
    let _ap_i = 0;
    while _ap_i < len(frames) {
        clear_line();
        cursor_home();
        emit frames[_ap_i];
        __sleep(delay_ms);
        let _ap_i = _ap_i + 1;
    };
    cursor_show();
};

// Play animation N loops with label
fn anim_loop(label, frames, delay_ms, loops) {
    cursor_hide();
    let _al_loop = 0;
    while _al_loop < loops {
        let _al_i = 0;
        while _al_i < len(frames) {
            clear_line();
            cursor_home();
            emit $"[ {label} ] : {frames[_al_i]}";
            __sleep(delay_ms);
            let _al_i = _al_i + 1;
        };
        let _al_loop = _al_loop + 1;
    };
    cursor_show();
};

// ── Status bar (multi-line dashboard) ─────────────────────────────────

// Show HomeOS status dashboard (one frame cycle)
fn status_dashboard(frame_idx) {
    let _sd_work = anim_working();
    let _sd_search = anim_searching();
    let _sd_load = anim_loading();
    let _sd_debug = anim_debugging();

    let _sd_w = _sd_work[frame_idx % len(_sd_work)];
    let _sd_s = _sd_search[frame_idx % len(_sd_search)];
    let _sd_l = _sd_load[frame_idx % len(_sd_load)];
    let _sd_d = _sd_debug[frame_idx % len(_sd_debug)];

    clear_line();
    cursor_home();
    emit $"  CORE KERNEL  : {_sd_w}  [ STABLE    ]";
    emit $"  DATA SCANNER : {_sd_s}  [ SEARCHING ]";
    emit $"  LOADING OS   : {_sd_l}  [ PROGRESS  ]";
    emit $"  ENCRYPTION   : {_sd_d}  [ DEBUGGING ]";
};

// ── Spline animation (T dimension) ───────────────────────────────────
// Render T spline as animated circle size:
//   amplitude low  → ⋅ (tiny)
//   amplitude mid  → ○ (normal)
//   amplitude high → 〇 (large)
//   amplitude max  → ⬤ (filled)

fn spline_frame(amplitude) {
    if amplitude < 0.15 { return "⋅"; };
    if amplitude < 0.3  { return "◦"; };
    if amplitude < 0.5  { return "○"; };
    if amplitude < 0.7  { return "◯"; };
    if amplitude < 0.85 { return "〇"; };
    return "⬤";
};

// Animate a spline curve: array of amplitude values [0..1]
fn anim_spline(amplitudes, delay_ms) {
    cursor_hide();
    let _as_i = 0;
    while _as_i < len(amplitudes) {
        clear_line();
        cursor_home();
        let _as_a = amplitudes[_as_i];
        let _as_f = spline_frame(_as_a);
        emit $"  T spline [{_as_i}]: {_as_f}  amp={_as_a}";
        __sleep(delay_ms);
        let _as_i = _as_i + 1;
    };
    cursor_show();
};

// Generate sine wave amplitudes for rhythmic T pattern
fn gen_sine_frames(count, frequency) {
    let _gs_frames = [];
    let _gs_i = 0;
    while _gs_i < count {
        // sin approximation: Taylor series sin(x) ≈ x - x³/6 + x⁵/120
        let _gs_x = _gs_i * frequency * 6.2832 / count;  // 2*pi*f*t/N
        // Normalize x to [-pi, pi] range
        let _gs_x2 = _gs_x - __floor(_gs_x / 6.2832) * 6.2832;
        if _gs_x2 > 3.1416 { let _gs_x2 = _gs_x2 - 6.2832; };
        // Taylor: sin(x) ≈ x - x³/6 + x⁵/120
        let _gs_x3 = _gs_x2 * _gs_x2 * _gs_x2;
        let _gs_x5 = _gs_x3 * _gs_x2 * _gs_x2;
        let _gs_sin = _gs_x2 - _gs_x3 / 6 + _gs_x5 / 120;
        // Map [-1,1] → [0,1]
        let _gs_amp = (_gs_sin + 1) / 2;
        push(_gs_frames, _gs_amp);
        let _gs_i = _gs_i + 1;
    };
    return _gs_frames;
};

// ── Demo ──────────────────────────────────────────────────────────────

fn demo_animations() {
    emit "═══ HomeOS Terminal Animation Demo ═══";
    emit "";

    emit "① WORKING (breathing):";
    anim_play(anim_working(), 120);
    emit "";

    emit "② LOADING (fill):";
    anim_play(anim_loading(), 100);
    emit "";

    emit "③ SEARCHING (radar):";
    anim_play(anim_searching(), 150);
    emit "";

    emit "④ SUCCESS:";
    anim_play(anim_success(), 130);
    emit "";

    emit "⑤ ERROR:";
    anim_play(anim_error(), 100);
    emit "";

    emit "⑥ T SPLINE (sine wave, freq=2):";
    let frames = gen_sine_frames(40, 2);
    anim_spline(frames, 80);
    emit "";

    emit "═══ Done ═══";
};
