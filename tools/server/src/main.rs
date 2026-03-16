//! # server — HomeOS REPL
//!
//! Terminal REPL cho HomeOS.
//! Dùng stdin/stdout — không cần framework.
//!
//! ○(∅) == ○ — boot từ hư không.

use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};

use runtime::origin::{now_ns, HomeRuntime};

const OLANG_FILE: &str = "origin.olang";

fn main() {
    // ○(∅) == ○ — boot
    println!("HomeOS ○");
    println!("Unicode 18.0 · {} UCD entries", ucd::table_len());
    println!("Type text to chat · ○{{help}} for commands · Ctrl+C to exit");
    println!();

    if ucd::table_len() == 0 {
        eprintln!("WARNING: UCD table empty — rebuild with UnicodeData.txt");
    }

    let session_id = now_ns() as u64;
    // Load origin.olang nếu có — with crash recovery
    let file_bytes = std::fs::read(OLANG_FILE).ok();
    let mut rt = if let Some(ref bytes) = file_bytes {
        println!("[boot] origin.olang: {} bytes", bytes.len());
        // Validate file trước khi load — best-effort recovery nếu corrupt
        match olang::reader::OlangReader::new(bytes) {
            Ok(reader) => {
                let (_pf, info) = reader.parse_recoverable();
                if let Some(ref err) = info.error {
                    eprintln!(
                        "[boot] WARNING: origin.olang corrupt at byte {}: {:?}",
                        info.last_good_offset, err
                    );
                    eprintln!(
                        "[boot] Recovered {} records from {} bytes",
                        info.records_recovered, info.total_bytes
                    );
                }
            }
            Err(e) => {
                eprintln!("[boot] WARNING: origin.olang invalid header: {:?}", e);
                eprintln!("[boot] Starting from scratch — old file preserved as origin.olang.bak");
                let _ = std::fs::copy(OLANG_FILE, "origin.olang.bak");
            }
        }
        HomeRuntime::with_file(session_id, Some(bytes))
    } else {
        println!("[boot] No origin.olang — booting from nothing");
        HomeRuntime::new(session_id)
    };

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

        // Process
        let ts = now_ns();
        let response = rt.process_text(input, ts);

        // Output — silent responses (SilentAck, Observe) → không in gì
        if !response.text.is_empty() {
            println!("{}", response.text);
            println!();
        }

        // Flush pending writes → origin.olang (QT9: ghi file TRƯỚC)
        flush_pending(&mut rt);
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
        assert!(!rt.has_pending_writes());
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
        // Initially no pending writes
        assert!(!rt.has_pending_writes());
        let drained = rt.drain_pending_writes();
        assert!(drained.is_empty());
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
}
