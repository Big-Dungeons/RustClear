use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Attribute, Expr, Token, Type, Visibility};

struct MetadataField {
    metadata_id: usize,
    visibility: Visibility,
    ident: Ident,
    ty: Type,
    default_expr: Option<Expr>,
}

impl Parse for MetadataField {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let id_lit: syn::LitInt = input.parse()?;
        let metadata_id = id_lit.base10_parse::<usize>()?;

        let _: Token![=>] = input.parse()?;

        let visibility: Visibility = input.parse()?;

        let ident: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        let default_expr = if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            visibility,
            metadata_id,
            ident,
            ty,
            default_expr,
        })
    }
}

impl ToTokens for MetadataField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.visibility;
        let ident = &self.ident;
        let ty = &self.ty;

        tokens.extend(quote! {
            #vis #ident: #ty
        });
    }
}

struct EnumVariant {
    pub ident: Ident,
    pub fields: Vec<MetadataField>,
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;

        let braced;
        syn::braced!(braced in input);
        let punctuated: Punctuated<MetadataField, Token![,]> =
            Punctuated::parse_terminated(&braced)?;
        Ok(Self {
            ident,
            fields: punctuated.into_iter().collect(),
        })
    }
}

enum MetadataInput {
    Struct {
        attrs: Vec<Attribute>,
        visibility: Visibility,
        ident: Ident,
        fields: Vec<MetadataField>,
    },
    Enum {
        attrs: Vec<Attribute>,
        visibility: Visibility,
        ident: Ident,
        variants: Vec<EnumVariant>,
    },
}

impl Parse for MetadataInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let visibility: Visibility = input.parse()?;

        if input.peek(Token![struct]) {
            let _: Token![struct] = input.parse()?;
            let ident = input.parse()?;

            let braced;
            syn::braced!(braced in input);
            let punctuated: Punctuated<MetadataField, Token![,]> =
                Punctuated::parse_terminated(&braced)?;

            Ok(Self::Struct {
                attrs,
                visibility,
                ident,
                fields: punctuated.into_iter().collect(),
            })
        } else {
            let _: Token![enum] = input.parse()?;
            let ident = input.parse()?;

            let braced;
            syn::braced!(braced in input);
            let punctuated: Punctuated<EnumVariant, Token![,]> =
                Punctuated::parse_terminated(&braced)?;

            Ok(Self::Enum {
                attrs,
                visibility,
                ident,
                variants: punctuated.into_iter().collect(),
            })
        }
    }
}

pub fn entity_metadata_serializable_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MetadataInput);
    match input {
        MetadataInput::Struct { attrs, visibility, ident, fields } => {

            let mut write_size_stream: Vec<TokenStream2> = Vec::new();
            let mut write_stream: TokenStream2 = TokenStream2::new();

            // if this doesn't match length but != 0,
            // that means only some implement a default expr
            let mut default_expr_counter = 0;
            let mut default_stream: TokenStream2 = TokenStream2::new();

            for field in fields.iter() {
                let ident = &field.ident;
                let ty = &field.ty;
                let id = field.metadata_id as u8;

                write_size_stream.push(quote! {
                    1 + self.#ident.write_size()
                });
                write_stream.extend(quote! {
                    u8::write(&(<#ty as MetadataSerializable>::ID << 5 | #id & 31), buf);
                    <#ty as PacketSerializable>::write(&self.#ident, buf);
                });

                if let Some(expr) = &field.default_expr {
                    default_expr_counter += 1;
                    default_stream.extend(quote! { #ident: #expr, })
                }
            }

            let default_impl = if default_expr_counter == fields.len() {
                quote! {
                    impl Default for #ident {
                        fn default() -> Self {
                            Self {
                                #default_stream
                            }
                        }
                    }
                }
            } else if default_expr_counter != 0 {
                quote! {
                    compile_error!("This implements a default value for only some values. It must be implemented for all");
                }
            } else {
                quote! {}
            };

            quote! {
                #(#attrs)*
                #visibility struct #ident {
                    #(#fields),*
                }

                impl PacketSerializable for #ident {
                    fn write_size(&self) -> usize {
                        1 + #(#write_size_stream)+*
                    }
                    fn write(&self, buf: &mut bytes::BytesMut) {
                        #write_stream
                        u8::write(&127, buf);
                    }
                }

                #default_impl
            }
        }
        MetadataInput::Enum { attrs, visibility, ident, variants } => {

            let mut field_stream: TokenStream2 = TokenStream2::new();
            let mut metadata_impls: Vec<TokenStream2> = Vec::new();

            let mut write_size_stream: Vec<TokenStream2> = Vec::new();
            let mut write_stream: TokenStream2 = TokenStream2::new();

            for variant in variants.iter() {
                let ident = &variant.ident;
                let fields = &variant.fields;
                let metadata_ident = format_ident!("{}{}", ident, "Metadata");

                // if this doesn't match length but != 0,
                // that means only some implement a default expr
                let mut default_expr_counter = 0;
                let mut default_stream: TokenStream2 = TokenStream2::new();

                field_stream.extend(quote! {
                    #ident(#metadata_ident),
                });

                let mut variant_write_sizes: Vec<TokenStream2> = Vec::new();
                let mut variant_writes: TokenStream2 = TokenStream2::new();

                for field in fields {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    let id = field.metadata_id as u8;

                    variant_write_sizes.push(quote! {
                        1 + metadata.#ident.write_size()
                    });
                    variant_writes.extend(quote! {
                        u8::write(&(<#ty as MetadataSerializable>::ID << 5 | #id & 31), buf);
                        <#ty as PacketSerializable>::write(&metadata.#ident, buf);
                    });

                    if let Some(expr) = &field.default_expr {
                        default_expr_counter += 1;
                        default_stream.extend(quote! { #ident: #expr, })
                    }
                }

                write_size_stream.push(quote! {
                    Self::#ident(metadata) => {
                        #(#variant_write_sizes)+*
                    }
                });
                write_stream.extend(quote! {
                    Self::#ident(metadata) => {
                        #variant_writes
                    }
                });

                let default_impl = if default_expr_counter == fields.len() {
                    quote! {
                        impl Default for #metadata_ident {
                            fn default() -> Self {
                                Self {
                                    #default_stream
                                }
                            }
                        }
                    }
                } else if default_expr_counter != 0 {
                    quote! {
                        compile_error!("This implements a default value for only some values. It must be implemented for all");
                    }
                } else {
                    quote! {}
                };

                metadata_impls.push(quote! {
                    #(#attrs)*
                    #visibility struct #metadata_ident {
                        #(#fields),*
                    }

                    #default_impl
                })
            }

            quote! {
                #(#attrs)*
                #visibility enum #ident {
                    #field_stream
                }

                impl PacketSerializable for #ident {
                    fn write_size(&self) -> usize {
                        1 + match self {
                            #(#write_size_stream)*
                        }
                    }
                    fn write(&self, buf: &mut bytes::BytesMut) {
                        match self {
                            #write_stream
                        }
                        u8::write(&127, buf);
                    }
                }

                #(#metadata_impls)*
            }
        }
    }.into()
}
