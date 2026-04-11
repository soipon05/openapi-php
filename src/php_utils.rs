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
