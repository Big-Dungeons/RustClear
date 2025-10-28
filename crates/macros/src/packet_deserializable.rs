use proc_macro::TokenStream;
use quote::quote;
use syn::Fields::Unit;
use syn::{parse_macro_input, Item};

pub fn packet_deserializable_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    match input {
        Item::Struct(ref item_struct) => {
            let name = &item_struct.ident;
            let fields = item_struct.fields.iter().map(|field| {
                let ident = &field.ident;
                quote! { #ident: crate::network::packets::packet_deserialize::PacketDeserializable::read(buffer)?, }
            });
            quote! {
                #input

                impl crate::network::packets::packet_deserialize::PacketDeserializable for #name {
                   fn read(buffer: &mut impl bytes::Buf) -> anyhow::Result<Self> {
                        Ok(Self {
                            #(#fields)*
                        })
                    }
                }
            }
        }
        Item::Enum(ref item_enum) => {
            let name = &item_enum.ident;
            let variants = item_enum.variants.iter().enumerate().map(|(index, variant)| {
                if let Unit = variant.fields {
                    let index = index as i8;
                    let indent = &variant.ident;
                    quote! {
                        #index => Ok(#name::#indent),
                    }
                } else {
                    return quote! {
                        compile_error!("packet_deserializable doesn't support enums with fields");
                    }.into()
                }
            });
            quote! {
                #input

                impl crate::network::packets::packet_deserialize::PacketDeserializable for #name {
                   fn read(buffer: &mut impl bytes::Buf) -> anyhow::Result<Self> {
                        let id: i8 = crate::network::packets::packet_deserialize::PacketDeserializable::read(buffer)?;
                        match id {
                            #(#variants)*
                            _ => Err(anyhow::anyhow!("Invalid id ({}) for enum {}", id, stringify!(#name)))
                        }
                    }
                }
            }
        }
        _ => {
            syn::Error::new_spanned(input, "packet_deserializable only supports structs and enums")
                .into_compile_error()
        }
    }.into()
}