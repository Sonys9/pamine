//! `#[derive(Decode)]` — structs from a byte decoder.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

use crate::common::decode_lifetime;

/// Expands `#[derive(Decode)]` for a struct.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let body = process_struct(&input.data)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime = decode_lifetime(input)?;

    Ok(quote! {
        impl #impl_generics ::pcore::Decode #lifetime for #name #ty_generics #where_clause {
            fn decode(decoder: &mut ::pcore::decoder::Decoder #lifetime) -> Result<Self, ::pcore::errors::Error> {
                Ok(#body)
            }
        }
    })
}

/// How a field's bytes are delimited on the wire.
enum FieldMode {
    /// Default: the type decides (VarInt prefix for `&[u8]`, etc.).
    Typed,
    /// `#[decode(rest)]` — everything up to the end of the packet.
    Rest,
    /// `#[decode(len = "field")]` — exactly `field` bytes, no prefix.
    Len(syn::Ident),
}

/// Parses `#[decode(...)]` on a field.
fn field_mode(field: &syn::Field) -> syn::Result<FieldMode> {
    let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("decode")) else {
        return Ok(FieldMode::Typed);
    };

    let mut mode = FieldMode::Typed;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rest") {
            mode = FieldMode::Rest;
            return Ok(());
        }
        if meta.path.is_ident("len") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            let ident = syn::Ident::new(&lit.value(), lit.span());
            mode = FieldMode::Len(ident);
            return Ok(());
        }
        Err(meta.error("expected `rest` or `len = \"field\"`"))
    })?;
    Ok(mode)
}

/// Generates the decode expression for one field.
///
/// Returns the expression and whether it still needs a `?`. `rest()` yields a
/// plain slice, everything else yields a `Result`.
fn field_decode(field: &syn::Field) -> syn::Result<(TokenStream, bool)> {
    Ok(match field_mode(field)? {
        FieldMode::Typed => (decode_type(&field.ty), true),
        FieldMode::Rest => (quote! { decoder.rest() }, false),
        FieldMode::Len(name) => (quote! { decoder.read_len(#name as usize) }, true),
    })
}

/// Decode a single type field, returning code to decode it.
///
/// Shared with [`decode_packet`](crate::decode_packet), which needs the same
/// per-type dispatch for its variant payloads.
pub fn decode_type_for_packet(ty: &Type) -> TokenStream {
    decode_type(ty)
}

/// Decode a single type field, returning code to decode it.
fn decode_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Reference(reference) => {
            match &*reference.elem {
                // &[u8] -> borrowed byte slice, no copy. Only `u8` is supported:
                // reinterpreting as `&[u32]` would need alignment and would keep
                // native byte order, while the protocol is big-endian.
                Type::Slice(slice) => {
                    let elem_type: &Type = &slice.elem;
                    if !matches!(elem_type, Type::Path(p) if p.path.is_ident("u8")) {
                        return quote! {
                            compile_error!("Decode only supports &[u8]; decode numeric arrays element-wise with from_be_bytes")
                        };
                    }
                    return quote! { decoder.slice() };
                }
                // &str / &'b str -> borrowed UTF-8, no copy
                Type::Path(path) if path.path.is_ident("str") => {
                    return quote! { decoder.string() };
                }
                _ => {}
            }
            // For other reference types, fallback to Decode
            quote! { ::pcore::Decode::decode(decoder) }
        }
        Type::Path(path) => {
            let segment = path
                .path
                .segments
                .last()
                .expect("Type path must have a segment");
            let name = segment.ident.to_string();

            // Handle Option<T>
            if name == "Option" {
                if let Some(inner_type) = extract_option_inner(ty) {
                    let inner_decode = decode_type(&inner_type);
                    return quote! {
                        {
                            if decoder.bool()? {
                                Ok(Some(#inner_decode?))
                            } else {
                                Ok(None)
                            }
                        }
                    };
                }
            }

            // Handle Vec<u8> - the ident alone is "Vec", so inspect the generic args
            if name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(Type::Path(inner))) = args.args.first() {
                        if inner.path.is_ident("u8") {
                            return quote! { decoder.bytes() };
                        }
                    }
                }
            }

            // Handle primitive types
            match name.as_str() {
                "i8" => return quote! { decoder.i8() },
                "u8" => return quote! { decoder.u8() },
                "bool" => return quote! { decoder.bool() },
                "i16" => return quote! { decoder.i16() },
                "u16" => return quote! { decoder.u16() },
                "i32" => return quote! { decoder.i32() },
                "u32" => return quote! { decoder.u32() },
                "i64" => return quote! { decoder.i64() },
                "u64" => return quote! { decoder.u64() },
                "f32" => return quote! { decoder.f32() },
                "f64" => return quote! { decoder.f64() },
                "Uuid" => return quote! { decoder.uuid() },
                "VarInt" => return quote! { decoder.varint() },
                "VarLong" => return quote! { decoder.varlong() },
                // Owned String: the only case that must allocate, because the
                // field type owns its data. Prefer `&'a str` to avoid the copy.
                "String" => return quote! { decoder.string().map(str::to_owned) },
                // NBT is self-describing: the tag type id is on the wire, so the
                // decoder reads it and dispatches internally.
                "NbtTag" => return quote! { decoder.nbt() },
                // Fallback: assume type implements ::pcore::Decode
                _ => return quote! { ::pcore::Decode::decode(decoder) },
            }
        }
        _ => {
            // Fallback for other types - assume they implement Decode
            quote! { ::pcore::Decode::decode(decoder) }
        }
    }
}

/// Extract the inner type from an Option<T> type.
fn extract_option_inner(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty.clone());
                    }
                }
            }
        }
    }
    None
}

/// Process a struct's fields and generate the decode token stream.
fn process_struct(data: &syn::Data) -> syn::Result<TokenStream> {
    let struct_ = match data {
        Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Decode derive requires a struct",
            ));
        }
    };

    Ok(match &struct_.fields {
        Fields::Named(fields) => {
            let mut iter = Vec::new();
            for field in &fields.named {
                let name = field.ident.as_ref().expect("Struct field must have a name");
                let (expr, needs_try) = field_decode(field)?;
                let expr = if needs_try {
                    quote! { #expr? }
                } else {
                    expr
                };
                iter.push(quote! {
                    #name: #expr,
                });
            }
            quote! { Self { #(#iter)* } }
        }
        Fields::Unit => quote! { Self },
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            // Single unnamed field - decode it directly
            let (expr, needs_try) = field_decode(&fields.unnamed[0])?;
            if needs_try {
                quote! { Self(#expr?) }
            } else {
                quote! { Self(#expr) }
            }
        }
        Fields::Unnamed(fields) => {
            // Multiple unnamed fields - decode each one in order
            let mut iter = Vec::new();
            for field in &fields.unnamed {
                let (expr, needs_try) = field_decode(field)?;
                let expr = if needs_try {
                    quote! { #expr? }
                } else {
                    expr
                };
                iter.push(quote! { #expr, });
            }
            quote! { Self(#(#iter)*) }
        }
    })
}
