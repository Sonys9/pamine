//! Derive macros for the Minecraft protocol types.
//!
//! Two pairs, each a reader and its writer:
//!
//! | Read | Write |
//! |---|---|
//! | [`Decode`] | [`Encode`] |
//! | [`DecodePacket`] | [`EncodePacket`] |
//!
//! Module layout:
//!
//! - [`decode`] — `Decode` for structs
//! - [`decode_packet`] — `DecodePacket` for opcode-tagged enums
//! - [`encode`] — `Encode` for structs
//! - [`encode_packet`] — `EncodePacket` for opcode-tagged enums
//! - [`common`] — helpers shared by both directions

mod common;
mod decode;
mod decode_packet;
mod encode;
mod encode_packet;
mod lua;
mod lua_packet;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Proc macro derive for decoding structs from a byte decoder.
///
/// Generated implementation:
/// ```text
/// impl ::pcore::Decode<'_> for MyStruct {
///     fn decode(decoder: &mut ::pcore::decoder::Decoder<'_>) -> Result<Self, ::pcore::errors::Error> {
///         Ok(Self { field1: decoder.varint()?, field2: decoder.u8()? })
///     }
/// }
/// ```
#[proc_macro_derive(Decode, attributes(decode))]
pub fn derive_decode(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match decode::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Proc macro derive for decoding packet enums by opcode.
///
/// Generated implementation:
/// ```text
/// impl MyEnum {
///     pub fn decode_by_opcode(opcode: i32, decoder: &mut ::pcore::decoder::Decoder<'_>) -> Result<Self, ::pcore::errors::Error> {
///         match opcode {
///             0 => Ok(Self::Variant1(Decode::decode(decoder)?)),
///             1 => Ok(Self::Variant2(Decode::decode(decoder)?)),
///             _ => Err(::pcore::errors::Error::UnknownOpcode(opcode))
///         }
///     }
/// }
/// ```
#[proc_macro_derive(DecodePacket, attributes(id))]
pub fn derive_decode_packet(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match decode_packet::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Proc macro derive for encoding structs into a byte buffer.
///
/// The mirror of [`Decode`]. Generated implementation:
/// ```text
/// impl ::pcore::Encode for MyStruct {
///     fn encode(&self, encoder: &mut ::pcore::encoder::Encoder) {
///         ::pcore::Encode::encode(&self.field1, encoder);
///         ::pcore::Encode::encode(&self.field2, encoder);
///     }
/// }
/// ```
#[proc_macro_derive(Encode, attributes(encode))]
pub fn derive_encode(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match encode::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Proc macro derive for encoding packet enums by opcode.
///
/// The mirror of [`DecodePacket`]. Generated implementation:
/// ```text
/// impl ::pcore::Encode for MyEnum {
///     fn encode(&self, encoder: &mut ::pcore::encoder::Encoder) {
///         match self {
///             Self::Variant1(inner) => {
///                 encoder.varint(0);
///                 ::pcore::Encode::encode(inner, encoder);
///             }
///             ...
///         }
///     }
/// }
/// ```
#[proc_macro_derive(EncodePacket, attributes(id))]
pub fn derive_encode_packet(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match encode_packet::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Proc macro derive for Lua table conversions.
///
/// Generates `IntoLua` for `T` and `&T`, plus `FromLua` for `T` when every
/// field is owned. Types with borrowed fields (`&'a str`, `&'a [u8]`,
/// `NbtTag<'a>`, `Slot<'a>`) get only the `IntoLua` impls: a Lua string cannot
/// be borrowed for the packet's lifetime, so rebuilding such a value from Lua
/// would require leaking.
///
/// Field mapping:
///
/// | Rust | Lua |
/// |---|---|
/// | `&'a str`, `String` | string |
/// | `&'a [u8]`, `Vec<u8>` | string (raw bytes) |
/// | `Option<T>` | `nil` or `T` |
/// | `Vec<T>` | sequence table |
/// | nested struct | table |
///
/// Enums become `{ type = "VariantName", value = ... }`.
///
/// Use `#[lua(skip)]` on a field to leave it out of the table; `FromLua` fills
/// it with `Default::default()`.
#[proc_macro_derive(Lua, attributes(lua))]
pub fn derive_lua(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match lua::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Proc macro derive for Lua packet constructors.
///
/// The inverse of [`Lua`]: instead of turning a packet into a table, it
/// generates a constructor per variant so a script can *build* a packet.
///
/// ```lua
/// local p = s2c.status.StatusResponse { json_response = "{}" }
/// -- p.opcode == 0, p.body == <encoded fields>
/// ```
///
/// Each constructor encodes its fields straight into bytes and returns a
/// [`LuaPacketValue`](pcore::lua::LuaPacketValue) — opcode plus body, no
/// framing. The caller frames it with the connection's compression threshold.
///
/// Encoding directly, rather than building the Rust struct, sidesteps the
/// lifetime problem: most packet structs borrow from the decode buffer
/// (`&'a str`, `&'a [u8]`), and a Lua string cannot be borrowed for that
/// lifetime.
///
/// The generated `lua_bindings(lua)` returns a table keyed by variant name.
#[proc_macro_derive(LuaPacket, attributes(id))]
pub fn derive_lua_packet(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match lua_packet::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
