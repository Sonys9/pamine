//! `#[derive(LuaPacket)]` — Lua constructors for packets.
//!
//! `#[derive(Lua)]` turns a packet into a table, which is what a handler
//! *reads*. This derive is the other direction: it lets a script *build* a
//! packet and hand it back for injection.
//!
//! Two shapes, because the packet types are two shapes:
//!
//! - **On a struct** it generates `lua_encode(table, encoder)`, which reads the
//!   named fields out of a Lua table and writes them into an encoder.
//! - **On an enum** it generates `lua_bindings(lua)`, a table of constructors
//!   keyed by variant name. Each constructor takes one table, calls the inner
//!   type's `lua_encode`, and returns a [`LuaPacketValue`] carrying the
//!   variant's `#[id(N)]` opcode.
//!
//! ```lua
//! local p = s2c.status.StatusResponse { json_response = "{}" }
//! -- p.opcode == 0, p.body == <encoded fields>
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
//! | `Option<T>` | `nil` or `T` | `encoder.bool(v.is_some())` + value |
//! | `Vec<T>` | sequence | `encoder.varint(len)` + each |
//! | `VarInt` | integer | `encoder.varint(v)` |
//! | `Uuid` | 16-byte string | `encoder.uuid(&v)` |
//! | `NbtTag` | table | `encoder.nbt(&v)` |
//! | `Slot` | table | `encoder.slot(&v)` |
//! | primitives | number/bool | `encoder.<type>(v)` |
//! | nested struct | table | `Inner::lua_encode(&v, encoder)` |

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

use crate::common::{check_duplicate_id, check_single_unnamed, variant_id};

/// How a field is read from Lua and written to the encoder.
enum FieldKind {
    Str,
    Bytes,
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
    /// Anything else: assume it has `lua_encode`.
    Other,
}

fn field_kind(ty: &Type) -> FieldKind {
    match ty {
        Type::Reference(reference) => match &*reference.elem {
            Type::Slice(_) => FieldKind::Bytes,
            Type::Path(path) if path.path.is_ident("str") => FieldKind::Str,
            _ => FieldKind::Other,
        },
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return FieldKind::Other;
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
            FieldKind::Other
        }
        _ => FieldKind::Other,
    }
}

/// The Lua type a field is read as.
fn lua_type(ty: &Type) -> TokenStream {
    match field_kind(ty) {
        FieldKind::Str => quote! { ::std::string::String },
        FieldKind::Bytes => quote! { ::mlua::String },
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
            let inner = lua_type(&inner);
            quote! { ::core::option::Option<#inner> }
        }
        FieldKind::Sequence(inner) => {
            let inner = lua_type(&inner);
            quote! { ::std::vec::Vec<#inner> }
        }
        FieldKind::Other => quote! { #ty },
    }
}

/// Statements that write one field into `encoder`.
fn encode_field(ty: &Type, value: &TokenStream) -> TokenStream {
    match field_kind(ty) {
        FieldKind::Str => quote! { encoder.string(&#value); },
        FieldKind::Bytes => quote! { encoder.bytes(&#value); },
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
            let inner_encode = encode_field(&inner, &quote! { inner_value });
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
            let inner_encode = encode_field(&inner, &quote! { item });
            quote! {
                encoder.varint(#value.len() as i32);
                for item in #value {
                    #inner_encode
                }
            }
        }
        FieldKind::Other => quote! { #value.lua_encode(encoder)?; },
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

    let mut reads = Vec::new();
    for field in &named.named {
        let field_name = field.ident.as_ref().expect("named field");
        let ty = lua_type(&field.ty);
        let key = field_name.to_string();
        let encode = encode_field(&field.ty, &quote! { #field_name });
        reads.push(quote! {
            let #field_name: #ty = table.get(#key)?;
            #encode
        });
    }

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

/// Expands `#[derive(LuaPacket)]` for an enum: `lua_bindings`.
fn expand_enum(
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
                "LuaPacket enum variants must be tuple variants wrapping a struct",
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

/// Expands `#[derive(LuaPacket)]`.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    match &input.data {
        Data::Struct(s) => expand_struct(name, &input.generics, &s.fields),
        Data::Enum(e) => expand_enum(name, &input.generics, e),
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "LuaPacket does not support unions",
        )),
    }
}
