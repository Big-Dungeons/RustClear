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

/// used inside of Blocks to represent only 2 bits, implementing BlockMetadata
/// 
/// Keep in the actual struct is the equivalent to an u8,
/// it is only used for implementing BlockMetadata
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct u2(pub u8);

impl From<u8> for u2 {
    fn from(value: u8) -> Self {
        u2(value & 0b11)
    }
}

impl BlockMetadata for u2 {
    fn meta_size() -> u8 {
        2
    }
    fn get_meta(&self) -> u8 {
        self.0 & 0b11
    }
    fn from_meta(meta: u8) -> Self {
        (meta & 0b11).into()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct u3(pub u8);

impl From<u8> for u3 {
    fn from(value: u8) -> Self {
        u3(value & 0b111)
    }
}

impl BlockMetadata for u3 {
    fn meta_size() -> u8 {
        3
    }
    fn get_meta(&self) -> u8 {
        self.0 & 0b111
    }
    fn from_meta(meta: u8) -> Self {
        (meta & 0b111).into()
    }
}

