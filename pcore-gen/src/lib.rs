use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Meta, Type};

#[proc_macro_derive(Decode)]
pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;

    let Data::Struct(struct_) = input.data else {
        panic!("Bad data type");
    };

    let decode = process_struct(struct_);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime = match input.generics.lifetimes().next().is_some() {
        true => quote! { #ty_generics },
        false => quote! { <'_> },
    };

    TokenStream::from(quote! {
        impl #impl_generics ::pcore::Decode #lifetime for #name #ty_generics #where_clause {
            fn decode(decoder: &mut ::pcore::decoder::Decoder #lifetime) -> Result<Self, ::pcore::errors::Error> {
                Ok(#decode)
            }
        }
    })
}

fn process_struct(struct_: DataStruct) -> proc_macro2::TokenStream {
    match struct_.fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().map(|field| {
                let name = field
                    .ident
                    .as_ref()
                    .expect("Struct's field doesn't contains a name");
                let type_ = &field.ty;
                let match_ = match_types(type_);
                quote! {
                    #name: #match_?,
                }
            });
            quote! { Self { #(#iter)* } }
        }
        Fields::Unit => quote! { Self },
        Fields::Unnamed(_) => panic!("Unsupported struct type"),
    }
}

fn match_types(type_: &Type) -> proc_macro2::TokenStream {
    match type_ {
        Type::Reference(_) => {
            let type_str = quote! { #type_ }.to_string().replace(" ", "");
            if type_str.ends_with("[u8]") {
                quote! { decoder.slice() }
            } else {
                panic!("Unsupported reference type: {}", type_str);
            }
        }
        Type::Path(path) => {
            let segment = match path.path.segments.last() {
                Some(seg) => seg,
                None => panic!("Empty path"),
            };
            let name = segment.ident.to_string();

            if name == "Option" {
                let inner_type = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        match args.args.first() {
                            Some(syn::GenericArgument::Type(ty)) => ty,
                            _ => panic!("Expected type inside Option<>"),
                        }
                    }
                    _ => panic!("Expected angle bracketed arguments for Option"),
                };

                let inner_decode = match_types(inner_type);
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

            match name.as_str() {
                "i8" => quote! { decoder.i8() },
                "u8" => quote! { decoder.u8() },
                "bool" => quote! { decoder.bool() },
                "u16" => quote! { decoder.u16() },
                "i16" => quote! { decoder.i16() },
                "i32" => quote! { decoder.i32() },
                "u64" => quote! { decoder.u64() },
                "f32" => quote! { decoder.f32() },
                "f64" => quote! { decoder.f64() },
                "i64" => quote! { decoder.i64() },
                "Uuid" => quote! {
                    Ok(::pcore::decoder::Uuid::new(
                        decoder.uuid()?.try_into().unwrap(),
                    ))
                },
                "VarInt" => quote! { decoder.varint() },
                _ => quote! { ::pcore::Decode::decode(decoder) },
            }
        }
        _ => {
            let type_str = quote! { #type_ }.to_string();
            panic!("Unknown field type: {}", type_str)
        }
    }
}

#[proc_macro_derive(DecodePacket, attributes(id))]
pub fn derive_packet_enum(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        panic!("Expected enum not other type");
    };

    let mut match_arms = Vec::new();

    for variant in data_enum.variants {
        let variant_name = &variant.ident;

        let mut opcode_lit = None;
        for attr in &variant.attrs {
            if attr.path().is_ident("id") {
                if let Meta::List(meta_list) = &attr.meta {
                    opcode_lit = Some(meta_list.tokens.clone());
                }
            }
        }

        let opcode = opcode_lit.expect("Invalid enum");

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                match_arms.push(quote! {
                    #opcode => {
                        let inner = Decode::decode(decoder)?;
                        Ok(Self::#variant_name(inner))
                    }
                });
            }
            _ => panic!("Invalid enum"),
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime = match input.generics.lifetimes().next().is_some() {
        true => quote! { #ty_generics },
        false => quote! { <'_> },
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