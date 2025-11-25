use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, token, Ident, LitFloat, Token, Type};

pub struct BlockVariants {
    idents: Vec<Ident>,
}

pub struct BlockField {
    ident: Ident,
    ty: Type,
}

pub struct BlockEntry {
    block_toughness: LitFloat,
    // only valid options: Pickaxe, Axe and Shovel
    tool: Option<Ident>,

    ident: Ident,
    variants: Option<BlockVariants>,
    fields: Vec<BlockField>,
}

impl Parse for BlockEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        // block toughness must be present for every block entry
        let block_toughness: LitFloat;
        let tool: Option<Ident>;

        {
            let _: Token![#] = input.parse()?;
            let bracketed;
            syn::bracketed!(bracketed in input);

            let block_toughness_ident: Ident = bracketed.parse()?;
            let _: Token![=] = bracketed.parse()?;

            if block_toughness_ident != "block_toughness" {
                return Err(bracketed.error("expected block_toughness"));
            }

            block_toughness = bracketed.parse()?;

            if !bracketed.is_empty() {
                let _: Token![,] = bracketed.parse()?;
                let ident: Ident = bracketed.parse()?;
                let _: Token![=] = bracketed.parse()?;

                if ident != "tool" {
                    return Err(bracketed.error("expected tool"));
                }

                let tool_ident: Ident = bracketed.parse()?;
                if tool_ident != "Pickaxe" && tool_ident != "Axe" && tool_ident != "Shovel" {
                    return Err(bracketed.error("invalid tool, only tools are: Pickaxe, Axe and Shovel."));
                }
                tool = Some(tool_ident)
            } else {
                tool = None;
            }
        }

        let ident: Ident = input.parse()?;

        let mut variants: Option<BlockVariants> = None;
        let mut fields: Vec<BlockField> = Vec::new();

        if input.peek(token::Brace) {

            let braced;
            syn::braced!(braced in input);

            while !braced.is_empty() {
                
                let ident: Ident = braced.parse()?;
                let _: Token![:] = braced.parse()?;

                if ident == "variants" {
                    if variants.is_some() {
                        return Err(braced.error("you can only have 1 variant per block entry"));
                    }

                    let variants_braced;
                    syn::braced!(variants_braced in braced);

                    let punctuated = Punctuated::<Ident, Token![,]>::parse_terminated(&variants_braced)?;

                    variants = Some(BlockVariants {
                        idents: punctuated.into_iter().collect(),
                    })
                } else {
                    let ty: Type = braced.parse()?;
                    fields.push(BlockField { ident, ty });
                }

                if braced.is_empty() {
                    break;
                }

                let _: Token![,] = braced.parse()?;
            }
        }

        Ok(Self {
            block_toughness,
            tool,
            ident,
            variants,
            fields,
        })
    }
}

pub struct BlockEntries {
    entries: Vec<BlockEntry>
}

impl Parse for BlockEntries {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let puntuacted: Punctuated::<BlockEntry, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(Self {
            entries: puntuacted.into_iter().collect()
        })
    }
}

pub fn blocks_macro(input: TokenStream) -> TokenStream {
    let BlockEntries { entries } = parse_macro_input!(input as BlockEntries);
    
    let mut enum_variants: proc_macro2::TokenStream = Default::default();
    let mut block_toughness: proc_macro2::TokenStream = Default::default();
    
    let mut serialization: proc_macro2::TokenStream = Default::default();
    let mut deserialization: proc_macro2::TokenStream = Default::default();
    
    for (block_id, entry) in entries.iter().enumerate() {
    
        let block_id = block_id as u16;
        let entry_ident = &entry.ident;
        let toughness = &entry.block_toughness;
        
        
        // { field1: Type, field2: Type }
        let mut fields_named: proc_macro2::TokenStream = Default::default();
        // { field1, field2 }
        let mut fields_struct_pattern: proc_macro2::TokenStream = Default::default();
        
        let mut entry_serialization: proc_macro2::TokenStream = Default::default();
        let mut entry_deserialization: proc_macro2::TokenStream = Default::default();
        
        for block_field in entry.fields.iter() {
            let ident = &block_field.ident;
            let ty = &block_field.ty;
            
            fields_named.extend(quote! {
               #ident: #ty,
            });
            fields_struct_pattern.extend(quote! { 
                #ident, 
            });
            
            entry_serialization.extend(quote! {
                meta |= #ident.get_meta() << offset;
                offset += <#ty>::meta_size();
            });
            entry_deserialization.extend(quote! {
                let #ident = <#ty>::from_meta(((meta >> offset) & ((1 << <#ty>::meta_size()) - 1)) as u8);
                offset += <#ty>::meta_size();
            });
        }
        
        if !entry.fields.is_empty() {
            fields_named = quote! { { #fields_named } };
            fields_struct_pattern = quote! { { #fields_struct_pattern } };
        }
        
        if let Some(block_variants) = &entry.variants {
            
            let variant_min_size = (64 - (block_variants.idents.len() - 1).leading_zeros()) as u8;
            let mut variant_deserialization: proc_macro2::TokenStream = Default::default();
            
            for (index, v_ident) in block_variants.idents.iter().enumerate() {
                let variant_index = index as u8;
            
                enum_variants.extend(quote! {
                    #v_ident #fields_named,
                });
                    
                serialization.extend(quote! {
                    Self::#v_ident #fields_struct_pattern => {
                        let mut meta: u8 = #variant_index;
                        let mut offset: u8 = #variant_min_size;
                        #entry_serialization
                        (#block_id << 4) | (meta as u16)
                    },
                });
                
                variant_deserialization.extend(quote! {
                    #variant_index => Self::#v_ident #fields_struct_pattern,
                });
                
                block_toughness.extend(quote! {
                    Self::#v_ident #fields_struct_pattern => #toughness,
                });
            }
            
            deserialization.extend(quote! {
                #block_id => {
                    let meta = (value & 0x0F) as u8;
                    let mut offset: u8 = #variant_min_size;
                    #entry_deserialization
                    match meta & ((1 << #variant_min_size) - 1) as u8 {
                        #variant_deserialization
                        _ => unreachable!()
                    }
                }
            });
        } else {
            enum_variants.extend(quote! {
                #entry_ident #fields_named,
            });
            
            serialization.extend(quote! {
                Self::#entry_ident #fields_struct_pattern => {
                    let mut meta: u8 = 0;
                    let mut offset: u8 = 0;
                    #entry_serialization
                    (#block_id << 4) | (meta as u16)
                },
            });
            deserialization.extend(quote! {
                #block_id => {
                    let meta = (value & 0x0F) as u8;
                    let mut offset: u8 = 0;
                    #entry_deserialization
                    Self::#entry_ident #fields_struct_pattern
                }
            });
            
            block_toughness.extend(quote! {
                Self::#entry_ident { .. } => #toughness,
            });
        }
    }
    
    quote! {
        #[derive(Debug, Eq, PartialEq, Copy, Clone)]
        pub enum Block {
            #enum_variants
        }
        
        impl Block {
            
            pub fn get_blockstate_id(&self) -> u16 {
                match self {
                    #serialization
                }
            }
            
            pub const fn get_toughness(&self) -> f32 {
                match self {
                    #block_toughness
                }
            }
        }
        
        impl From<u16> for Block {
            fn from(value: u16) -> Self {
                let block_id = value >> 4;
                match block_id {
                    #deserialization
                    _ => Block::Air,
                }
            }
        }
    }.into()
}
