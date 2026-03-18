//! # migrate — Rust → Olang skeleton generator
//!
//! Reads Rust source files and generates Olang module skeletons with
//! type stubs and TODO markers for manual migration.
//!
//! Usage:
//!   migrate <rust_file_or_dir> [output_dir]
//!
//! Example:
//!   migrate crates/silk/src/graph.rs silk/
//!   → generates silk/graph.ol with struct/fn skeletons

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: migrate <rust_file_or_dir> [output_dir]");
        eprintln!("  Generates Olang skeleton from Rust source files.");
        std::process::exit(1);
    }

    let input = Path::new(&args[1]);
    let output_dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(".")
    };

    if input.is_dir() {
        migrate_dir(input, &output_dir);
    } else if input.is_file() {
        migrate_file(input, &output_dir);
    } else {
        eprintln!("Error: '{}' is not a valid file or directory", input.display());
        std::process::exit(1);
    }
}

fn migrate_dir(dir: &Path, output_dir: &Path) {
    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("Error reading directory '{}': {}", dir.display(), e);
            return;
        }
    };

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let sub_out = output_dir.join(path.file_name().unwrap());
            migrate_dir(&path, &sub_out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            // Skip mod.rs, lib.rs — they're re-exports
            let fname = path.file_name().unwrap().to_string_lossy();
            if fname == "mod.rs" || fname == "lib.rs" {
                continue;
            }
            migrate_file(&path, output_dir);
        }
    }
}

fn migrate_file(rust_file: &Path, output_dir: &Path) {
    let source = match fs::read_to_string(rust_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", rust_file.display(), e);
            return;
        }
    };

    let stem = rust_file.file_stem().unwrap().to_string_lossy();
    let ol_name = format!("{}.ol", stem);
    let out_path = output_dir.join(&ol_name);

    let skeleton = generate_skeleton(&stem, &source);

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Error creating output dir '{}': {}", output_dir.display(), e);
        return;
    }

    match fs::File::create(&out_path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(skeleton.as_bytes()) {
                eprintln!("Error writing '{}': {}", out_path.display(), e);
            } else {
                println!("Generated: {}", out_path.display());
            }
        }
        Err(e) => eprintln!("Error creating '{}': {}", out_path.display(), e),
    }
}

/// Generate an Olang skeleton from Rust source text.
fn generate_skeleton(module_name: &str, source: &str) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "// ─── {}.ol ── Auto-generated skeleton from Rust ───\n",
        module_name
    ));
    out.push_str("// TODO: Manually migrate logic. This file contains type stubs only.\n\n");
    out.push_str(&format!("mod {};\n\n", module_name));

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Extract struct definitions
        if line.starts_with("pub struct ") || line.starts_with("struct ") {
            let name = extract_name(line, "struct ");
            if let Some(name) = name {
                out.push_str(&format!("type {} {{\n", name));
                // Collect fields until closing brace
                if line.contains('{') {
                    i += 1;
                    while i < lines.len() {
                        let fl = lines[i].trim();
                        if fl.starts_with('}') {
                            break;
                        }
                        if let Some(field) = extract_field(fl) {
                            out.push_str(&format!("    {}\n", field));
                        }
                        i += 1;
                    }
                }
                out.push_str("}\n\n");
            }
        }
        // Extract enum definitions
        else if line.starts_with("pub enum ") || line.starts_with("enum ") {
            let name = extract_name(line, "enum ");
            if let Some(name) = name {
                out.push_str(&format!("enum {} {{\n", name));
                if line.contains('{') {
                    i += 1;
                    while i < lines.len() {
                        let fl = lines[i].trim();
                        if fl.starts_with('}') {
                            break;
                        }
                        if let Some(variant) = extract_variant(fl) {
                            out.push_str(&format!("    {}\n", variant));
                        }
                        i += 1;
                    }
                }
                out.push_str("}\n\n");
            }
        }
        // Extract impl blocks
        else if line.starts_with("impl ") && !line.contains(" for ") {
            let name = extract_impl_name(line);
            if let Some(name) = name {
                out.push_str(&format!("impl {} {{\n", name));
                if line.contains('{') {
                    i += 1;
                    let mut brace_depth = 1;
                    while i < lines.len() && brace_depth > 0 {
                        let fl = lines[i].trim();
                        // Extract fn signatures at depth 1 BEFORE counting braces
                        let depth_before = brace_depth;
                        if depth_before == 1
                            && (fl.starts_with("pub fn ") || fl.starts_with("fn "))
                        {
                            if let Some(sig) = extract_fn_sig(fl) {
                                out.push_str(&format!(
                                    "    {} {{ }}  // TODO: migrate\n",
                                    sig
                                ));
                            }
                        }
                        for ch in fl.chars() {
                            match ch {
                                '{' => brace_depth += 1,
                                '}' => brace_depth -= 1,
                                _ => {}
                            }
                        }
                        if brace_depth == 0 {
                            break;
                        }
                        i += 1;
                    }
                }
                out.push_str("}\n\n");
            }
        }
        // Extract standalone pub fn
        else if (line.starts_with("pub fn ") || line.starts_with("pub(crate) fn "))
            && !line.contains("pub fn main")
        {
            if let Some(sig) = extract_fn_sig(line) {
                out.push_str(&format!("{} {{ }}  // TODO: migrate\n\n", sig));
            }
            // Skip function body
            if line.contains('{') {
                i += 1;
                let mut brace_depth = 1;
                while i < lines.len() && brace_depth > 0 {
                    for ch in lines[i].chars() {
                        match ch {
                            '{' => brace_depth += 1,
                            '}' => brace_depth -= 1,
                            _ => {}
                        }
                    }
                    i += 1;
                }
                continue;
            }
        }

        i += 1;
    }

    out
}

/// Extract type/struct/enum name from declaration line.
fn extract_name(line: &str, keyword: &str) -> Option<String> {
    let after = line.split(keyword).nth(1)?;
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extract impl target name.
fn extract_impl_name(line: &str) -> Option<String> {
    let after = line.strip_prefix("impl ")?;
    // Handle generics: impl<T> Foo<T> → Foo
    let after = if after.starts_with('<') {
        // Skip generic params
        after.split('>').nth(1)?.trim()
    } else {
        after
    };
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Convert a Rust struct field to Olang type stub.
fn extract_field(line: &str) -> Option<String> {
    let line = line.trim().trim_end_matches(',');
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    // "pub name: Type" or "name: Type"
    let line = line.strip_prefix("pub ").unwrap_or(line);
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        let name = parts[0].trim();
        let ty = rust_type_to_olang(parts[1].trim());
        Some(format!("{}: {},", name, ty))
    } else {
        None
    }
}

/// Extract enum variant.
fn extract_variant(line: &str) -> Option<String> {
    let line = line.trim().trim_end_matches(',');
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    // "VariantName" or "VariantName { ... }" or "VariantName(Type)"
    let name: String = line
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    if line.contains('(') || line.contains('{') {
        Some(format!("{},  // TODO: payload", name))
    } else {
        Some(format!("{},", name))
    }
}

/// Extract function signature, converting to Olang style.
fn extract_fn_sig(line: &str) -> Option<String> {
    // Find "fn name(...)" pattern
    let fn_pos = line.find("fn ")?;
    let after_fn = &line[fn_pos + 3..];
    let paren_pos = after_fn.find('(')?;
    let name = after_fn[..paren_pos].trim();

    // Extract params between ( and )
    let params_start = fn_pos + 3 + paren_pos + 1;
    let params_end = line[params_start..].find(')')? + params_start;
    let params_str = &line[params_start..params_end];

    // Convert Rust params to Olang params
    let params = convert_params(params_str);

    // Check for return type
    let after_paren = &line[params_end + 1..];
    let ret = if let Some(arrow_pos) = after_paren.find("->") {
        let ret_str = after_paren[arrow_pos + 2..].trim();
        let ret_str = ret_str.trim_end_matches('{').trim();
        let ret_str = ret_str.trim_end_matches("where").trim();
        let olang_ret = rust_type_to_olang(ret_str);
        format!(" -> {}", olang_ret)
    } else {
        String::new()
    };

    let vis = if line.contains("pub ") { "pub " } else { "" };
    Some(format!("{}fn {}({}){}", vis, name, params, ret))
}

/// Convert Rust parameter list to Olang style.
fn convert_params(params: &str) -> String {
    if params.trim().is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = params.split(',').collect();
    let mut olang_params = Vec::new();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Skip &self, &mut self, self
        if part == "&self" || part == "&mut self" || part == "self" || part == "mut self" {
            olang_params.push("self".to_string());
            continue;
        }
        // "name: Type" or "&name: &Type" etc.
        let part = part.trim_start_matches("mut ");
        if let Some(colon) = part.find(':') {
            let name = part[..colon].trim().trim_start_matches('&');
            olang_params.push(name.to_string());
        } else {
            olang_params.push(part.to_string());
        }
    }

    olang_params.join(", ")
}

/// Map common Rust types to Olang equivalents.
fn rust_type_to_olang(ty: &str) -> String {
    let ty = ty.trim();
    match ty {
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
        | "f32" | "f64" => "Num".to_string(),
        "bool" => "Bool".to_string(),
        "String" | "&str" | "&'static str" => "Str".to_string(),
        "()" => "()".to_string(),
        "Self" => "Self".to_string(),
        _ => {
            // Vec<T> → Vec[T]
            if let Some(inner) = ty.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
                return format!("Vec[{}]", rust_type_to_olang(inner));
            }
            // Option<T> → Option[T]
            if let Some(inner) = ty.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
                return format!("Option[{}]", rust_type_to_olang(inner));
            }
            // Result<T, E> → Result[T]
            if ty.starts_with("Result<") {
                return "Result".to_string();
            }
            // HashMap<K,V> → Dict
            if ty.starts_with("HashMap<") || ty.starts_with("BTreeMap<") {
                return "Dict".to_string();
            }
            // Strip references and lifetimes
            let ty = ty
                .trim_start_matches('&')
                .trim_start_matches("'_ ")
                .trim_start_matches("'static ")
                .trim_start_matches("mut ");
            ty.to_string()
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_struct() {
        let src = r#"
pub struct SilkEdge {
    pub from: u64,
    pub to: u64,
    pub weight: f64,
    pub kind: EdgeKind,
}
"#;
        let skel = generate_skeleton("graph", src);
        assert!(skel.contains("type SilkEdge {"));
        assert!(skel.contains("from: Num,"));
        assert!(skel.contains("to: Num,"));
        assert!(skel.contains("weight: Num,"));
        assert!(skel.contains("kind: EdgeKind,"));
    }

    #[test]
    fn test_extract_enum() {
        let src = r#"
pub enum NodeKind {
    Alphabet,
    Knowledge,
    Memory,
    Agent,
}
"#;
        let skel = generate_skeleton("registry", src);
        assert!(skel.contains("enum NodeKind {"));
        assert!(skel.contains("Alphabet,"));
        assert!(skel.contains("Agent,"));
    }

    #[test]
    fn test_extract_impl() {
        let src = r#"
impl SilkGraph {
    pub fn new() -> Self {
        Self { nodes: vec![], edges: vec![] }
    }

    pub fn add_edge(&mut self, e: SilkEdge) {
        self.edges.push(e);
    }

    fn private_helper(&self, x: u64) -> bool {
        true
    }
}
"#;
        let skel = generate_skeleton("graph", src);
        assert!(skel.contains("impl SilkGraph {"));
        assert!(skel.contains("pub fn new() -> Self"), "should contain 'pub fn new() -> Self', got:\n{}", skel);
        assert!(skel.contains("pub fn add_edge(self, e)"), "should contain 'pub fn add_edge(self, e)', got:\n{}", skel);
        assert!(skel.contains("TODO: migrate"));
    }

    #[test]
    fn test_type_conversion() {
        assert_eq!(rust_type_to_olang("u64"), "Num");
        assert_eq!(rust_type_to_olang("f64"), "Num");
        assert_eq!(rust_type_to_olang("String"), "Str");
        assert_eq!(rust_type_to_olang("Vec<u8>"), "Vec[Num]");
        assert_eq!(rust_type_to_olang("Option<String>"), "Option[Str]");
        assert_eq!(rust_type_to_olang("bool"), "Bool");
    }

    #[test]
    fn test_pub_fn_extraction() {
        let src = r#"
pub fn walk_weighted(graph: &SilkGraph, words: &[u64]) -> EmotionDim {
    let total = EmotionDim::default();
    for w in words {
        // logic
    }
    total
}
"#;
        let skel = generate_skeleton("walk", src);
        assert!(skel.contains("pub fn walk_weighted(graph, words)"));
        assert!(skel.contains("-> EmotionDim"));
    }

    #[test]
    fn test_full_skeleton_header() {
        let src = "pub struct Foo { pub x: u32 }";
        let skel = generate_skeleton("foo", src);
        assert!(skel.starts_with("// ─── foo.ol"));
        assert!(skel.contains("mod foo;"));
    }
}
