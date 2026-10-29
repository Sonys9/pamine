use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, Meta, Type, parse_macro_input};

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
#[proc_macro_derive(Decode)]
pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match expand_decode(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let decode = process_struct(&input.data)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime = decode_lifetime(input)?;

    Ok(quote! {
        impl #impl_generics ::pcore::Decode #lifetime for #name #ty_generics #where_clause {
            fn decode(decoder: &mut ::pcore::decoder::Decoder #lifetime) -> Result<Self, ::pcore::errors::Error> {
                Ok(#decode)
            }
        }
    })
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
pub fn derive_packet_enum(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match expand_decode_packet(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_decode_packet(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "DecodePacket derive requires an enum",
        ));
    };

    let mut match_arms = Vec::new();
    // opcode literal -> variant name, used to reject duplicates
    let mut seen: HashMap<String, String> = HashMap::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        let opcode = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("id"))
            .and_then(|attr| match &attr.meta {
                Meta::List(list) => Some(list.tokens.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    variant_name,
                    format!("variant `{variant_name}` is missing a #[id(N)] attribute"),
                )
            })?;

        let key = opcode.to_string();
        if let Some(previous) = seen.insert(key.clone(), variant_name.to_string()) {
            return Err(syn::Error::new_spanned(
                variant_name,
                format!("duplicate #[id({key})]: already used by variant `{previous}`"),
            ));
        }

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let inner_ty = &fields.unnamed[0].ty;
                let inner_decode = decode_type(inner_ty);
                match_arms.push(quote! {
                    #opcode => {
                        let inner = #inner_decode?;
                        Ok(Self::#variant_name(inner))
                    },
                });
            }
            Fields::Unit => {
                match_arms.push(quote! {
                    #opcode => Ok(Self::#variant_name),
                });
            }
            Fields::Named(named) => {
                let field_decoders = named.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_ty = &field.ty;
                    let decoder = decode_type(field_ty);
                    quote! { #field_name: #decoder?, }
                });
                match_arms.push(quote! {
                    #opcode => {
                        let inner = Self::#variant_name { #(#field_decoders)* };
                        Ok(inner)
                    },
                });
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant_name,
                    format!(
                        "variant `{variant_name}` has multiple unnamed fields; \
                         use a single field or a struct variant"
                    ),
                ));
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime = decode_lifetime(input)?;

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn decode_by_opcode(opcode: i32, decoder: &mut ::pcore::decoder::Decoder #lifetime) -> Result<Self, ::pcore::errors::Error> {
                match opcode {
                    #(#match_arms)*
                    _ => Err(::pcore::errors::Error::UnknownOpcode(opcode))
                }
            }
        }
    })
}

/// Builds the lifetime argument for `Decode<...>`.
///
/// `Decode` takes exactly one lifetime, so a type with zero lifetimes gets
/// `'_` and a type with one lifetime reuses it. More than one is rejected
/// instead of silently emitting invalid code.
fn decode_lifetime(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut lifetimes = input.generics.lifetimes();
    match (lifetimes.next(), lifetimes.next()) {
        (None, _) => Ok(quote! { <'_> }),
        (Some(first), None) => {
            let lt = &first.lifetime;
            Ok(quote! { <#lt> })
        }
        (Some(_), Some(second)) => Err(syn::Error::new_spanned(
            &second.lifetime,
            "Decode supports at most one lifetime parameter",
        )),
    }
}

/// Decode a single type field, returning TokenStream code to decode it.
fn decode_type(ty: &Type) -> proc_macro2::TokenStream {
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
fn process_struct(data: &syn::Data) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ = match data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Decode derive requires a struct",
            ));
        }
    };

    Ok(match &struct_.fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().map(|field| {
                let name = field
                    .ident
                    .as_ref()
                    .expect("Struct field must have a name");
                let type_ = &field.ty;
                let match_ = decode_type(type_);
                quote! {
                    #name: #match_?,
                }
            });
            quote! { Self { #(#iter)* } }
        }
        Fields::Unit => quote! { Self },
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            // Single unnamed field - decode it directly
            let field = &fields.unnamed[0];
            let type_ = &field.ty;
            let match_ = decode_type(type_);
            quote! { Self(#match_?) }
        }
        Fields::Unnamed(fields) => {
            // Multiple unnamed fields - decode each one in order
            let iter = fields.unnamed.iter().map(|field| {
                let type_ = &field.ty;
                let match_ = decode_type(type_);
                quote! { #match_?, }
            });
            quote! { Self(#(#iter)*) }
        }
    })
}
