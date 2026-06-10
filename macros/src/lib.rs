use crate::block_field_metadata::block_field_metadata;
use crate::blocks::blocks_macro;
use crate::packets::{identified_packet_macro, packet_deserializable_macro, packet_serializable_macro};
use proc_macro::TokenStream;

mod blocks;
mod block_field_metadata;
mod packets;

#[proc_macro_derive(PacketSerializable)]
pub fn derive_packet_serializable(item: TokenStream) -> TokenStream {
    packet_serializable_macro(item)
}

#[proc_macro_attribute]
pub fn identified_packet(attr: TokenStream, item: TokenStream) -> TokenStream {
    identified_packet_macro(attr, item)
}

#[proc_macro_derive(PacketDeserializable)]
pub fn derive_packet_deserializable(input: TokenStream) -> TokenStream {
    packet_deserializable_macro(input)
}

#[proc_macro]
pub fn blocks(input: TokenStream) -> TokenStream {
    blocks_macro(input)
}

#[proc_macro_derive(BlockFieldMetadata)]
pub fn derive_block_field_metadata(input: TokenStream) -> TokenStream {
    block_field_metadata(input)
}