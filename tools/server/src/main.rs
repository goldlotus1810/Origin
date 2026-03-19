//! # server — HomeOS REPL
//!
//! Terminal REPL cho HomeOS.
//! Dùng stdin/stdout — không cần framework.
//!
//! ○(∅) == ○ — boot từ hư không.

use std::fs::OpenOptions;
use std::io::{self, BufRead, Read, Write};

use runtime::origin::{now_ns, HomeRuntime};

const OLANG_FILE: &str = "origin.olang";

// ─────────────────────────────────────────────────────────────────────────────
// Boot sequence:
//   Phase 1: START    — ○(∅) == ○, UCD check
//   Phase 2: BOOT     — load origin.olang file bytes, validate/recover
//   Phase 3: LOAD     — registry + QT axioms → HomeRuntime
//   Phase 4: SETUP    — HAL detect, manifest scan, agent inventory
//   Phase 5: RUN      — REPL loop (or --eval mode)
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let eval_mode = args.contains(&"--eval".to_string());

    if eval_mode {
        run_eval();
        return;
    }
    // ══════════════════════════════════════════════════════════════════════════
    // Phase 1: START — ○(∅) == ○
    // ══════════════════════════════════════════════════════════════════════════
    println!("HomeOS ○");
    println!();

    let session_id = now_ns() as u64;

    // ══════════════════════════════════════════════════════════════════════════
    // Phase 2: BOOT origin.olang — load file bytes
    // ══════════════════════════════════════════════════════════════════════════
    let file_bytes = std::fs::read(OLANG_FILE).ok();
    if let Some(ref bytes) = &file_bytes {
        println!("[boot] origin.olang: {} bytes", bytes.len());
        // Validate — best-effort recovery nếu corrupt
        match olang::reader::OlangReader::new(bytes) {
            Ok(reader) => {
                let (_pf, info) = reader.parse_recoverable();
                if let Some(ref err) = info.error {
                    eprintln!(
                        "[boot] WARNING: corrupt at byte {}: {:?}",
                        info.last_good_offset, err
                    );
                    eprintln!(
                        "[boot] Recovered {} records / {} bytes",
                        info.records_recovered, info.total_bytes
                    );
                }
            }
            Err(e) => {
                eprintln!("[boot] WARNING: invalid header: {:?}", e);
                eprintln!("[boot] Preserved as origin.olang.bak");
                let _ = std::fs::copy(OLANG_FILE, "origin.olang.bak");
            }
        }
    } else {
        println!("[boot] No origin.olang — booting from nothing");
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Phase 3: LOAD registry + QT axioms
    // ══════════════════════════════════════════════════════════════════════════
    let desktop_bridge = Box::new(hal::ffi::DesktopBridge::new());
    let mut rt = if let Some(ref bytes) = file_bytes {
        HomeRuntime::with_platform(session_id, Some(bytes), desktop_bridge)
    } else {
        HomeRuntime::with_platform(session_id, None, desktop_bridge)
    };

    println!(
        "[load] Registry: {} nodes, {} aliases · Stage {:?}",
        rt.registry_len(),
        rt.registry_alias_count(),
        rt.boot_stage(),
    );
    for err in rt.boot_errors() {
        eprintln!("[load] ERROR: {}", err);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Phase 4: SETUP — HAL detect + manifest scan + L0 self-awareness
    // ══════════════════════════════════════════════════════════════════════════
    let arch = hal::Architecture::detect();
    let tier = hal::HardwareTier::from_arch(arch);
    println!(
        "[setup] {} · {} · UCD {}",
        arch.name(),
        tier.summary(),
        ucd::table_len(),
    );
    println!("[setup] {}", rt.manifest().summary());
    println!(
        "[setup] Agents: {} chiefs, {} workers",
        rt.chief_count(),
        rt.worker_count(),
    );

    // ── L0 Self-Awareness — hệ thống nhìn thấy chính mình ───────────────
    println!();
    println!("○ L0 — Bảng tuần hoàn");
    println!("  Atoms : {} nguyên tố (5 chiều)", rt.registry_len());
    // Count groups in UCD
    let mut sdf = 0usize;
    let mut math = 0usize;
    let mut emo = 0usize;
    let mut mus = 0usize;
    for entry in ucd::table() {
        match entry.group {
            0x01 => sdf += 1,
            0x02 => math += 1,
            0x03 => emo += 1,
            0x04 => mus += 1,
            _ => {}
        }
    }
    println!("  Shape : {} (SDF — trông như thế nào)", sdf);
    println!("  Relate: {} (MATH — liên kết thế nào)", math);
    println!("  Feel  : {} (EMOTICON — cảm thế nào)", emo);
    println!("  Time  : {} (MUSICAL — thay đổi thế nào)", mus);
    println!("  DNA   : Skills={} Agents={} Programs={}",
        rt.manifest().count_by_category(olang::startup::NodeCategory::Skill),
        rt.manifest().count_by_category(olang::startup::NodeCategory::Agent),
        0, // Programs counted via manifest
    );
    println!("  ○ Tôi biết {} hình dạng, {} quan hệ, {} cảm xúc, {} nhịp thời gian",
        sdf, math, emo, mus);

    // QT8: Flush boot pending_writes → origin.olang TRƯỚC khi chạy
    // Đảm bảo mọi seed/agent records có trong file
    flush_pending(&mut rt);

    // L0 Integrity Check — phát hiện node RAM-only chưa ghi file
    let file_bytes_after = std::fs::read(OLANG_FILE).ok();
    let violations = rt.integrity_check(
        file_bytes_after.as_deref(),
    );
    if violations.is_empty() {
        println!("[setup] QT8 integrity: OK — all nodes in origin.olang");
    } else {
        eprintln!("[setup] QT8 integrity: {} violations", violations.len());
        for v in &violations {
            eprintln!("  {}", v);
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Phase 5: RUN — REPL loop
    // ══════════════════════════════════════════════════════════════════════════
    println!();
    println!("Type text to chat · ○{{help}} for commands · Ctrl+C to exit");
    println!();

    let stdin = io::stdin();
    let stdout = io::stdout();

    loop {
        // Prompt
        {
            let mut out = stdout.lock();
            write!(out, "○ ").unwrap();
            out.flush().unwrap();
        }

        // Read line
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        // ── Olang file execution: `olang <filename>` ─────────────────────
        let response;
        let ts = now_ns();
        if let Some(filename) = input.strip_prefix("olang ") {
            let filename = filename.trim();
            match std::fs::read_to_string(filename) {
                Ok(source) => {
                    response = rt.run_program(&source, ts);
                }
                Err(e) => {
                    println!("[error] Cannot read '{}': {}", filename, e);
                    continue;
                }
            }
        }
        // ── Inline Olang program: lines starting with `>` ───────────────
        else if input.starts_with("> ") || input.starts_with(">") {
            let source = input.strip_prefix("> ").unwrap_or(
                input.strip_prefix(">").unwrap_or(input)
            );
            response = rt.run_program(source, ts);
        }
        // ── Normal text processing ──────────────────────────────────────
        else {
            response = rt.process_text(input, ts);
        }

        // Output — silent responses (SilentAck, Observe) → không in gì
        if !response.text.is_empty() {
            println!("{}", response.text);
            println!();
        }

        // Flush pending writes → origin.olang (QT9: ghi file TRƯỚC)
        flush_pending(&mut rt);

        // RegistryGate: drain + display notifications
        let notifs = rt.drain_registry_notifications();
        for n in &notifs {
            eprintln!("[gate] {}", n);
        }
    }

    // Final persist: serialize remaining learned data
    let final_bytes = rt.serialize_learned(now_ns());
    if !final_bytes.is_empty() {
        if let Err(e) = append_to_file(OLANG_FILE, &final_bytes) {
            eprintln!("[persist] Error writing final state: {}", e);
        } else {
            println!("[persist] Saved {} bytes on exit", final_bytes.len());
        }
    }

    // Stats khi thoát
    println!();
    println!(
        "○ Session ended · {} turns · f(x)={:.3}",
        rt.turn_count(),
        rt.fx()
    );
}

/// --eval mode: đọc stdin → process → output → exit.
/// Không banner, không REPL prompt, không persist.
/// Dùng cho scripting và automated testing.
fn run_eval() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("failed to read stdin");

    let session_id = now_ns() as u64;
    let file_bytes = std::fs::read(OLANG_FILE).ok();
    let mut rt = if let Some(ref bytes) = file_bytes {
        HomeRuntime::with_file(session_id, Some(bytes))
    } else {
        HomeRuntime::new(session_id)
    };

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let ts = now_ns();
        let response = if let Some(filename) = trimmed.strip_prefix("olang ") {
            match std::fs::read_to_string(filename.trim()) {
                Ok(source) => rt.run_program(&source, ts),
                Err(e) => {
                    eprintln!("[error] Cannot read '{}': {}", filename.trim(), e);
                    continue;
                }
            }
        } else if trimmed.starts_with("> ") || trimmed.starts_with('>') {
            let source = trimmed.strip_prefix("> ").unwrap_or(
                trimmed.strip_prefix('>').unwrap_or(trimmed),
            );
            rt.run_program(source, ts)
        } else {
            rt.process_text(trimmed, ts)
        };

        if !response.text.is_empty() {
            println!("{}", response.text);
        }
    }
}

/// Flush pending writes từ runtime → origin.olang.
fn flush_pending(rt: &mut HomeRuntime) {
    if !rt.has_pending_writes() {
        return;
    }

    let bytes = rt.drain_pending_writes();
    if bytes.is_empty() {
        return;
    }

    match append_to_file(OLANG_FILE, &bytes) {
        Ok(()) => {
            // Silent success — không spam console
        }
        Err(e) => {
            eprintln!("[persist] Error flushing {} bytes: {}", bytes.len(), e);
        }
    }
}

/// Append bytes vào file (tạo mới nếu chưa có).
fn append_to_file(path: &str, bytes: &[u8]) -> io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::origin::{now_ns, HomeRuntime};

    #[test]
    fn olang_file_constant() {
        assert_eq!(OLANG_FILE, "origin.olang");
    }

    #[test]
    fn append_to_file_creates_and_appends() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("homeos_test_{}.bin", now_ns()));
        let path_str = path.to_str().unwrap();

        // First write creates the file
        append_to_file(path_str, b"hello").unwrap();
        assert_eq!(std::fs::read(path_str).unwrap(), b"hello");

        // Second write appends
        append_to_file(path_str, b" world").unwrap();
        assert_eq!(std::fs::read(path_str).unwrap(), b"hello world");

        // Cleanup
        let _ = std::fs::remove_file(path_str);
    }

    #[test]
    fn append_to_file_empty_bytes() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("homeos_test_empty_{}.bin", now_ns()));
        let path_str = path.to_str().unwrap();

        append_to_file(path_str, b"").unwrap();
        assert_eq!(std::fs::read(path_str).unwrap().len(), 0);

        let _ = std::fs::remove_file(path_str);
    }

    #[test]
    fn runtime_boots_from_nothing() {
        let rt = HomeRuntime::new(12345);
        assert_eq!(rt.turn_count(), 0);
        // QT8: fresh boot has seed writes for origin.olang
        assert!(rt.has_pending_writes());
    }

    #[test]
    fn runtime_fx_starts_at_zero() {
        let rt = HomeRuntime::new(99999);
        // f(x) should start at 0 or near 0 with no turns
        assert!(rt.fx().abs() < 0.01);
    }

    #[test]
    fn runtime_process_text_increments_turn() {
        let mut rt = HomeRuntime::new(11111);
        let ts = now_ns();
        let _resp = rt.process_text("hello", ts);
        assert!(rt.turn_count() >= 1);
    }

    #[test]
    fn runtime_with_invalid_file_still_boots() {
        // Passing garbage bytes — runtime should handle gracefully
        let garbage = vec![0u8; 50];
        let rt = HomeRuntime::with_file(22222, Some(&garbage));
        assert_eq!(rt.turn_count(), 0);
    }

    #[test]
    fn runtime_serialize_learned_empty() {
        let rt = HomeRuntime::new(33333);
        let bytes = rt.serialize_learned(now_ns());
        // No turns processed, so serialized data may be empty or just header
        // The key thing is it does not panic
        let _ = bytes;
    }

    #[test]
    fn runtime_drain_pending_writes() {
        let mut rt = HomeRuntime::new(44444);
        // QT8: fresh boot has seed writes → drain them
        assert!(rt.has_pending_writes());
        let drained = rt.drain_pending_writes();
        assert!(!drained.is_empty());
        // After drain → empty
        assert!(!rt.has_pending_writes());
        let drained2 = rt.drain_pending_writes();
        assert!(drained2.is_empty());
    }

    #[test]
    fn flush_pending_noop_when_empty() {
        let mut rt = HomeRuntime::new(55555);
        // Should not panic or error when there's nothing to flush
        flush_pending(&mut rt);
    }

    #[test]
    fn exit_quit_commands_recognized() {
        // Verify the trim + comparison logic used in the REPL
        for cmd in &["exit", "quit"] {
            let line = format!("  {}  ", cmd);
            let input = line.trim();
            assert!(input == "exit" || input == "quit");
        }
    }

    #[test]
    fn empty_input_skipped() {
        // Verify that empty/whitespace input would be skipped
        for input in &["", "   ", "\t", "\n"] {
            assert!(input.trim().is_empty());
        }
    }

    #[test]
    fn boot_info_accessible() {
        let rt = HomeRuntime::new(66666);
        // Boot stage should be at least UcdReady when UCD table is present
        let _stage = rt.boot_stage();
        // Manifest should have entries after boot
        let manifest = rt.manifest();
        let _summary = manifest.summary();
        // Registry should have nodes (axioms + L1 system)
        assert!(rt.registry_len() > 0);
        assert!(rt.registry_alias_count() > 0);
        // No errors on clean boot
        assert!(rt.boot_errors().is_empty());
    }

    #[test]
    fn eval_flag_detected() {
        // --eval should be recognized in args
        let args = vec!["server".to_string(), "--eval".to_string()];
        assert!(args.contains(&"--eval".to_string()));
    }

    #[test]
    fn eval_flag_not_present() {
        let args = vec!["server".to_string()];
        assert!(!args.contains(&"--eval".to_string()));
    }
}
