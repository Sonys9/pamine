//! `#[derive(DecodePacket)]` — opcode-tagged enums.

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::common::{check_duplicate_id, check_single_unnamed, decode_lifetime, variant_id};

/// Expands `#[derive(DecodePacket)]` for an enum.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
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
        let opcode = variant_id(variant)?;
        check_duplicate_id(&mut seen, &opcode, variant)?;
        check_single_unnamed(variant)?;

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let inner_ty = &fields.unnamed[0].ty;
                let inner_decode = crate::decode::decode_type_for_packet(inner_ty);
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
                    let decoder = crate::decode::decode_type_for_packet(field_ty);
                    quote! { #field_name: #decoder?, }
                });
                match_arms.push(quote! {
                    #opcode => {
                        let inner = Self::#variant_name { #(#field_decoders)* };
                        Ok(inner)
                    },
                });
            }
            Fields::Unnamed(_) => unreachable!("check_single_unnamed rejects this"),
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
