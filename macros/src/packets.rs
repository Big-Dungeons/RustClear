use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Field, Fields, Item, ItemEnum, ItemStruct, LitInt, Token, Variant};

pub struct PacketAttr {
    pub packet_id: LitInt
}

impl Parse for PacketAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "id" {
            return Err(syn::Error::new(ident.span(), "expected `id`"));
        }
        let _: Token![=] = input.parse()?;
        let packet_id: LitInt = input.parse()?;
        Ok(PacketAttr { packet_id })
    }
}

pub fn identified_packet_macro(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let PacketAttr { packet_id } = parse_macro_input!(attr as PacketAttr);
    let input = parse_macro_input!(input as Item);

    let (ident, generics) = match &input {
        Item::Struct(ItemStruct { ident, generics, .. }) => (ident, generics),
        Item::Enum(ItemEnum { ident, generics, .. }) => (ident, generics),
        _ => return syn::Error::new_spanned(input, "identified_packet only supports structs and enums").into_compile_error().into()
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #input

        impl #impl_generics IdentifiedPacket for #ident #ty_generics #where_clause {
            const PACKET_ID: i32 = #packet_id;
        }
    }.into()
}

pub fn packet_serializable_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as ItemStruct);
    let ItemStruct { ident, generics, fields, .. } = parsed;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut write_size = TokenStream::new();
    let mut write = TokenStream::new();
    for Field { ident, .. } in &fields {
        if !write_size.is_empty() {
            write_size.extend(quote! { + });
        }
        write_size.extend(quote! { self.#ident.write_size() });
        write.extend(quote! { PacketSerializable::write(&self.#ident, buf); })
    }

    quote! {
        impl #impl_generics PacketSerializable for #ident #ty_generics #where_clause {
            fn write_size(&self) -> usize {
                #write_size
            }
            fn write(&self, buf: &mut BytesMut) {
                #write
            }
        }

    }.into()
}

pub fn packet_deserializable_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);
    match input {
        Item::Struct(ItemStruct { ref ident, ref fields, ref generics, .. }) => {
            let mut read = TokenStream::new();
            for Field { ident, .. } in fields {
                read.extend(quote! { #ident: PacketDeserializable::read(buffer)?, })
            }
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            quote! {
                impl #impl_generics PacketDeserializable for #ident #ty_generics #where_clause {
                    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
                        Ok(Self { #read })
                    }
                }
            }
        },
        Item::Enum(ItemEnum { ref ident, ref variants, .. }) => {
            let mut read = TokenStream::new();
            for (index, Variant { ident, fields, .. }) in variants.iter().enumerate() {
                if !matches!(fields, Fields::Unit) {
                    return syn::Error::new_spanned(input, "enum variants with fields aren't supported").into_compile_error().into()
                }
                let index = index as i8;
                read.extend(quote! { #index => Ok(Self::#ident), })
            }
            quote! {
                impl PacketDeserializable for #ident {
                    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
                        let id: i8 = PacketDeserializable::read(buffer)?;
                        match id {
                            #read
                            _ => Err(anyhow::anyhow!("Invalid id ({}) for enum {}", id, stringify!(#ident)))
                        }
                    }
                }
            }
        }
        _ => syn::Error::new_spanned(input, "PacketDeserializable only supports structs and enums").into_compile_error()
    }.into()
}