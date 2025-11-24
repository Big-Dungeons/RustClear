mod packet_serializable;
mod packet_deserializable;
mod entity_metadata;
mod blocks;

use crate::packet_deserializable::packet_deserializable_macro;
use crate::packet_serializable::packet_serializable_macro;
use crate::{blocks::blocks_macro, entity_metadata::entity_metadata_serializable_macro};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro]
pub fn packet_serializable(input: TokenStream) -> TokenStream {
    packet_serializable_macro(input)
}

#[proc_macro]
pub fn packet_deserializable(input: TokenStream) -> TokenStream {
    packet_deserializable_macro(input)
}

#[proc_macro]
pub fn entity_metadata_serializable(input: TokenStream) -> TokenStream {
    entity_metadata_serializable_macro(input)
}

#[proc_macro]
pub fn blocks(input: TokenStream) -> TokenStream {
    blocks_macro(input)
}

/// This macro is used to generate a BlockMetadata impl for enums.
#[proc_macro_derive(BlockMetadata)]
pub fn derive_block_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(e) => &e.variants,
        _ => panic!("BlockMetadata can only be derived for enums"),
    };

    let mut max_discriminant = 0u64;
    let mut match_arms = vec![];

    for (index, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;

        let value = if let Some((_, expr)) = &variant.discriminant {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = expr
            {
                lit_int.base10_parse::<u64>().unwrap()
            } else {
                panic!("Unsupported discriminant expression for {:?}", ident);
            }
        } else {
            index as u64
        };

        max_discriminant = max_discriminant.max(value);
        match_arms.push(quote! {
            #value => #name::#ident,
        });
    }

    let fallback = &variants.last().unwrap().ident;
    let meta_size = (max_discriminant + 1).next_power_of_two().trailing_zeros() as u8;

    let expanded = quote! {
        impl BlockMetadata for #name {
            fn meta_size() -> u8 {
                #meta_size
            }

            fn get_meta(&self) -> u8 {
                *self as u8
            }

            fn from_meta(meta: u8) -> Self {
                match meta as u64 {
                    #(#match_arms)*
                    _ => #name::#fallback,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
