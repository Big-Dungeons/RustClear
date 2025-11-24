#![allow(non_camel_case_types)]


/// Used in Blocks enum, to encode and decode fields
/// 
/// For enums it is possible to use derive instead of writing this out.
pub trait BlockMetadata {
    /// returns size of the metadata in bits
    fn meta_size() -> u8;
    
    /// returns the actual metadata
    fn get_meta(&self) -> u8;
    
    /// returns Self from metadata provided
    fn from_meta(meta: u8) -> Self;

}

impl BlockMetadata for u8 {
    fn meta_size() -> u8 {
        4
    }
    fn get_meta(&self) -> u8 {
        self & 0x0F
    }
    fn from_meta(meta: u8) -> Self {
        meta & 0x0F
    }
}

impl BlockMetadata for bool {
    fn meta_size() -> u8 {
        1
    }
    fn get_meta(&self) -> u8 {
        *self as u8
    }
    fn from_meta(meta: u8) -> Self {
        (meta & 0b1) != 0
    }
}