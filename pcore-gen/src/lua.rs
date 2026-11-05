//! `#[derive(Lua)]` — Lua table conversions for protocol types.
//!
//! Generates:
//!
//! - `impl IntoLua for T` — consumes the value, builds a table.
//! - `impl IntoLua for &T` — clones and delegates, so callers can pass `&self`.
//! - `impl FromLua for T` — **only** when every field is owned.
//!
//! The last restriction is not laziness: a field of type `&'a str` or
//! `&'a [u8]` borrows from the packet buffer, and a Lua string cannot be
//! borrowed for `'a` — it lives only for the duration of the call. Generating
//! `FromLua` for such a struct would either fail to compile or require leaking
//! the string, so the derive skips it and the type stays read-only from Lua.
//!
//! Field mapping:
//!
//! | Rust | Lua |
//! |---|---|
//! | `&'a str`, `String` | string |
//! | `&'a [u8]`, `Vec<u8>` | string (raw bytes, not a number table) |
//! | `Option<T>` | `nil` or `T` |
//! | `Vec<T>` | sequence table |
//! | primitives, `VarInt`, `Uuid`, … | number / string via their own impls |
//! | nested struct | table |
//!
//! Enums become `{ type = "VariantName", ... }`, with a single tuple payload
//! under `value`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

/// How a field is represented in Lua.
enum FieldKind {
    /// `&'a str` or `String` — a Lua string.
    Str,
    /// `&'a [u8]` or `Vec<u8>` — a Lua string holding raw bytes.
    Bytes,
    /// A tuple `(A, B, ...)` — a sequence table, since mlua has no blanket
    /// tuple impl and the orphan rule forbids adding one in `pcore`.
    Tuple(Vec<Type>),
    /// `Vec<(A, B, ...)>` — a sequence of sequence tables, for the same reason.
    VecTuple(Vec<Type>),
    /// Anything else: delegate to the type's own `IntoLua`/`FromLua`.
    Other,
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

/// Classifies a field type.
fn field_kind(ty: &Type) -> FieldKind {
    match ty {
        Type::Tuple(tuple) => FieldKind::Tuple(tuple.elems.iter().cloned().collect()),
        Type::Reference(reference) => match &*reference.elem {
            Type::Slice(_) => FieldKind::Bytes,
            Type::Path(path) if path.path.is_ident("str") => FieldKind::Str,
            _ => FieldKind::Other,
        },
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return FieldKind::Other;
            };
            match segment.ident.to_string().as_str() {
                "String" => FieldKind::Str,
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        // `Vec<u8>` is a byte string; `Vec<T>` is a sequence.
                        if let Type::Path(p) = inner
                            && p.path.is_ident("u8")
                        {
                            return FieldKind::Bytes;
                        }
                        // `Vec<(A, B)>` needs per-element tuple handling.
                        if let Type::Tuple(tuple) = inner {
                            return FieldKind::VecTuple(tuple.elems.iter().cloned().collect());
                        }
                    }
                    FieldKind::Other
                }
                _ => FieldKind::Other,
            }
        }
        _ => FieldKind::Other,
    }
}

/// True when the type borrows from the packet buffer, which makes `FromLua`
/// impossible without leaking.
fn is_borrowed(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return false;
            };
            // `NbtTag<'a>` and `Slot<'a>` borrow even though they are paths.
            if matches!(segment.ident.to_string().as_str(), "NbtTag" | "Slot") {
                return true;
            }
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                return args.args.iter().any(|arg| match arg {
                    syn::GenericArgument::Type(inner) => is_borrowed(inner),
                    syn::GenericArgument::Lifetime(_) => true,
                    _ => false,
                });
            }
            false
        }
        _ => false,
    }
}

/// Expression that turns `access` (e.g. `self.name` or a variant binding) into
/// a `LuaValue`.
fn into_lua_expr(field: &syn::Field, access: TokenStream) -> TokenStream {
    match field_kind(&field.ty) {
        // Raw bytes must stay a string: `&[u8]` would become a table of numbers.
        FieldKind::Str | FieldKind::Bytes => {
            quote! { ::mlua::Value::String(lua.create_string(#access)?) }
        }
        // A tuple becomes a sequence table; mlua has no blanket tuple impl.
        FieldKind::Tuple(elems) => {
            let len = elems.len();
            let sets = elems.iter().enumerate().map(|(i, _)| {
                let idx = i + 1;
                let field_index = syn::Index::from(i);
                // Build `access.N` as its own token stream: `#access.#idx` is
                // ambiguous to `quote!` because of the dot.
                let element = quote! { #access.#field_index };
                quote! {
                    table.set(#idx, ::mlua::IntoLua::into_lua(#element, lua)?)?;
                }
            });
            quote! {{
                let table = lua.create_table_with_capacity(#len, 0)?;
                #(#sets)*
                ::mlua::Value::Table(table)
            }}
        }
        // `Vec<(A, B)>`: build a sequence of two-element tables.
        FieldKind::VecTuple(elems) => {
            let len = elems.len();
            let sets = elems.iter().enumerate().map(|(i, _)| {
                let idx = i + 1;
                let field_index = syn::Index::from(i);
                let element = quote! { item.#field_index };
                quote! {
                    inner.set(#idx, ::mlua::IntoLua::into_lua(#element, lua)?)?;
                }
            });
            quote! {{
                let outer = lua.create_table()?;
                for (i, item) in #access.into_iter().enumerate() {
                    let inner = lua.create_table_with_capacity(#len, 0)?;
                    #(#sets)*
                    outer.set(i + 1, inner)?;
                }
                ::mlua::Value::Table(outer)
            }}
        }
        FieldKind::Other => quote! { ::mlua::IntoLua::into_lua(#access, lua)? },
    }
}

/// Expression that reads `#name` out of `table`.
fn from_lua_expr(field: &syn::Field) -> TokenStream {
    let name = field.ident.as_ref().expect("named field");
    let ty = &field.ty;
    quote! { table.get::<#ty>(stringify!(#name))? }
}

/// Builds the `IntoLua` body for a struct.
fn struct_into_lua(fields: &Fields) -> syn::Result<TokenStream> {
    let Fields::Named(named) = fields else {
        return Err(syn::Error::new_spanned(
            fields,
            "Lua derive requires named fields",
        ));
    };

    let mut sets = Vec::new();
    for field in &named.named {
        if is_skipped(field) {
            continue;
        }
        let name = field.ident.as_ref().expect("named field");
        let expr = into_lua_expr(field, quote! { self.#name });
        sets.push(quote! { table.set(stringify!(#name), #expr)?; });
    }

    Ok(quote! {
        let table = lua.create_table()?;
        #(#sets)*
        Ok(::mlua::Value::Table(table))
    })
}

/// Builds the `FromLua` body for a struct.
fn struct_from_lua(fields: &Fields) -> syn::Result<TokenStream> {
    let Fields::Named(named) = fields else {
        return Err(syn::Error::new_spanned(
            fields,
            "Lua derive requires named fields",
        ));
    };

    let mut inits = Vec::new();
    for field in &named.named {
        let name = field.ident.as_ref().expect("named field");
        if is_skipped(field) {
            inits.push(quote! { #name: ::core::default::Default::default(), });
            continue;
        }
        let expr = from_lua_expr(field);
        inits.push(quote! { #name: #expr, });
    }

    Ok(quote! { Ok(Self { #(#inits)* }) })
}

/// Builds the `IntoLua` body for an enum.
fn enum_into_lua(data: &syn::DataEnum) -> syn::Result<TokenStream> {
    let mut arms = Vec::new();
    for variant in &data.variants {
        let vname = &variant.ident;
        let tag = vname.to_string();
        match &variant.fields {
            Fields::Unit => arms.push(quote! {
                Self::#vname => {
                    let table = lua.create_table()?;
                    table.set("type", #tag)?;
                    ::mlua::Value::Table(table)
                }
            }),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => arms.push(quote! {
                Self::#vname(inner) => {
                    let table = lua.create_table()?;
                    table.set("type", #tag)?;
                    table.set("value", ::mlua::IntoLua::into_lua(inner, lua)?)?;
                    ::mlua::Value::Table(table)
                }
            }),
            Fields::Named(named) => {
                let mut sets = Vec::new();
                let mut bindings = Vec::new();
                for field in &named.named {
                    let fname = field.ident.as_ref().expect("named field");
                    bindings.push(quote! { #fname });
                    if is_skipped(field) {
                        continue;
                    }
                    let expr = into_lua_expr(field, quote! { #fname });
                    sets.push(quote! { table.set(stringify!(#fname), #expr)?; });
                }
                arms.push(quote! {
                    Self::#vname { #(#bindings),* } => {
                        let table = lua.create_table()?;
                        table.set("type", #tag)?;
                        #(#sets)*
                        ::mlua::Value::Table(table)
                    }
                });
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Lua derive supports at most one unnamed field per variant",
                ));
            }
        }
    }

    Ok(quote! { Ok(match self { #(#arms),* }) })
}

/// Builds the `FromLua` body for an enum.
fn enum_from_lua(data: &syn::DataEnum, name: &syn::Ident) -> syn::Result<TokenStream> {
    let mut arms = Vec::new();
    for variant in &data.variants {
        let vname = &variant.ident;
        let tag = vname.to_string();
        match &variant.fields {
            Fields::Unit => arms.push(quote! { #tag => Ok(Self::#vname), }),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                arms.push(quote! {
                    #tag => Ok(Self::#vname(table.get::<#ty>("value")?)),
                });
            }
            Fields::Named(named) => {
                let mut inits = Vec::new();
                for field in &named.named {
                    let fname = field.ident.as_ref().expect("named field");
                    if is_skipped(field) {
                        inits.push(quote! { #fname: ::core::default::Default::default(), });
                        continue;
                    }
                    let expr = from_lua_expr(field);
                    inits.push(quote! { #fname: #expr, });
                }
                arms.push(quote! {
                    #tag => Ok(Self::#vname { #(#inits)* }),
                });
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Lua derive supports at most one unnamed field per variant",
                ));
            }
        }
    }

    Ok(quote! {
        let tag: ::std::string::String = table.get("type")?;
        match tag.as_str() {
            #(#arms)*
            other => Err(::mlua::Error::runtime(format!(
                "unknown variant {other:?} for {}", stringify!(#name)
            ))),
        }
    })
}

/// Expands `#[derive(Lua)]`.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // `&T` needs a fresh lifetime declared on the impl itself; `T` may already
    // carry one, so the reference impl gets its own generics with `'lua_ref`
    // prepended.
    let mut ref_generics = input.generics.clone();
    ref_generics
        .params
        .insert(0, syn::GenericParam::Lifetime(syn::LifetimeParam::new(
            syn::Lifetime::new("'lua_ref", proc_macro2::Span::call_site()),
        )));
    let (ref_impl_generics, _, _) = ref_generics.split_for_impl();
    let ref_lifetime = quote! { 'lua_ref };

    let (into_body, from_body, borrowed) = match &input.data {
        Data::Struct(s) => {
            let borrowed = match &s.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .any(|f| !is_skipped(f) && is_borrowed(&f.ty)),
                _ => false,
            };
            (
                struct_into_lua(&s.fields)?,
                struct_from_lua(&s.fields)?,
                borrowed,
            )
        }
        Data::Enum(e) => {
            let borrowed = e.variants.iter().any(|v| match &v.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .any(|f| !is_skipped(f) && is_borrowed(&f.ty)),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .any(|f| is_borrowed(&f.ty)),
                Fields::Unit => false,
            });
            (enum_into_lua(e)?, enum_from_lua(e, name)?, borrowed)
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Lua derive does not support unions",
            ));
        }
    };

    let from_impl = if borrowed {
        // Borrowed fields cannot be rebuilt from a Lua string without leaking.
        quote! {}
    } else {
        quote! {
            impl #impl_generics ::mlua::FromLua for #name #ty_generics #where_clause {
                fn from_lua(
                    value: ::mlua::Value,
                    _lua: &::mlua::Lua,
                ) -> ::mlua::Result<Self> {
                    let table = match value {
                        ::mlua::Value::Table(table) => table,
                        other => {
                            return Err(::mlua::Error::runtime(format!(
                                "expected a table for {}, got {}",
                                stringify!(#name),
                                other.type_name()
                            )));
                        }
                    };
                    #from_body
                }
            }
        }
    };

    Ok(quote! {
        impl #impl_generics ::mlua::IntoLua for #name #ty_generics #where_clause {
            fn into_lua(self, lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Value> {
                #into_body
            }
        }

        impl #ref_impl_generics ::mlua::IntoLua for & #ref_lifetime #name #ty_generics #where_clause {
            fn into_lua(self, lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Value> {
                ::mlua::IntoLua::into_lua(::core::clone::Clone::clone(self), lua)
            }
        }

        #from_impl
    })
}
