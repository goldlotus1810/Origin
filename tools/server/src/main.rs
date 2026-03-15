//! # server — HomeOS REPL
//!
//! Terminal REPL cho HomeOS.
//! Dùng stdin/stdout — không cần framework.
//!
//! ○(∅) == ○ — boot từ hư không.

use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use runtime::origin::HomeRuntime;

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
    let mut rt = HomeRuntime::new(session_id);

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
    }

    // Stats khi thoát
    println!();
    println!("○ Session ended · {} turns · f(x)={:.3}",
        rt.turn_count(), rt.fx());
}
