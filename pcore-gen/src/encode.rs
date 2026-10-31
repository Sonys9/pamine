//! `#[derive(Encode)]` — structs into a byte buffer.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Expands `#[derive(Encode)]` for a struct.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let body = process_struct(&input.data)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::pcore::Encode for #name #ty_generics #where_clause {
            fn encode(&self, encoder: &mut ::pcore::encoder::Encoder) {
                #body
            }
        }
    })
}

/// How a field's bytes are delimited on the wire, for encoding.
enum FieldMode {
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
fn field_mode(field: &syn::Field) -> syn::Result<FieldMode> {
    let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("encode")) else {
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

/// Generates the encode statement for one field.
///
/// `access` is the expression that yields the field's value, e.g. `&self.name`
/// or `name` for a destructured enum variant.
fn field_encode(field: &syn::Field, access: TokenStream) -> syn::Result<TokenStream> {
    Ok(match field_mode(field)? {
        FieldMode::Typed => quote! { ::pcore::Encode::encode(#access, encoder); },
        // `rest` and `len` both write the raw bytes with no length prefix.
        FieldMode::Rest | FieldMode::Len(_) => {
            quote! { encoder.raw(#access); }
        }
    })
}

/// Process a struct's fields and generate the encode body.
fn process_struct(data: &syn::Data) -> syn::Result<TokenStream> {
    let struct_ = match data {
        Data::Struct(s) => s,
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
