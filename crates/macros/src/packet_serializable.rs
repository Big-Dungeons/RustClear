use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, token, Expr, Generics, Token, Type, Visibility};

struct Field {
    vis: Visibility,
    name: Ident,
    colon_token: Token![:],
    ty: Type,
    transform: Option<(Token![=>], Expr)>,
    comma_token: Option<Token![,]>,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        let transform = if input.peek(Token![=>]) {
            let arrow: Token![=>] = input.parse()?;
            let expr: Expr = input.parse()?;
            Some((arrow, expr))
        } else {
            None
        };

        let comma_token = input.parse().ok();

        Ok(Self {
            vis,
            name,
            colon_token,
            ty,
            transform,
            comma_token,
        })
    }
}

struct PacketStruct {
    vis: Visibility,
    struct_token: Token![struct],
    name: Ident,
    generics: Generics,
    brace_token: token::Brace,
    fields: Vec<Field>,
}

impl Parse for PacketStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let struct_token: Token![struct] = input.parse()?;
        let name: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        let brace_token = syn::braced!(content in input);

        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
        }

        Ok(Self {
            vis,
            struct_token,
            name,
            generics,
            brace_token,
            fields,
        })
    }
}

pub fn packet_serializable_macro(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as PacketStruct);
    let PacketStruct {
        vis,
        name,
        generics,
        fields,
        ..
    } = parsed;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_fields = fields.iter().map(|f| {
        let vis = &f.vis;
        let name = &f.name;
        let colon = &f.colon_token;
        let ty = &f.ty;
        quote! {
            #vis #name #colon #ty,
        }
    });

    let write_fields = fields.iter().map(|f| {
        let write_expr = if let Some((_, expr)) = &f.transform {
            quote! { #expr }
        } else {
            let name = &f.name;
            quote! { &self.#name }
        };
        quote! {
            PacketSerializable::write(#write_expr, buf);
        }
    });

    quote! {
        #vis struct #name #generics {
            #(#struct_fields)*
        }

        impl #impl_generics crate::net::packets::packet_serialize::PacketSerializable for #name #ty_generics #where_clause {
            fn write(&self, buf: &mut Vec<u8>) {
                #(#write_fields)*
            }
        }
    }.into()
}