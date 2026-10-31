//! `#[derive(EncodePacket)]` — opcode-tagged enums.

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::common::{check_duplicate_id, check_single_unnamed, variant_id};

/// Expands `#[derive(EncodePacket)]` for an enum.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "EncodePacket derive requires an enum",
        ));
    };

    let mut match_arms = Vec::new();
    // `Self::Variant(..) => 0x05` arms for the generated `opcode()` method.
    let mut opcode_arms = Vec::new();
    let mut seen: HashMap<String, String> = HashMap::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let opcode = variant_id(variant)?;
        check_duplicate_id(&mut seen, &opcode, variant)?;
        check_single_unnamed(variant)?;

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                match_arms.push(quote! {
                    Self::#variant_name(inner) => {
                        encoder.varint(#opcode);
                        ::pcore::Encode::encode(inner, encoder);
                    }
                });
                opcode_arms.push(quote! { Self::#variant_name(..) => #opcode, });
            }
            Fields::Unit => {
                match_arms.push(quote! {
                    Self::#variant_name => {
                        encoder.varint(#opcode);
                    }
                });
                opcode_arms.push(quote! { Self::#variant_name => #opcode, });
            }
            Fields::Named(named) => {
                let names: Vec<_> = named
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().expect("named field"))
                    .collect();
                match_arms.push(quote! {
                    Self::#variant_name { #(#names),* } => {
                        encoder.varint(#opcode);
                        #(::pcore::Encode::encode(#names, encoder);)*
                    }
                });
                opcode_arms.push(quote! { Self::#variant_name { .. } => #opcode, });
            }
            Fields::Unnamed(_) => unreachable!("check_single_unnamed rejects this"),
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::pcore::Encode for #name #ty_generics #where_clause {
            fn encode(&self, encoder: &mut ::pcore::encoder::Encoder) {
                match self {
                    #(#match_arms)*
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// The wire opcode of this variant, taken from its `#[id(N)]`.
            ///
            /// [`Encode`](::pcore::Encode) already writes this value as the
            /// first VarInt of the packet; this method exposes it for logging,
            /// filtering and building frames by hand.
            pub fn opcode(&self) -> i32 {
                match self {
                    #(#opcode_arms)*
                }
            }
        }
    })
}
