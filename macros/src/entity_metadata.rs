use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Attribute, Expr, LitInt, Token, Type, Visibility};

struct MetadataField {
    metadata_index: usize,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    default_expr: Expr,
}

impl Parse for MetadataField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id_lit: LitInt = input.parse()?;
        let metadata_index = id_lit.base10_parse::<usize>()?;
        let _: Token![=>] = input.parse()?;

        let vis = input.parse()?;
        let ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;

        let _: Token![=] = input.parse()?;
        let default_expr = input.parse()?;

        Ok(Self {
            metadata_index,
            vis,
            ident,
            ty,
            default_expr,
        })
    }
}

impl ToTokens for MetadataField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let MetadataField { vis, ident, ty, .. } = self;
        tokens.extend(quote! {
            #vis #ident: #ty
        })
    }
}

struct MetadataVariant {
    ident: Ident,
    fields: Vec<MetadataField>,
}

impl Parse for MetadataVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;

        let braced;
        syn::braced!(braced in input);
        let punctuated: Punctuated<MetadataField, Token![,]> = Punctuated::parse_terminated(&braced)?;
        let fields = punctuated.into_iter().collect();

        Ok(Self {
            ident,
            fields
        })
    }
}

struct MetadataInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    variants: Vec<MetadataVariant>
}

impl Parse for MetadataInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;

        let _: Token![enum] = input.parse()?;
        let ident = input.parse()?;

        let braced;
        syn::braced!(braced in input);
        let punctuated: Punctuated<MetadataVariant, Token![,]> = Punctuated::parse_terminated(&braced)?;
        let variants = punctuated.into_iter().collect();

        Ok(Self {
            attrs,
            vis,
            ident,
            variants,
        })
    }
}

pub fn entity_metadata_serializable_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MetadataInput { attrs, vis, ident, variants } = parse_macro_input!(input as MetadataInput);

    let mut field_stream: TokenStream = Default::default();
    let mut metadata_structs: TokenStream = Default::default();

    // for packet serializable of main enum
    let mut write_size: TokenStream = Default::default();
    let mut write: TokenStream = Default::default();

    for MetadataVariant { ident: variant_ident, fields } in variants {
        let metadata_ident = format_ident!("{}Metadata", variant_ident);

        field_stream.extend(quote! {
            #variant_ident(#metadata_ident),
        });

        let mut variant_write_size: TokenStream = Default::default();
        let mut variant_write: TokenStream = Default::default();
        let mut defaults: TokenStream = Default::default();

        for MetadataField { metadata_index, ident, ty, default_expr, .. } in &fields {
            variant_write_size.extend(quote! {
                + 1 + self.#ident.write_size()
            });

            let index = *metadata_index as u8;
            variant_write.extend(quote! {
                u8::write(&(<#ty as EntityMetadataSerializable>::ID << 5 | #index & 31), buf);
                <#ty as PacketSerializable>::write(&self.#ident, buf);
            });
            defaults.extend(quote! { #ident: #default_expr, })
        }

        metadata_structs.extend(quote! {
            #(#attrs)*
            #vis struct #metadata_ident {
                #(#fields),*
            }

            impl Default for #metadata_ident {
                fn default() -> Self {
                    Self {
                        #defaults
                    }
                }
            }

            impl PacketSerializable for #metadata_ident {
                fn write_size(&self) -> usize {
                    1 #variant_write_size
                }
                fn write(&self, buf: &mut bytes::BytesMut) {
                    #variant_write
                    u8::write(&127, buf);
                }
            }

            impl From<#metadata_ident> for #ident {
                fn from(value: #metadata_ident) -> Self {
                    Self::#variant_ident(value)
                }
            }
        });

        write_size.extend(quote! {
            Self::#variant_ident(metadata) => metadata.write_size(),
        });
        write.extend(quote! {
            Self::#variant_ident(metadata) => {
                metadata.write(buf);
            }
        });
    }

    quote! {
        #(#attrs)*
        #vis enum #ident {
            #field_stream
        }

        impl PacketSerializable for #ident {
            fn write_size(&self) -> usize {
                match self {
                    #write_size
                }
            }
            fn write(&self, buf: &mut bytes::BytesMut) {
                match self {
                    #write
                }
            }
        }

        #metadata_structs
    }.into()
}