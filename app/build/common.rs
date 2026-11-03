//! Helpers shared by the code generators.
//!
//! Every accessor takes a `context` string so a malformed input fails the build
//! with a message that names the offending block or registry instead of dumping
//! a multi-megabyte JSON value into the compiler output.

use serde_json::Value;

pub fn field<'a>(value: &'a Value, key: &str, context: &str) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| panic!("{context}: missing `{key}`"))
}

pub fn as_u64(value: &Value, key: &str, context: &str) -> u64 {
    field(value, key, context)
        .as_u64()
        .unwrap_or_else(|| panic!("{context}: `{key}` is not an unsigned integer"))
}

pub fn as_f64(value: &Value, key: &str, context: &str) -> f64 {
    field(value, key, context)
        .as_f64()
        .unwrap_or_else(|| panic!("{context}: `{key}` is not a number"))
}

pub fn as_bool(value: &Value, key: &str, context: &str) -> bool {
    field(value, key, context)
        .as_bool()
        .unwrap_or_else(|| panic!("{context}: `{key}` is not a bool"))
}

pub fn as_str<'a>(value: &'a Value, key: &str, context: &str) -> &'a str {
    field(value, key, context)
        .as_str()
        .unwrap_or_else(|| panic!("{context}: `{key}` is not a string"))
}

/// Returns the array under `key`, or an empty slice when the key is absent.
pub fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// `{:?}` on `&str` produces a correctly escaped Rust string literal.
pub fn lit(value: &str) -> String {
    format!("{value:?}")
}

/// `{:?}` on floats is round-trippable and always contains a `.` or an
/// exponent, so the result is a valid float literal.
pub fn f32_lit(value: f64) -> String {
    format!("{:?}", value as f32)
}

pub fn f64_lit(value: f64) -> String {
    format!("{value:?}")
}

/// Drops the `minecraft:` namespace, leaving `stone` or `worldgen/feature`.
pub fn strip_namespace(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// Turns an arbitrary registry key into a valid Rust identifier fragment:
/// `mineable/pickaxe` -> `mineable_pickaxe`.
pub fn ident(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    for ch in tag.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Same as [`ident`], but upper-cased for use in a constant name.
pub fn upper_ident(tag: &str) -> String {
    ident(tag).to_uppercase()
}
