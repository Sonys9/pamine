//! Compile-fail tests for pcore-gen macros
//! These tests verify that the macros produce appropriate compile errors for invalid input.

#[test]
fn test_decode_derive_on_enum_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/decode_on_enum.rs");
}

#[test]
fn test_decode_packet_on_struct_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/decode_packet_on_struct.rs");
}

#[test]
fn test_decode_packet_missing_id_attribute_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/missing_id_attribute.rs");
}

#[test]
fn test_decode_packet_invalid_variant_fields_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/invalid_variant_fields.rs");
}

#[test]
fn test_decode_derive_on_union_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/decode_on_union.rs");
}

#[test]
fn test_decode_packet_non_integer_id_should_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/non_integer_id.rs");
}