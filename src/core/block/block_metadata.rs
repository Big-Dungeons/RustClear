

pub trait BlockFieldMetadata {
    const META_SIZE: u8;

    fn get_meta(self) -> u8;
    fn from_meta(meta: u8) -> Self;
}

impl BlockFieldMetadata for u8 {
    const META_SIZE: u8 = 4;

    fn get_meta(self) -> u8 {
        self & 0x0F
    }
    fn from_meta(meta: u8) -> Self {
        meta & 0x0F
    }
}

impl BlockFieldMetadata for bool {
    const META_SIZE: u8 = 1;

    fn get_meta(self) -> u8 {
        self as u8
    }
    fn from_meta(meta: u8) -> Self {
        (meta & 0b1) != 0
    }
}