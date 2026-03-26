//! # import_utf32 — Load udc_utf32.json → L1 nodes in origin.olang
//!
//! Chạy 1 lần khi first boot (Virgin state).
//! Sau đó origin.olang tự đủ — không cần json nữa.
//!
//! Flow: json → parse → encode P_weight → OlangWriter → append origin.olang

use std::fs;
use std::io::Write;

const UTF32_JSON: &str = "json/udc_utf32.json";

/// Check if udc_utf32.json exists and hasn't been imported yet.
pub fn needs_import(olang_path: &str) -> bool {
    // If json doesn't exist → nothing to import
    if !std::path::Path::new(UTF32_JSON).exists() {
        return false;
    }
    // If origin.olang is small → hasn't been imported
    let olang_size = fs::metadata(olang_path).map(|m| m.len()).unwrap_or(0);
    // After import, origin.olang should be >1MB (41K nodes × ~20B each)
    olang_size < 500_000
}

/// Import udc_utf32.json → origin.olang L1 nodes + aliases.
/// Returns: (chars_imported, aliases_imported, bytes_written)
pub fn import(olang_path: &str, ts: i64) -> Result<(usize, usize, usize), String> {
    let json_str = fs::read_to_string(UTF32_JSON)
        .map_err(|e| format!("Cannot read {}: {}", UTF32_JSON, e))?;

    // Parse JSON manually (no serde — server already depends on nothing extra)
    // Structure: { "planes": { "0": { "blocks": { "Basic Latin": { "chars": { "0041": { "P": 12345, "aliases": {...} } } } } } } }
    let chars = extract_chars(&json_str)?;

    let mut writer = olang::writer::OlangWriter::new_append();
    let mut chars_imported = 0usize;
    let mut aliases_imported = 0usize;

    for (cp, p_weight, names) in &chars {
        if *p_weight == 0 { continue; }

        // Encode codepoint → MolecularChain
        let chain = olang::encoder::encode_codepoint(*cp);
        if chain.is_empty() { continue; }

        let hash = chain.chain_hash();

        // Write L1 node (append-only)
        let _ = writer.append_node(&chain, 1, false, ts);
        chars_imported += 1;

        // Write alias records
        for name in names {
            if !name.is_empty() {
                writer.append_alias(name, hash, ts);
                aliases_imported += 1;
            }
        }
    }

    // Append to origin.olang
    let bytes = writer.as_bytes();
    let bytes_written = bytes.len();
    if !bytes.is_empty() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(olang_path)
            .map_err(|e| format!("Cannot write {}: {}", olang_path, e))?;
        file.write_all(bytes)
            .map_err(|e| format!("Write failed: {}", e))?;
    }

    Ok((chars_imported, aliases_imported, bytes_written))
}

/// Simple JSON char extractor — no serde dependency.
/// Returns: Vec<(codepoint, p_weight, vec_of_names)>
fn extract_chars(json: &str) -> Result<Vec<(u32, u16, Vec<String>)>, String> {
    // Use minimal JSON parsing — find "chars" objects and extract entries
    let mut results = Vec::new();

    // Find all patterns: "XXXX": {"P": NNNNN, ...}
    // where XXXX is hex codepoint
    let chars_pat = "\"chars\"";
    let mut search_from = 0;

    while let Some(chars_pos) = json[search_from..].find(chars_pat) {
        let abs_pos = search_from + chars_pos + chars_pat.len();
        search_from = abs_pos;

        // Skip to opening { of chars object
        let rest = &json[abs_pos..];
        let brace_pos = match rest.find('{') {
            Some(p) => p,
            None => continue,
        };

        // Parse char entries within this chars block
        let chars_start = abs_pos + brace_pos;
        let mut depth = 0i32;
        let mut i = chars_start;
        let bytes = json.as_bytes();

        // Find matching closing brace
        let mut chars_end = chars_start;
        for j in chars_start..bytes.len() {
            if bytes[j] == b'{' { depth += 1; }
            if bytes[j] == b'}' {
                depth -= 1;
                if depth == 0 {
                    chars_end = j + 1;
                    break;
                }
            }
        }

        let chars_content = &json[chars_start..chars_end];

        // Extract individual char entries: "XXXX": {"P": ...}
        let mut cp_search = 0;
        while cp_search < chars_content.len() {
            // Find "XXXX" pattern (4+ hex digits)
            if let Some(quote_pos) = chars_content[cp_search..].find('"') {
                let start = cp_search + quote_pos + 1;
                if let Some(end_quote) = chars_content[start..].find('"') {
                    let key = &chars_content[start..start + end_quote];

                    // Must be hex codepoint (4-6 hex chars)
                    if key.len() >= 4 && key.len() <= 6
                        && key.chars().all(|c| c.is_ascii_hexdigit())
                    {
                        let cp = u32::from_str_radix(key, 16).unwrap_or(0);

                        // Find "P": NNNNN
                        let entry_start = start + end_quote;
                        let entry_rest = &chars_content[entry_start..];

                        let p_weight = extract_p_value(entry_rest);
                        let names = extract_names(entry_rest);

                        if cp > 0 && p_weight > 0 {
                            results.push((cp, p_weight, names));
                        }
                    }

                    cp_search = start + end_quote + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        search_from = chars_end;
    }

    Ok(results)
}

/// Extract "P": NNNNN from a JSON fragment.
fn extract_p_value(s: &str) -> u16 {
    if let Some(p_pos) = s.find("\"P\"") {
        let after_p = &s[p_pos + 3..];
        if let Some(colon) = after_p.find(':') {
            let after_colon = after_p[colon + 1..].trim_start();
            let num_end = after_colon.find(|c: char| !c.is_ascii_digit()).unwrap_or(after_colon.len());
            after_colon[..num_end].parse::<u16>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    }
}

/// Extract alias names from a JSON fragment.
fn extract_names(s: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Find "name": "value"
    let mut search = 0;
    while let Some(name_pos) = s[search..].find("\"name\"") {
        let after = &s[search + name_pos + 6..];
        if let Some(colon) = after.find(':') {
            let after_colon = after[colon + 1..].trim_start();
            if after_colon.starts_with('"') {
                let start = 1;
                if let Some(end) = after_colon[start..].find('"') {
                    let name = &after_colon[start..start + end];
                    if !name.is_empty()
                        && name != "control"
                        && !names.contains(&name.to_lowercase())
                    {
                        names.push(name.to_lowercase());
                    }
                }
            }
        }
        search += name_pos + 6;
    }

    // Find "alias": "value"
    search = 0;
    while let Some(alias_pos) = s[search..].find("\"alias\"") {
        let after = &s[search + alias_pos + 7..];
        if let Some(colon) = after.find(':') {
            let after_colon = after[colon + 1..].trim_start();
            if after_colon.starts_with('"') {
                let start = 1;
                if let Some(end) = after_colon[start..].find('"') {
                    let alias = &after_colon[start..start + end];
                    if !alias.is_empty() && !names.contains(&alias.to_lowercase()) {
                        names.push(alias.to_lowercase());
                    }
                }
            }
        }
        search += alias_pos + 7;

        // Limit aliases per char to avoid explosion
        if names.len() >= 5 { break; }
    }

    names
}
