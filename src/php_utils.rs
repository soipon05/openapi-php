//! PHP naming utilities shared by the parser (resolver) and generator layers.
//!
//! Kept at the crate root so that `parser::resolve` can use these without
//! depending on `generator`.

// ---------------------------------------------------------------------------
// PHP reserved words — appended with `_` when used as variable/method names
// ---------------------------------------------------------------------------
pub const PHP_RESERVED: &[&str] = &[
    "array",
    "list",
    "string",
    "int",
    "float",
    "bool",
    "null",
    "true",
    "false",
    "match",
    "fn",
    "class",
    "interface",
    "enum",
    "default",
    "return",
    "echo",
    "print",
    "unset",
    "isset",
    "empty",
];

pub fn to_camel_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap_next = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '_' || ch == '-' {
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase());
            cap_next = false;
        } else if i == 0 {
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase());
            cap_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn escape_reserved(name: &str) -> String {
    if PHP_RESERVED.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

// ─── Output sanitizers ─────────────────────────────────────────────────────

/// Strip any character that is not a valid PHP identifier character (`[A-Za-z0-9_]`).
///
/// If the result is empty or starts with a digit, an underscore is prepended.
/// Prevents code injection when spec-derived names land in `class`, `function`,
/// or `$var` positions in generated PHP.
pub fn sanitize_php_ident(s: &str) -> String {
    let filtered: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if filtered.is_empty() {
        return "_".to_string();
    }
    if filtered.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{filtered}")
    } else {
        filtered
    }
}

/// Strip characters that would escape a PHP single-quoted string literal.
///
/// Removes `'`, `\`, and ASCII control characters so the result can be safely
/// embedded between `'...'` delimiters in generated PHP without additional quoting.
pub fn sanitize_php_string_literal(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '\'' && *c != '\\' && !c.is_control())
        .collect()
}

/// Sanitize free-form text for use inside a PHPDoc block comment.
///
/// Strips ASCII control characters and replaces `*/` with `* /` to prevent
/// premature block-comment termination by attacker-controlled descriptions.
pub fn sanitize_phpdoc(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .replace("*/", "* /")
}

// ─── Namespace validation ──────────────────────────────────────────────────

/// Validate that a PHP namespace string contains only allowed characters.
///
/// Valid chars: ASCII letters, digits, underscore, and backslash (namespace separator).
/// Empty namespaces are rejected — callers should supply a non-empty value.
pub fn validate_namespace(ns: &str) -> anyhow::Result<()> {
    if ns.is_empty() {
        anyhow::bail!("Namespace must not be empty");
    }
    if ns.ends_with('\\') {
        anyhow::bail!("Namespace must not end with a backslash: {:?}", ns);
    }
    let invalid: String = ns
        .chars()
        .filter(|&c| !c.is_ascii_alphanumeric() && c != '_' && c != '\\')
        .collect();
    if !invalid.is_empty() {
        anyhow::bail!(
            "Namespace {:?} contains invalid character(s): {:?}",
            ns,
            invalid
        );
    }
    Ok(())
}
