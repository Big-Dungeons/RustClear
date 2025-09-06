use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_deref_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (member, ty) = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                if fields.named.len() != 1 {
                    panic!("you can only derive deref with structs with 1 field");
                }
                let field = fields.named.first().unwrap();
                (syn::Member::Named(field.ident.clone().unwrap()), field.ty.clone())
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    panic!("you can only derive deref with structs with 1 field");
                }
                let field = fields.unnamed.first().unwrap();
                (syn::Member::Unnamed(syn::Index::from(0)), field.ty.clone())
            }
            Fields::Unit => {
                panic!("derive deref only works on structs with 1 field.");
            }
        },
        _ => panic!("derive deref only works on structs with 1 field."),
    };

    quote! {
        impl #impl_generics ::std::ops::Deref for #name #ty_generics #where_clause {
            type Target = #ty;
            fn deref(&self) -> &Self::Target {
                &self.#member
            }
        }
    }.into()
}

pub fn derive_deref_mut_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let member = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                if fields.named.len() != 1 {
                    panic!("you can only derive deref with structs with 1 field");
                }
                let field = fields.named.first().unwrap();
                syn::Member::Named(field.ident.clone().unwrap())
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    panic!("you can only derive deref with structs with 1 field");
                }
                syn::Member::Unnamed(syn::Index::from(0))
            }
            Fields::Unit => {
                panic!("derive deref only works on structs with 1 field.");
            }
        },
        _ => panic!("derive deref only works on structs with 1 field."),
    };

    quote! {
        impl #impl_generics ::std::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#member
            }
        }
    }.into()
}