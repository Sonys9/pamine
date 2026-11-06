//! `#[derive(LuaPacket)]` — Lua constructors for packets.
//!
//! `#[derive(Lua)]` turns a packet into a table, which is what a handler
//! *reads*. This derive is the other direction: it lets a script *build* a
//! packet and hand it back for injection.
//!
//! Three shapes, because the packet types are three shapes:
//!
//! - **Struct** → `lua_encode_table(table, encoder)`: reads the named fields
//!   out of a Lua table and writes them into an encoder.
//! - **Opcode enum** (variants carry `#[id(N)]`, no `#[lua(tag)]` on the
//!   container) → `lua_bindings(lua)`: a table of constructors keyed by variant
//!   name. Each returns a [`LuaPacketValue`] carrying the variant's opcode.
//! - **Value enum** (`#[lua(tag)]` on the container) → `lua_encode_table`: the
//!   variant is chosen by `table.type`, its `#[id(N)]` is written as a VarInt,
//!   then the variant's fields. This is for enums discriminated by a value in
//!   the middle of a packet, like `BossBarAction` (action id) or
//!   `ParticleData` (particle type id).
//!
//! ```lua
//! local p = s2c.status.StatusResponse { json_response = "{}" }
//! -- p.opcode == 0, p.body == <encoded fields>
//!
//! local bar = s2c.play.BossBar {
//!     uuid = <16 bytes>,
//!     action = { type = "Add", title = "hi", health = 1.0,
//!                color = 0, division = 0, flags = 0 },
//! }
//! ```
//!
//! ## Why encode instead of building the struct
//!
//! Most packet structs borrow from the decode buffer (`&'a str`, `&'a [u8]`).
//! A Lua string lives in Lua's GC heap and cannot be borrowed for the packet
//! lifetime, so `FromLua` for those structs is impossible without leaking.
//! Encoding directly sidesteps the lifetime: the bytes are produced while the
//! Lua string is still alive.
//!
//! ## Field encoding
//!
//! | Rust | Lua | Encoder call |
//! |---|---|---|
//! | `&'a str`, `String` | string | `encoder.string(&v)` |
//! | `&'a [u8]`, `Vec<u8>` | string | `encoder.bytes(&v)` |
//! | `#[decode(rest)]` / `#[decode(len)]` | string | `encoder.raw(&v)` |
//! | `Option<T>` | `nil` or `T` | `encoder.bool(v.is_some())` + value |
//! | `Vec<T>` | sequence | `encoder.varint(len)` + each |
//! | `VarInt` | integer | `encoder.varint(v)` |
//! | `Uuid` | 16-byte string | `encoder.uuid(&v)` |
//! | `NbtTag` | table | `encoder.nbt(&v)` |
//! | `Slot` | table | `encoder.slot(&v)` |
//! | primitives | number/bool | `encoder.<type>(v)` |
//! | nested struct / value enum | table | `T::lua_encode_table(&v, encoder)` |

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

use crate::common::{check_duplicate_id, check_single_unnamed, variant_id};

/// How a field is read from Lua and written to the encoder.
enum FieldKind {
    Str,
    Bytes,
    /// `#[decode(rest)]` / `#[decode(len = "...")]`: raw bytes, no prefix.
    Raw,
    VarInt,
    VarLong,
    Uuid,
    Nbt,
    Slot,
    Primitive(&'static str),
    /// `Option<T>`: a bool gate followed by the value.
    Optional(Box<Type>),
    /// `Vec<T>`: a VarInt count followed by each element.
    Sequence(Box<Type>),
    /// `[T; N]`: a fixed number of elements, no length prefix.
    Array(Box<Type>),
    /// `(A, B, ...)`: a fixed sequence of heterogeneous values.
    Tuple(Vec<Type>),
    /// Anything else: assume it has `lua_encode_table`.
    Nested,
}

/// True when the field is marked `#[decode(rest)]` or `#[decode(len = "...")]`.
fn is_raw(field: &syn::Field) -> bool {
    field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("decode") {
            return false;
        }
        let mut raw = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rest") || meta.path.is_ident("len") {
                raw = true;
            }
            Ok(())
        });
        raw
    })
}

/// Reads `#[lua(validate = "path::to::fn")]` from a variant.
///
/// The generated arm calls `path(table)?` before encoding anything, which lets
/// a variant reject a table whose fields are individually well-typed but
/// mutually inconsistent — e.g. a particle id that does not match the payload
/// variant. Without it the bytes would be written and the peer would
/// desynchronise.
fn variant_validate(variant: &syn::Variant) -> syn::Result<Option<syn::Path>> {
    let Some(attr) = variant.attrs.iter().find(|a| a.path().is_ident("lua")) else {
        return Ok(None);
    };
    let mut path = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("validate") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            path = Some(lit.parse::<syn::Path>()?);
        }
        Ok(())
    })?;
    Ok(path)
}

/// True when the field is marked `#[lua(skip)]`.
fn is_skipped(field: &syn::Field) -> bool {
    field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("lua") {
            return false;
        }
        let mut skip = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
            }
            Ok(())
        });
        skip
    })
}

fn field_kind(field: &syn::Field) -> FieldKind {
    if is_raw(field) {
        return FieldKind::Raw;
    }
    let ty = &field.ty;
    match ty {
        // `[T; N]` is a fixed-size array: no length prefix, one element each.
        Type::Array(array) => FieldKind::Array(array.elem.clone()),
        // A tuple is a fixed sequence of heterogeneous values.
        Type::Tuple(tuple) => FieldKind::Tuple(tuple.elems.iter().cloned().collect()),
        Type::Reference(reference) => match &*reference.elem {
            Type::Slice(_) => FieldKind::Bytes,
            Type::Path(path) if path.path.is_ident("str") => FieldKind::Str,
            _ => FieldKind::Nested,
        },
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return FieldKind::Nested;
            };
            let name = segment.ident.to_string();
            match name.as_str() {
                "String" => return FieldKind::Str,
                "VarInt" => return FieldKind::VarInt,
                "VarLong" => return FieldKind::VarLong,
                "Uuid" => return FieldKind::Uuid,
                "NbtTag" => return FieldKind::Nbt,
                "Slot" => return FieldKind::Slot,
                "i8" => return FieldKind::Primitive("i8"),
                "u8" => return FieldKind::Primitive("u8"),
                "bool" => return FieldKind::Primitive("bool"),
                "i16" => return FieldKind::Primitive("i16"),
                "u16" => return FieldKind::Primitive("u16"),
                "i32" => return FieldKind::Primitive("i32"),
                "u32" => return FieldKind::Primitive("u32"),
                "i64" => return FieldKind::Primitive("i64"),
                "u64" => return FieldKind::Primitive("u64"),
                "f32" => return FieldKind::Primitive("f32"),
                "f64" => return FieldKind::Primitive("f64"),
                _ => {}
            }

            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                match name.as_str() {
                    "Option" => return FieldKind::Optional(Box::new(inner.clone())),
                    "Vec" => {
                        // `Vec<u8>` is a length-prefixed byte string, not a
                        // sequence of numbers.
                        if let Type::Path(p) = inner
                            && p.path.is_ident("u8")
                        {
                            return FieldKind::Bytes;
                        }
                        return FieldKind::Sequence(Box::new(inner.clone()));
                    }
                    _ => {}
                }
            }
            FieldKind::Nested
        }
        _ => FieldKind::Nested,
    }
}

/// The Lua type a field is read as.
fn lua_type(field: &syn::Field) -> TokenStream {
    match field_kind(field) {
        FieldKind::Str => quote! { ::std::string::String },
        FieldKind::Bytes | FieldKind::Raw => quote! { ::mlua::String },
        FieldKind::VarInt => quote! { i32 },
        FieldKind::VarLong => quote! { i64 },
        FieldKind::Uuid => quote! { ::mlua::String },
        FieldKind::Nbt => quote! { ::pcore::lua::OwnedNbtTag },
        FieldKind::Slot => quote! { ::pcore::lua::OwnedSlot },
        FieldKind::Primitive(name) => {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident }
        }
        FieldKind::Optional(inner) => {
            let inner = lua_type_of(&inner);
            quote! { ::core::option::Option<#inner> }
        }
        FieldKind::Sequence(inner) => {
            let inner = lua_type_of(&inner);
            quote! { ::std::vec::Vec<#inner> }
        }
        FieldKind::Array(inner) => {
            let inner = lua_type_of(&inner);
            let len = match &field.ty {
                Type::Array(array) => array.len.clone(),
                _ => unreachable!("Array kind implies Type::Array"),
            };
            quote! { [#inner; #len] }
        }
        FieldKind::Tuple(elems) => {
            let types = elems.iter().map(lua_type_of);
            quote! { (#(#types),*) }
        }
        // A nested struct or value enum is read as a Lua table and then
        // converted by `T::lua_encode_table`, which takes `&mlua::Table`.
        FieldKind::Nested => quote! { ::mlua::Table },
    }
}

/// The Lua type of a bare `Type`, for `Option<T>` / `Vec<T>` elements.
///
/// Recursive: `Vec<Vec<&str>>` becomes `Vec<Vec<String>>`, and an unknown path
/// keeps its Rust type so `lua_encode_table` can be called on it.
fn lua_type_of(ty: &Type) -> TokenStream {
    match ty {
        // `[T; N]` is read as a Lua sequence and converted to a fixed array.
        Type::Array(array) => {
            let inner = lua_type_of(&array.elem);
            let len = &array.len;
            quote! { [#inner; #len] }
        }
        Type::Reference(reference) => match &*reference.elem {
            Type::Slice(_) => quote! { ::mlua::String },
            Type::Path(path) if path.path.is_ident("str") => quote! { ::std::string::String },
            _ => quote! { ::mlua::Table },
        },
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return quote! { ::mlua::Table };
            };
            let name = segment.ident.to_string();

            // `Option<T>` and `Vec<T>` recurse into their element type.
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                match name.as_str() {
                    "Option" => {
                        let inner = lua_type_of(inner);
                        return quote! { ::core::option::Option<#inner> };
                    }
                    "Vec" => {
                        if let Type::Path(p) = inner
                            && p.path.is_ident("u8")
                        {
                            return quote! { ::mlua::String };
                        }
                        let inner = lua_type_of(inner);
                        return quote! { ::std::vec::Vec<#inner> };
                    }
                    _ => {}
                }
            }

            match name.as_str() {
                "String" => quote! { ::std::string::String },
                "VarInt" => quote! { i32 },
                "VarLong" => quote! { i64 },
                "Uuid" => quote! { ::mlua::String },
                "NbtTag" => quote! { ::pcore::lua::OwnedNbtTag },
                "Slot" => quote! { ::pcore::lua::OwnedSlot },
                "i8" => quote! { i8 },
                "u8" => quote! { u8 },
                "bool" => quote! { bool },
                "i16" => quote! { i16 },
                "u16" => quote! { u16 },
                "i32" => quote! { i32 },
                "u32" => quote! { u32 },
                "i64" => quote! { i64 },
                "u64" => quote! { u64 },
                "f32" => quote! { f32 },
                "f64" => quote! { f64 },
                // A nested struct or value enum is read as a Lua table; the
                // generated code then calls `T::lua_encode_table(&table, ...)`.
                _ => quote! { ::mlua::Table },
            }
        }
        _ => quote! { ::mlua::Table },
    }
}

/// Statements that write one field into `encoder`.
fn encode_field(field: &syn::Field, value: &TokenStream) -> TokenStream {
    match field_kind(field) {
        FieldKind::Str => quote! { encoder.string(&#value); },
        FieldKind::Bytes => quote! { encoder.bytes(&#value.as_bytes()); },
        FieldKind::Raw => quote! { encoder.raw(&#value.as_bytes()); },
        FieldKind::VarInt => quote! { encoder.varint(#value); },
        FieldKind::VarLong => quote! { encoder.varlong(#value); },
        FieldKind::Uuid => quote! {
            {
                let bytes = #value.as_bytes();
                if bytes.len() != 16 {
                    return ::core::result::Result::Err(::mlua::Error::runtime(
                        "Uuid must be a 16-byte string",
                    ));
                }
                let mut raw = [0u8; 16];
                raw.copy_from_slice(&bytes);
                encoder.uuid(&::pcore::decoder::Uuid::new(raw));
            }
        },
        FieldKind::Nbt => quote! { encoder.nbt(&#value.as_borrowed()); },
        FieldKind::Slot => quote! { encoder.slot(&#value.as_borrowed()); },
        FieldKind::Primitive(name) => {
            let method = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { encoder.#method(#value); }
        }
        FieldKind::Optional(inner) => {
            let inner_encode = encode_type(&inner, &quote! { inner_value });
            quote! {
                match #value {
                    ::core::option::Option::Some(inner_value) => {
                        encoder.bool(true);
                        #inner_encode
                    }
                    ::core::option::Option::None => encoder.bool(false),
                }
            }
        }
        FieldKind::Sequence(inner) => {
            let inner_encode = encode_type(&inner, &quote! { item });
            quote! {
                encoder.varint(#value.len() as i32);
                for item in #value {
                    #inner_encode
                }
            }
        }
        // `[T; N]`: one element each, no length prefix.
        FieldKind::Array(inner) => {
            let inner_encode = encode_type(&inner, &quote! { item });
            quote! {
                for item in #value {
                    #inner_encode
                }
            }
        }
        // `(A, B, ...)`: one element each, in order.
        FieldKind::Tuple(elems) => {
            let encodes = elems.iter().enumerate().map(|(i, ty)| {
                let index = syn::Index::from(i);
                let element = quote! { #value.#index };
                encode_type(ty, &element)
            });
            quote! { #(#encodes)* }
        }
        // `lua_encode_table` is an associated function, so it needs the type
        // spelled out: `T::lua_encode_table(&value, encoder)`.
        FieldKind::Nested => {
            let ty = &field.ty;
            quote! { <#ty>::lua_encode_table(&#value, encoder)?; }
        }
    }
}

/// Statements that write a bare `Type` value into `encoder`.
fn encode_type(ty: &Type, value: &TokenStream) -> TokenStream {
    match ty {
        // `[T; N]` is a fixed-size array: write each element, no length prefix.
        Type::Array(array) => {
            let inner = encode_type(&array.elem, &quote! { item });
            quote! {
                for item in #value {
                    #inner
                }
            }
        }
        Type::Reference(reference) => match &*reference.elem {
            Type::Slice(_) => quote! { encoder.bytes(&#value.as_bytes()); },
            Type::Path(path) if path.path.is_ident("str") => {
                quote! { encoder.string(&#value); }
            }
            _ => quote! { <#ty>::lua_encode_table(&#value, encoder)?; },
        },
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return quote! { <#ty>::lua_encode_table(&#value, encoder)?; };
            };
            let name = segment.ident.to_string();

            // `Option<T>` and `Vec<T>` recurse into their element type.
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                match name.as_str() {
                    "Option" => {
                        let inner_encode = encode_type(inner, &quote! { inner_value });
                        return quote! {
                            match #value {
                                ::core::option::Option::Some(inner_value) => {
                                    encoder.bool(true);
                                    #inner_encode
                                }
                                ::core::option::Option::None => encoder.bool(false),
                            }
                        };
                    }
                    "Vec" => {
                        if let Type::Path(p) = inner
                            && p.path.is_ident("u8")
                        {
                            return quote! { encoder.bytes(&#value.as_bytes()); };
                        }
                        let inner_encode = encode_type(inner, &quote! { item });
                        return quote! {
                            encoder.varint(#value.len() as i32);
                            for item in #value {
                                #inner_encode
                            }
                        };
                    }
                    _ => {}
                }
            }

            match name.as_str() {
                "String" => quote! { encoder.string(&#value); },
                "VarInt" => quote! { encoder.varint(#value); },
                "VarLong" => quote! { encoder.varlong(#value); },
                "Uuid" => quote! {
                    {
                        let bytes = #value.as_bytes();
                        if bytes.len() != 16 {
                            return ::core::result::Result::Err(::mlua::Error::runtime(
                                "Uuid must be a 16-byte string",
                            ));
                        }
                        let mut raw = [0u8; 16];
                        raw.copy_from_slice(&bytes);
                        encoder.uuid(&::pcore::decoder::Uuid::new(raw));
                    }
                },
                "NbtTag" => quote! { encoder.nbt(&#value.as_borrowed()); },
                "Slot" => quote! { encoder.slot(&#value.as_borrowed()); },
                "i8" => quote! { encoder.i8(#value); },
                "u8" => quote! { encoder.u8(#value); },
                "bool" => quote! { encoder.bool(#value); },
                "i16" => quote! { encoder.i16(#value); },
                "u16" => quote! { encoder.u16(#value); },
                "i32" => quote! { encoder.i32(#value); },
                "u32" => quote! { encoder.u32(#value); },
                "i64" => quote! { encoder.i64(#value); },
                "u64" => quote! { encoder.u64(#value); },
                "f32" => quote! { encoder.f32(#value); },
                "f64" => quote! { encoder.f64(#value); },
                _ => quote! { <#ty>::lua_encode_table(&#value, encoder)?; },
            }
        }
        _ => quote! { <#ty>::lua_encode_table(&#value, encoder)?; },
    }
}

/// Reads one named field out of `table` and encodes it.
///
/// A `#[lua(skip)]` field is not read and not written: it exists only to
/// satisfy a lifetime (`PhantomData`) or to hold derived state.
fn read_and_encode(field: &syn::Field) -> TokenStream {
    if is_skipped(field) {
        return quote! {};
    }
    let field_name = field.ident.as_ref().expect("named field");
    let ty = lua_type(field);
    let key = field_name.to_string();
    let encode = encode_field(field, &quote! { #field_name });
    quote! {
        let #field_name: #ty = table.get(#key)?;
        #encode
    }
}

/// Expands `#[derive(LuaPacket)]` for a struct: `lua_encode_table`.
fn expand_struct(
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &Fields,
) -> syn::Result<TokenStream> {
    let Fields::Named(named) = fields else {
        return Err(syn::Error::new_spanned(
            fields,
            "LuaPacket requires named fields",
        ));
    };

    let reads: Vec<_> = named.named.iter().map(read_and_encode).collect();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Encodes a Lua table into this struct's wire form.
            pub fn lua_encode_table(
                table: &::mlua::Table,
                encoder: &mut ::pcore::encoder::Encoder,
            ) -> ::core::result::Result<(), ::mlua::Error> {
                #(#reads)*
                ::core::result::Result::Ok(())
            }
        }
    })
}

/// Expands a value enum (`#[lua(tag)]`): `lua_encode_table` that writes the
/// variant's `#[id(N)]` as a VarInt, then the variant's fields.
fn expand_value_enum(
    name: &syn::Ident,
    generics: &syn::Generics,
    data: &syn::DataEnum,
) -> syn::Result<TokenStream> {
    let mut arms = Vec::new();
    let mut seen = std::collections::HashMap::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let tag = variant_id(variant)?;
        check_duplicate_id(&mut seen, &tag, variant)?;
        let key = variant_name.to_string();

        // `#[lua(validate = "path")]` runs before anything is written, so a
        // table whose fields are individually valid but mutually inconsistent
        // is rejected instead of producing bytes the peer cannot parse.
        let validate = match variant_validate(variant)? {
            Some(path) => quote! { #path(table)?; },
            None => quote! {},
        };

        let body = match &variant.fields {
            Fields::Unit => quote! {},
            Fields::Named(named) => {
                let reads: Vec<_> = named.named.iter().map(read_and_encode).collect();
                quote! { #(#reads)* }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed[0];
                let ty = lua_type(field);
                let encode = encode_field(field, &quote! { inner });
                quote! {
                    let inner: #ty = table.get("value")?;
                    #encode
                }
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "LuaPacket supports at most one unnamed field per variant",
                ));
            }
        };

        arms.push(quote! {
            #key => {
                #validate
                encoder.varint(#tag);
                #body
            }
        });
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Encodes a Lua table into this enum's wire form.
            ///
            /// The variant is chosen by `table.type`; its `#[id(N)]` is written
            /// first, then the variant's fields.
            pub fn lua_encode_table(
                table: &::mlua::Table,
                encoder: &mut ::pcore::encoder::Encoder,
            ) -> ::core::result::Result<(), ::mlua::Error> {
                let variant: ::std::string::String = table.get("type")?;
                match variant.as_str() {
                    #(#arms)*
                    other => {
                        return ::core::result::Result::Err(::mlua::Error::runtime(format!(
                            "unknown variant {other:?} for {}",
                            stringify!(#name)
                        )));
                    }
                }
                ::core::result::Result::Ok(())
            }
        }
    })
}

/// Expands an opcode enum: `lua_bindings`.
fn expand_opcode_enum(
    name: &syn::Ident,
    generics: &syn::Generics,
    data: &syn::DataEnum,
) -> syn::Result<TokenStream> {
    let mut sets = Vec::new();
    let mut seen = std::collections::HashMap::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let opcode = variant_id(variant)?;
        check_duplicate_id(&mut seen, &opcode, variant)?;
        check_single_unnamed(variant)?;

        let Fields::Unnamed(fields) = &variant.fields else {
            return Err(syn::Error::new_spanned(
                variant,
                "LuaPacket opcode enum variants must be tuple variants wrapping a struct",
            ));
        };
        let inner = &fields.unnamed[0].ty;

        let key = variant_name.to_string();
        sets.push(quote! {
            table.set(
                #key,
                lua.create_function(|_, fields: ::mlua::Table| {
                    let mut encoder = ::pcore::encoder::Encoder::new();
                    <#inner>::lua_encode_table(&fields, &mut encoder)?;
                    ::core::result::Result::Ok(::pcore::lua::LuaPacketValue::new(
                        #opcode,
                        encoder.into_bytes(),
                    ))
                })?,
            )?;
        });
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Builds the Lua namespace of constructors for this enum.
            pub fn lua_bindings(lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Table> {
                let table = lua.create_table()?;
                #(#sets)*
                ::core::result::Result::Ok(table)
            }
        }
    })
}

/// True when the container carries `#[lua(tag)]`.
fn has_tag_attr(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("lua") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                found = true;
            }
            Ok(())
        });
        found
    })
}

/// Expands `#[derive(LuaPacket)]`.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    match &input.data {
        Data::Struct(s) => expand_struct(name, &input.generics, &s.fields),
        Data::Enum(e) => {
            if has_tag_attr(input) {
                expand_value_enum(name, &input.generics, e)
            } else {
                expand_opcode_enum(name, &input.generics, e)
            }
        }
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "LuaPacket does not support unions",
        )),
    }
}
