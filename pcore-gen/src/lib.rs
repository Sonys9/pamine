use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, Meta, Type,
    parse_macro_input,
};

/// Proc macro derive for decoding structs from a byte decoder.
///
/// Generated implementation:
/// ```rust
/// impl ::pcore::Decode<'_> for MyStruct {
///     fn decode(decoder: &mut ::pcore::decoder::Decoder<'_>) -> Result<Self, ::pcore::errors::Error> {
///         Ok(Self { field1: decoder.varint()?, field2: decoder.u8()? })
///     }
/// }
/// ```
#[proc_macro_derive(Decode)]
pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;

    let decode = process_struct(&input.data);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime = match input.generics.lifetimes().next() {
        Some(_) => quote! { #ty_generics },
        None => quote! { <'_> },
    };

    TokenStream::from(quote! {
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
/// ```rust
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
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        panic!("DecodePacket derive requires an enum");
    };

    let mut match_arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        // Extract optional opcode attribute
        let opcode = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("id"))
            .and_then(|attr| {
                if let Meta::List(list) = &attr.meta {
                    Some(list.tokens.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                // Default opcode: use variant index as fallback
                panic!("Variant {} missing #[id(N)] attribute", variant_name)
            });

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let inner_ty = &fields.unnamed.iter().next().unwrap().ty;
                let inner_decode = decode_type(inner_ty);
                match_arms.push(quote! {
                    #opcode => {
                        let inner = #inner_decode?;
                        Ok(Self::#variant_name(inner))
                    }
                });
            }
            _ => {
                // Support unit variants and other variants
                match &variant.fields {
                    Fields::Unit => {
                        match_arms.push(quote! {
                            #opcode => Ok(Self::#variant_name)
                        });
                    }
                    _ => {
                        // For struct variants with named fields, we decode each field
                        if let Fields::Named(named) = &variant.fields {
                            let field_decoders = named
                                .named
                                .iter()
                                .map(|field| {
                                    let name = &field.ident;
                                    let field_ty = &field.ty;
                                    let decoder = decode_type(field_ty);
                                    quote! { #name: #decoder? }
                                });
                            match_arms.push(quote! {
                                #opcode => {
                                    let inner = Self::#variant_name { #(#field_decoders)* };
                                    Ok(inner)
                                }
                            });
                        } else {
                            // Fallback for other field types
                            match_arms.push(quote! {
                                #opcode => {
                                    let inner = ::pcore::Decode::decode(decoder)?;
                                    Ok(Self::#variant_name(inner))
                                }
                            });
                        }
                    }
                }
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime = match input.generics.lifetimes().next() {
        Some(_) => quote! { #ty_generics },
        None => quote! { <'_> },
    };

    TokenStream::from(quote! {
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

/// Decode a single type field, returning TokenStream code to decode it.
fn decode_type(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Reference(reference) => {
            // Handle &[T] references - use decoder.slice()
            if let Type::Slice(slice) = &*reference.elem {
                let elem_type = &slice.elem;
                return quote! { decoder.slice::<#elem_type>() };
            }
            // For other reference types, fallback to Decode
            quote! { ::pcore::Decode::decode(decoder) }
        }
        Type::Path(path) => {
            let segment = path.path.segments.last().expect("Type path must have a segment");
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
                "String" => return quote! { decoder.string() },
                "Vec<u8>" => return quote! { decoder.bytes() },
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
fn process_struct(data: &syn::Data) -> proc_macro2::TokenStream {
    let struct_ = match data {
        syn::Data::Struct(s) => s,
        _ => panic!("Decode derive requires a struct"),
    };

    match &struct_.fields {
        Fields::Named(fields) => {
            let iter = fields
                .named
                .iter()
                .map(|field| {
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
            quote! { Self(#match_) }
        }
        Fields::Unnamed(_) => {
            // Multiple unnamed fields - not supported yet, fallback to Decode
            quote! { ::pcore::Decode::decode(decoder) }
        }
    }
}