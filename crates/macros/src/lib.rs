mod packet_serializable;
mod packet_deserializable;
mod entity_metadata;
mod blocks;
mod block_metadata;

use crate::block_metadata::block_metadata;
use crate::packet_deserializable::packet_deserializable_macro;
use crate::packet_serializable::packet_serializable_macro;
use crate::{blocks::blocks_macro, entity_metadata::entity_metadata_serializable_macro};
use proc_macro::TokenStream;

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

#[proc_macro_derive(BlockMetadata)]
pub fn derive_block_metadata(input: TokenStream) -> TokenStream {
    block_metadata(input)
}
