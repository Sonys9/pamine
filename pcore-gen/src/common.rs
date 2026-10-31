//! Helpers shared by the decode and encode derives.

use std::collections::HashMap;
use syn::{DeriveInput, Fields, Meta};

/// Builds the lifetime argument for `Decode<...>`.
///
/// `Decode` takes exactly one lifetime, so a type with zero lifetimes gets
/// `'_` and a type with one lifetime reuses it. More than one is rejected
/// instead of silently emitting invalid code.
pub fn decode_lifetime(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut lifetimes = input.generics.lifetimes();
    match (lifetimes.next(), lifetimes.next()) {
        (None, _) => Ok(quote::quote! { <'_> }),
        (Some(first), None) => {
            let lt = &first.lifetime;
            Ok(quote::quote! { <#lt> })
        }
        (Some(_), Some(second)) => Err(syn::Error::new_spanned(
            &second.lifetime,
            "Decode supports at most one lifetime parameter",
        )),
    }
}

/// Reads the `#[id(N)]` attribute of an enum variant.
///
/// Returns the raw token stream of `N`, so the caller can paste it into a
/// `match` arm or a `write_varint` call without re-parsing it.
pub fn variant_id(variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream> {
    let name = &variant.ident;
    variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("id"))
        .and_then(|attr| match &attr.meta {
            Meta::List(list) => Some(list.tokens.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(
                name,
                format!("variant `{name}` is missing a #[id(N)] attribute"),
            )
        })
}

/// Rejects two variants that share an opcode.
///
/// `seen` maps the opcode literal to the variant that claimed it first.
pub fn check_duplicate_id(
    seen: &mut HashMap<String, String>,
    opcode: &proc_macro2::TokenStream,
    variant: &syn::Variant,
) -> syn::Result<()> {
    let key = opcode.to_string();
    if let Some(previous) = seen.insert(key.clone(), variant.ident.to_string()) {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            format!("duplicate #[id({key})]: already used by variant `{previous}`"),
        ));
    }
    Ok(())
}

/// Rejects a variant with more than one unnamed field.
///
/// Both derives only know how to handle a single payload, so a tuple variant
/// with two fields is a mistake rather than something to guess at.
pub fn check_single_unnamed(variant: &syn::Variant) -> syn::Result<()> {
    if let Fields::Unnamed(fields) = &variant.fields {
        if fields.unnamed.len() != 1 {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                format!(
                    "variant `{}` has multiple unnamed fields; \
                     use a single field or a struct variant",
                    variant.ident
                ),
            ));
        }
    }
    Ok(())
}
