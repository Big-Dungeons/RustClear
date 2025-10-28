use bytes::Buf;

pub mod bitset;
pub mod hasher;

// not sure where to put this
pub fn get_vec(buf: &mut impl Buf, take: usize) -> Vec<u8> {
    let len = take.min(buf.remaining());
    let mut data = vec![0u8; len];
    buf.copy_to_slice(&mut data);
    data
}