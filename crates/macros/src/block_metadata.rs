use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, ExprLit, Lit};

pub fn block_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(e) => &e.variants,
        _ => panic!("BlockMetadata can only be derived for enums"),
    };

    let mut largest_discriminant = 0;
    let mut match_arms: proc_macro2::TokenStream = Default::default();

    for (index, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;

        let value = if let Some((_, expr)) = &variant.discriminant {
            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                lit_int.base10_parse::<u64>().unwrap()
            } else {
                panic!("Unsupported discriminant expression for {:?}", ident);
            }
        } else {
            index as u64
        };

        largest_discriminant = largest_discriminant.max(value);
        match_arms.extend(quote! {
            #value => Self::#ident,
        })
    }

    let fallback = &variants.last().unwrap().ident;
    let meta_size = (usize::BITS - (largest_discriminant - 1).leading_zeros()) as u8;

    quote! {
        impl BlockMetadata for #ident {
            const META_SIZE: u8 = #meta_size;

            fn get_meta(&self) -> u8 {
                *self as u8
            }

            fn from_meta(meta: u8) -> Self {
                match meta as u64 {
                    #match_arms
                    _ => Self::#fallback,
                }
            }
        }
    }
    .into()
}
