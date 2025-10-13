use std::time::Duration;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use uuid::Uuid;

use crate::record::profile_id::ProfileId;

#[derive(Debug)]
pub struct ReplayPacket {
    pub since_start: Duration,
    pub profile: ProfileId,
    pub packet: Bytes,
}

impl ReplayPacket {
    pub const LEN_SIZE: usize = size_of::<u32>();
    
    // im not sure i like the custom serialization handling ive done here for these. it should probably reuse existing networking stuff, but i cant be bothered rn.
    // it could also not do an allocation and write directly into the file buffer...
    pub fn serialize(&self) -> Bytes {
        let data_size = self.data_size();
        let full_size = Self::LEN_SIZE + data_size;
        
        let mut buffer = BytesMut::with_capacity(full_size);
        let buf = &mut buffer;
        
        buf.put_u32(data_size as u32);
        
        buf.put_u64(self.since_start.as_secs());
        buf.put_u32(self.since_start.subsec_nanos());
        buf.put_u128(self.profile.get_id().as_u128());
        
        buf.put_u32(self.packet.len() as u32);
        buffer.put_slice(&self.packet);
        
        buffer.freeze()
    }
    
    pub fn deserialize(buffer: &mut impl Buf) -> Self {
        let secs = buffer.get_u64();
        let nanos = buffer.get_u32();
        let since_start = Duration::new(secs, nanos);
        let profile = ProfileId::new(Uuid::from_u128(buffer.get_u128()));
        
        let data_len = buffer.get_u32() as usize;
        
        // we cant copy_to_bytes here since it will keep the data alive in the vec.
        // we need to ensure the bytesmut arc never increments so it can fix itself rather than allocate again.
        let mut packet_data = vec![0u8; data_len];
        buffer.copy_to_slice(&mut packet_data);
        let packet = Bytes::from(packet_data);
        
        Self {
            since_start,
            profile,
            packet,
        }
    }
    
    fn data_size(&self) -> usize {
        size_of::<u64>() + 
        size_of::<u32>() + 
        size_of::<u128>() +
        size_of::<u32>() + 
        self.packet.len()
    }
}