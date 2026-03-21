//! # parse_rs — Lightweight Rust source structure extractor
//!
//! KHÔNG hardcode pattern. Parse cấu trúc thật:
//!   - struct fields (name, type, count, estimated bytes)
//!   - enum variants (name, has data, count)
//!   - fn signatures (name, params, return type)
//!   - match arms (pattern list inside match blocks)
//!   - builtin dispatch (__name calls in VM)

/// A parsed struct field.
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: String,
}

/// A parsed struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

impl StructDef {
    /// Estimate RAM size in bytes from field types.
    pub fn estimated_bytes(&self) -> usize {
        self.fields.iter().map(|f| type_size(&f.ty)).sum()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }

    pub fn field_type(&self, name: &str) -> Option<&str> {
        self.fields.iter().find(|f| f.name == name).map(|f| f.ty.as_str())
    }
}

/// A parsed enum variant.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub has_data: bool,
}

/// A parsed enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

impl EnumDef {
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.iter().any(|v| v.name == name)
    }

    pub fn variant_names(&self) -> Vec<&str> {
        self.variants.iter().map(|v| v.name.as_str()).collect()
    }
}

/// A parsed function signature.
#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub is_pub: bool,
}

/// Public wrapper for type_size.
pub fn type_size_pub(ty: &str) -> usize { type_size(ty) }

/// Estimate byte size from Rust type name.
fn type_size(ty: &str) -> usize {
    let t = ty.trim();
    match t {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        "u128" | "i128" => 16,
        "usize" | "isize" => 8, // assume 64-bit
        _ => {
            // Check common patterns
            if t.starts_with("Vec<") || t.starts_with("String") || t.starts_with("BTreeMap") {
                24 // heap-allocated, 3 words (ptr+len+cap or ptr+len+alloc)
            } else if t.starts_with("Option<") {
                // Option<T> ≈ T + 1 (discriminant, but may be niche-optimized)
                let inner = &t[7..t.len().saturating_sub(1)];
                type_size(inner) + 1
            } else if t.starts_with("[u8;") || t.starts_with("[u8 ;") {
                // Fixed array [u8; N]
                if let Some(n) = extract_array_len(t) { n } else { 8 }
            } else {
                // Unknown type — assume 8 bytes (1 word)
                8
            }
        }
    }
}

fn extract_array_len(ty: &str) -> Option<usize> {
    // [u8; 5] → 5
    let start = ty.find(';')?;
    let end = ty.find(']')?;
    ty[start + 1..end].trim().parse().ok()
}

// ═══════════════════════════════════════════════════════════════
// PARSER: Extract structs from Rust source
// ═══════════════════════════════════════════════════════════════

/// Extract all `pub struct Name { fields... }` from source.
pub fn extract_structs(source: &str) -> Vec<StructDef> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Match: pub struct Name {  or  struct Name {
        if (trimmed.starts_with("pub struct ") || trimmed.starts_with("struct "))
            && trimmed.ends_with('{')
            && !trimmed.contains("(") // skip tuple structs
        {
            let name = extract_struct_name(trimmed);
            let mut fields = Vec::new();
            i += 1;

            // Collect fields until closing }
            let mut brace_depth = 1;
            while i < lines.len() && brace_depth > 0 {
                let line = lines[i].trim();
                brace_depth += line.matches('{').count();
                brace_depth -= line.matches('}').count();

                if brace_depth > 0 && line.contains(':') && !line.starts_with("//") && !line.starts_with("///") {
                    if let Some(field) = parse_field(line) {
                        fields.push(field);
                    }
                }
                i += 1;
            }

            if !name.is_empty() {
                results.push(StructDef { name, fields });
            }
            continue;
        }
        i += 1;
    }
    results
}

fn extract_struct_name(line: &str) -> String {
    // "pub struct Molecule {" → "Molecule"
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if *p == "struct" && i + 1 < parts.len() {
            return parts[i + 1].trim_end_matches('{').trim_end_matches('<').to_string();
        }
    }
    String::new()
}

fn parse_field(line: &str) -> Option<StructField> {
    // "pub shape: u8," → StructField { name: "shape", ty: "u8" }
    let line = line.trim().trim_start_matches("pub ");
    let colon = line.find(':')?;
    let name = line[..colon].trim().to_string();
    if name.starts_with("//") || name.is_empty() {
        return None;
    }
    let rest = line[colon + 1..].trim();
    // Remove trailing comma and comments
    let ty = rest.split("//").next().unwrap_or(rest)
        .trim().trim_end_matches(',').trim().to_string();
    if ty.is_empty() { return None; }
    Some(StructField { name, ty })
}

// ═══════════════════════════════════════════════════════════════
// PARSER: Extract enums from Rust source
// ═══════════════════════════════════════════════════════════════

/// Extract all `pub enum Name { variants... }` from source.
pub fn extract_enums(source: &str) -> Vec<EnumDef> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if (trimmed.starts_with("pub enum ") || trimmed.starts_with("enum "))
            && trimmed.ends_with('{')
        {
            let name = extract_enum_name(trimmed);
            let mut variants = Vec::new();
            i += 1;

            let mut brace_depth = 1;
            while i < lines.len() && brace_depth > 0 {
                let line = lines[i].trim();
                brace_depth += line.matches('{').count();
                brace_depth -= line.matches('}').count();

                if brace_depth == 1 && !line.starts_with("//") && !line.starts_with("///")
                    && !line.starts_with('#') && !line.is_empty() && !line.starts_with('}')
                {
                    if let Some(variant) = parse_variant(line) {
                        variants.push(variant);
                    }
                }
                i += 1;
            }

            if !name.is_empty() {
                results.push(EnumDef { name, variants });
            }
            continue;
        }
        i += 1;
    }
    results
}

fn extract_enum_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if *p == "enum" && i + 1 < parts.len() {
            return parts[i + 1].trim_end_matches('{').trim_end_matches('<').to_string();
        }
    }
    String::new()
}

fn parse_variant(line: &str) -> Option<EnumVariant> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("//") { return None; }

    // "Push," or "Push(Chain)," or "Push = 0x01,"
    let name_end = line.find(|c: char| c == '(' || c == ',' || c == '=' || c == '{' || c == ' ')
        .unwrap_or(line.len());
    let name = line[..name_end].trim().to_string();
    if name.is_empty() { return None; }

    let has_data = line.contains('(');
    Some(EnumVariant { name, has_data })
}

// ═══════════════════════════════════════════════════════════════
// PARSER: Extract match arms
// ═══════════════════════════════════════════════════════════════

/// Extract match arm patterns from a function body.
/// Returns list of matched pattern strings.
pub fn extract_match_arms(source: &str, match_target: &str) -> Vec<String> {
    let mut arms = Vec::new();
    let mut in_match = false;
    let mut brace_depth = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.contains("match") && trimmed.contains(match_target) {
            in_match = true;
            brace_depth = 0;
            // Count braces on the match line itself
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;
            continue;
        }

        if in_match {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            if brace_depth <= 0 {
                in_match = false;
                continue;
            }

            // Extract arm pattern: "Op::Push(chain) =>" → "Push"
            if trimmed.contains("=>") && !trimmed.starts_with("//") {
                if let Some(arrow) = trimmed.find("=>") {
                    let pattern = trimmed[..arrow].trim();
                    // Extract the variant name
                    if let Some(last_colon) = pattern.rfind("::") {
                        let variant = pattern[last_colon + 2..]
                            .trim_start_matches(|c: char| c == ' ')
                            .split(|c: char| c == '(' || c == '{' || c == ' ')
                            .next()
                            .unwrap_or("");
                        if !variant.is_empty() && variant != "_" {
                            arms.push(variant.to_string());
                        }
                    }
                }
            }
        }
    }
    arms
}

// ═══════════════════════════════════════════════════════════════
// PARSER: Extract builtin calls from .ol files
// ═══════════════════════════════════════════════════════════════

/// Scan .ol source for `__builtin_name` calls. Returns unique names.
pub fn extract_ol_builtins(source: &str) -> Vec<String> {
    let mut builtins = std::collections::HashSet::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") { continue; }

        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if word.starts_with("__") && word.len() > 3 {
                builtins.insert(word.to_string());
            }
        }
    }

    builtins.into_iter().collect()
}

/// Extract builtin dispatch names from VM source (match arms calling builtins).
pub fn extract_vm_builtins(vm_source: &str) -> Vec<String> {
    let mut builtins = std::collections::HashSet::new();

    for line in vm_source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") { continue; }

        // Find quoted builtin names: "__hyp_add" or "__fnv1a"
        let mut start = 0;
        while let Some(pos) = trimmed[start..].find("\"__") {
            let abs_pos = start + pos + 1; // skip opening quote
            if let Some(end) = trimmed[abs_pos..].find('"') {
                let name = &trimmed[abs_pos..abs_pos + end];
                if name.starts_with("__") {
                    builtins.insert(name.to_string());
                }
            }
            start = abs_pos + 1;
        }
    }

    builtins.into_iter().collect()
}

// ═══════════════════════════════════════════════════════════════
// PARSER: Extract bit shift patterns from source
// ═══════════════════════════════════════════════════════════════

/// Extract all `x << N` patterns from a function. Returns (variable, shift_amount).
pub fn extract_bit_shifts(source: &str, fn_name: &str) -> Vec<(String, u32)> {
    let mut shifts = Vec::new();
    let mut in_fn = false;
    let mut brace_depth: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.contains(&format!("fn {}(", fn_name)) || trimmed.contains(&format!("fn {}<", fn_name)) {
            in_fn = true;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            if brace_depth <= 0 && in_fn && brace_depth != 0 {
                in_fn = false;
                continue;
            }

            // Find "var << N" patterns
            if let Some(pos) = trimmed.find("<<") {
                let before = trimmed[..pos].trim();
                let after = trimmed[pos + 2..].trim();

                // Extract variable name (last word before <<)
                let var = before.rsplit(|c: char| !c.is_alphanumeric() && c != '_')
                    .next().unwrap_or("").to_string();

                // Extract shift amount (first number after <<)
                let amount: u32 = after.split(|c: char| !c.is_ascii_digit())
                    .next().unwrap_or("0")
                    .parse().unwrap_or(0);

                if !var.is_empty() && amount > 0 {
                    shifts.push((var, amount));
                }
            }

            if brace_depth == 0 && in_fn {
                in_fn = false;
            }
        }
    }
    shifts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_struct() {
        let src = r#"
pub struct Molecule {
    pub shape: u8,
    pub relation: u8,
    pub emotion: EmotionDim,
    pub time: u8,
}
"#;
        let structs = extract_structs(src);
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Molecule");
        assert_eq!(structs[0].field_count(), 4);
        assert!(structs[0].has_field("shape"));
        assert_eq!(structs[0].field_type("shape"), Some("u8"));
    }

    #[test]
    fn test_extract_enum() {
        let src = r#"
pub enum ShapeBase {
    Sphere = 1,
    Capsule = 2,
    Box = 3,
}
"#;
        let enums = extract_enums(src);
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "ShapeBase");
        assert_eq!(enums[0].variant_count(), 3);
        assert!(enums[0].has_variant("Sphere"));
    }

    #[test]
    fn test_extract_ol_builtins() {
        let src = r#"
fn hash(x) {
    return __fnv1a(x);
}
fn print(msg) {
    __println(msg);
}
"#;
        let builtins = extract_ol_builtins(src);
        assert!(builtins.contains(&"__fnv1a".to_string()));
        assert!(builtins.contains(&"__println".to_string()));
    }

    #[test]
    fn test_type_size() {
        assert_eq!(type_size("u8"), 1);
        assert_eq!(type_size("u16"), 2);
        assert_eq!(type_size("u64"), 8);
        assert_eq!(type_size("[u8; 5]"), 5);
    }

    #[test]
    fn test_bit_shifts() {
        let src = r#"
pub fn from_molecule_lossy(mol: &Molecule) -> Self {
    let s = 3u16;
    let r = 2u16;
    let bits = (s << 13) | (r << 10) | (t << 7);
}
"#;
        let shifts = extract_bit_shifts(src, "from_molecule_lossy");
        assert!(shifts.iter().any(|(v, n)| v == "s" && *n == 13));
        assert!(shifts.iter().any(|(v, n)| v == "r" && *n == 10));
    }
}
