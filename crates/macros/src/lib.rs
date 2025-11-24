mod packet_serializable;
mod packet_deserializable;
mod entity_metadata;
mod blocks;

use crate::packet_deserializable::packet_deserializable_macro;
use crate::packet_serializable::packet_serializable_macro;
use crate::{blocks::blocks_macro, entity_metadata::entity_metadata_serializable_macro};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Ident, ItemEnum};

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

// todo: new file for block stuff
// at that point im redoing the macro
// because it is really annoying if a block has a niche metadata serializations
// and i don't want to have a bunch of direction enum copies,
// also want to be able to specify vanilla block toughness

type TokenStream2 = proc_macro2::TokenStream;

///
/// An array of rotatable_types, since it isn't possible to infer if something implements a trait.
/// 
const ROTATABLE_TYPES: &[&str] = &[
    "HorizontalDirection",
    "ButtonDirection",
    "StairDirection", 
    "Direction",
    "Axis"
];

#[proc_macro]
pub fn block_macro(input: TokenStream) -> TokenStream {
    let input_enum = parse_macro_input!(input as ItemEnum);
    let enum_name = &input_enum.ident;

    let (get_arms, from_arms) = generate_arms(enum_name, &input_enum);
    let rotate_arms = build_rotate(enum_name, &input_enum);

    let expanded = quote! {
        use crate::block::metadata::BlockMetadata;
        use crate::block::rotatable::Rotatable;

        #input_enum

        impl #enum_name {
            pub fn get_block_state_id(&self) -> u16 {
                match self {
                    #(#get_arms),*
                }
            }

            pub fn rotate(&mut self, new_direction: Direction) {
                match self {
                    #(#rotate_arms),*
                }
            }
        }

        impl From<u16> for #enum_name {
            fn from(v: u16) -> Self {
                let index = v >> 4;
                match index {
                    #(#from_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_arms(
    enum_name: &Ident,
    data_enum: &ItemEnum,
) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut get_arms = Vec::new();
    let mut from_arms = Vec::new();

    for (block_id, variant) in data_enum.variants.iter().enumerate() {
        let variant_ident = &variant.ident;
        let id_lit = syn::LitInt::new(&block_id.to_string(), proc_macro2::Span::call_site());

        match &variant.fields {
            Fields::Named(fields_named) => {
                let field_names: Vec<_> = fields_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let get_fields = fields_named
                    .named
                    .iter()
                    .map(|f| get_field_handler(f))
                    .collect::<Vec<_>>();

                let from_fields = fields_named
                    .named
                    .iter()
                    .map(|f| from_field_handler(f))
                    .collect::<Vec<_>>();

                get_arms.push(quote! {
                    #enum_name::#variant_ident { #(#field_names),* } => {
                        let mut meta: u8 = 0;
                        let mut offset: u8 = 0;
                        #( #get_fields )*
                        ((#block_id as u16) << 4) | (meta as u16)
                    }
                });

                from_arms.push(quote! {
                    #id_lit => {
                        let meta = (v & 0x0F) as u8;
                        let mut offset: u8 = 0;
                        #( #from_fields )*
                        #enum_name::#variant_ident { #(#field_names),* }
                    }
                });
            }
            Fields::Unit => {
                get_arms.push(quote! {
                    #enum_name::#variant_ident => ((#block_id as u16) << 4)
                });

                from_arms.push(quote! {
                    #id_lit => #enum_name::#variant_ident
                });
            }
            _ => panic!("unsupported variant form"),
        }
    }

    from_arms.push(quote! { _ => Blocks::Air });

    (get_arms, from_arms)
}

fn from_field_handler(field: &syn::Field) -> TokenStream2 {
    let name = field.ident.as_ref().unwrap();
    if let syn::Type::Path(type_path) = &field.ty {
        if type_path.path.is_ident("bool") {
            quote! {
                let #name = ((meta >> offset) & 0x01) != 0;
                offset += 1;
            }
        } else if type_path.path.is_ident("u8") {
            quote! {
                let #name = ((meta >> offset) & 0x0F) as u8;
                offset += 4;
            }
        } else {
            let ty = &type_path.path;
            quote! {
                let #name = <#ty>::from_meta(((meta >> offset) & ((1 << <#ty>::meta_size()) - 1)) as u8);
                offset += <#ty>::meta_size();
            }
        }
    } else {
        panic!("unsupported field type");
    }
}

fn get_field_handler(field: &syn::Field) -> TokenStream2 {
    let name = field.ident.as_ref().unwrap();
    if let syn::Type::Path(type_path) = &field.ty {
        if type_path.path.is_ident("bool") {
            quote! {
                meta |= (u8::from(*#name)) << offset;
                offset += 1;
            }
        } else if type_path.path.is_ident("u8") {
            quote! {
                meta |= (#name & 0x0F) << offset;
                offset += 4;
            }
        } else {
            let ty = &type_path.path;
            quote! {
                meta |= #name.get_meta() << offset;
                offset += <#ty>::meta_size();
            }
        }
    } else {
        panic!("unsupported field type");
    }
}

/// if you need to make a new type rotatable add it to [ROTATABLE_TYPES]
fn build_rotate(enum_name: &Ident, item_enum: &ItemEnum) -> Vec<TokenStream2> {
    item_enum
        .variants
        .iter()
        .map(|variant| {
            let name = &enum_name;
            let vname = &variant.ident;
            match &variant.fields {
                Fields::Named(fields_named) => {
                    let rot_fields: Vec<_> = fields_named
                        .named
                        .iter()
                        .filter_map(|f| {
                            if let syn::Type::Path(type_path) = &f.ty {
                                let ident = &type_path.path.segments.last().unwrap().ident;
                                if ROTATABLE_TYPES.contains(&ident.to_string().as_str()) {
                                    return f.ident.clone();
                                }
                            }
                            None
                        })
                        .collect();
                    if rot_fields.is_empty() {
                        quote! { #name::#vname { .. } => {} }
                    } else {
                        let rotates = rot_fields.iter().map(|f| {
                            quote! { *#f = #f.rotate(new_direction); }
                        });
                        quote! {
                            #name::#vname { #(#rot_fields),*, .. } => {
                                #(#rotates)*
                            }
                        }
                    }
                }
                _ => {
                    quote! { #name::#vname => {} }
                }
            }
        })
        .collect()
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
