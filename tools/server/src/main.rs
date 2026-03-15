//! # server — HomeOS REPL
//!
//! Terminal REPL cho HomeOS.
//! Dùng stdin/stdout — không cần framework.
//!
//! ○(∅) == ○ — boot từ hư không.

use std::io::{self, BufRead, Write};
use std::fs::OpenOptions;
use std::time::{SystemTime, UNIX_EPOCH};

use runtime::origin::HomeRuntime;

const OLANG_FILE: &str = "origin.olang";

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64
}

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
    // Load origin.olang nếu có
    let file_bytes = std::fs::read("origin.olang").ok();
    let mut rt = if let Some(ref bytes) = file_bytes {
        println!("[boot] origin.olang: {} bytes", bytes.len());
        HomeRuntime::with_file(session_id, Some(bytes))
    } else {
        println!("[boot] No origin.olang — booting from nothing");
        HomeRuntime::new(session_id)
    };

    let stdin  = io::stdin();
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
            Err(e) => { eprintln!("Read error: {}", e); break; }
        }

        let input = line.trim();
        if input.is_empty() { continue; }
        if input == "exit" || input == "quit" { break; }

        // Process
        let ts = now_ns();
        let response = rt.process_text(input, ts);

        // Output
        println!("{}", response.text);
        println!();

        // Flush pending writes → origin.olang (QT9: ghi file TRƯỚC)
        flush_pending(&mut rt);
    }

    // Final persist: serialize remaining learned data
    let final_bytes = rt.serialize_learned(now_ns());
    if final_bytes.len() > olang::writer::HEADER_SIZE {
        if let Err(e) = append_to_file(OLANG_FILE, &final_bytes) {
            eprintln!("[persist] Error writing final state: {}", e);
        } else {
            println!("[persist] Saved {} bytes on exit", final_bytes.len());
        }
    }

    // Stats khi thoát
    println!();
    println!("○ Session ended · {} turns · f(x)={:.3}",
        rt.turn_count(), rt.fx());
}

/// Flush pending writes từ runtime → origin.olang.
fn flush_pending(rt: &mut HomeRuntime) {
    if !rt.has_pending_writes() { return; }

    let bytes = rt.drain_pending_writes();
    if bytes.is_empty() { return; }

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
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}
