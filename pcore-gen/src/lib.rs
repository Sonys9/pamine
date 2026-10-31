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
#[proc_macro_derive(Decode, attributes(decode))]
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

/// Proc macro derive for encoding structs into a byte buffer.
///
/// The mirror of [`Decode`]. Generated implementation:
/// ```text
/// impl ::pcore::Encode for MyStruct {
///     fn encode(&self, out: &mut Vec<u8>) {
///         ::pcore::Encode::encode(&self.field1, out);
///         ::pcore::Encode::encode(&self.field2, out);
///     }
/// }
/// ```
#[proc_macro_derive(Encode, attributes(encode))]
pub fn derive_encode(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match expand_encode(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_encode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let body = process_struct_encode(&input.data)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::pcore::Encode for #name #ty_generics #where_clause {
            fn encode(&self, out: &mut Vec<u8>) {
                #body
            }
        }
    })
}

/// Proc macro derive for encoding packet enums by opcode.
///
/// The mirror of [`DecodePacket`]. Generated implementation:
/// ```text
/// impl MyEnum {
///     pub fn encode(&self, out: &mut Vec<u8>) {
///         match self {
///             Self::Variant1(inner) => {
///                 ::pcore::encoder::write_varint(out, 0);
///                 ::pcore::Encode::encode(inner, out);
///             }
///             ...
///         }
///     }
/// }
/// ```
#[proc_macro_derive(EncodePacket, attributes(id))]
pub fn derive_encode_packet(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    match expand_encode_packet(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_encode_packet(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "EncodePacket derive requires an enum",
        ));
    };

    let mut match_arms = Vec::new();
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
                match_arms.push(quote! {
                    Self::#variant_name(inner) => {
                        ::pcore::encoder::write_varint(out, #opcode);
                        ::pcore::Encode::encode(inner, out);
                    }
                });
            }
            Fields::Unit => {
                match_arms.push(quote! {
                    Self::#variant_name => {
                        ::pcore::encoder::write_varint(out, #opcode);
                    }
                });
            }
            Fields::Named(named) => {
                let names: Vec<_> = named
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().expect("named field"))
                    .collect();
                match_arms.push(quote! {
                    Self::#variant_name { #(#names),* } => {
                        ::pcore::encoder::write_varint(out, #opcode);
                        #(::pcore::Encode::encode(#names, out);)*
                    }
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

    Ok(quote! {
        impl #impl_generics ::pcore::Encode for #name #ty_generics #where_clause {
            fn encode(&self, out: &mut Vec<u8>) {
                match self {
                    #(#match_arms)*
                }
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
fn field_decode(field: &syn::Field) -> syn::Result<(proc_macro2::TokenStream, bool)> {
    Ok(match field_mode(field)? {
        FieldMode::Typed => (decode_type(&field.ty), true),
        FieldMode::Rest => (quote! { decoder.rest() }, false),
        FieldMode::Len(name) => (quote! { decoder.read_len(#name as usize) }, true),
    })
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

// ---------------------------------------------------------------------------
// Encode
// ---------------------------------------------------------------------------

/// How a field's bytes are delimited on the wire, for encoding.
enum EncodeMode {
    /// Default: the type decides (VarInt prefix for `&[u8]`, etc.).
    Typed,
    /// `#[encode(rest)]` — everything up to the end of the packet.
    Rest,
    /// `#[encode(len = "field")]` — exactly `field` bytes, no prefix.
    ///
    /// The length is implied by the field itself, so the identifier is only
    /// validated at parse time and not used when emitting code.
    Len(#[allow(dead_code)] syn::Ident),
}

/// Parses `#[encode(...)]` on a field.
fn encode_field_mode(field: &syn::Field) -> syn::Result<EncodeMode> {
    let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("encode")) else {
        return Ok(EncodeMode::Typed);
    };

    let mut mode = EncodeMode::Typed;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rest") {
            mode = EncodeMode::Rest;
            return Ok(());
        }
        if meta.path.is_ident("len") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            let ident = syn::Ident::new(&lit.value(), lit.span());
            mode = EncodeMode::Len(ident);
            return Ok(());
        }
        Err(meta.error("expected `rest` or `len = \"field\"`"))
    })?;
    Ok(mode)
}

/// Generates the encode statement for one field.
///
/// `access` is the expression that yields the field's value, e.g. `&self.name`
/// or `name` for a destructured enum variant.
fn field_encode(
    field: &syn::Field,
    access: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    Ok(match encode_field_mode(field)? {
        EncodeMode::Typed => quote! { ::pcore::Encode::encode(#access, out); },
        // `rest` and `len` both write the raw bytes with no length prefix.
        EncodeMode::Rest | EncodeMode::Len(_) => {
            quote! { out.extend_from_slice(#access); }
        }
    })
}

/// Process a struct's fields and generate the encode body.
fn process_struct_encode(data: &syn::Data) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ = match data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Encode derive requires a struct",
            ));
        }
    };

    Ok(match &struct_.fields {
        Fields::Named(fields) => {
            let mut iter = Vec::new();
            for field in &fields.named {
                let name = field.ident.as_ref().expect("Struct field must have a name");
                iter.push(field_encode(field, quote! { &self.#name })?);
            }
            quote! { #(#iter)* }
        }
        Fields::Unit => quote! {},
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            field_encode(&fields.unnamed[0], quote! { &self.0 })?
        }
        Fields::Unnamed(fields) => {
            let mut iter = Vec::new();
            for (index, field) in fields.unnamed.iter().enumerate() {
                let index = syn::Index::from(index);
                iter.push(field_encode(field, quote! { &self.#index })?);
            }
            quote! { #(#iter)* }
        }
    })
}
