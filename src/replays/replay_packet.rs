use std::{collections::HashMap, time::Duration};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use fstr::FString;
use uuid::Uuid;

use crate::{network::{binary::var_int::{read_var_int, var_int_size, VarInt}, packets::{packet_deserialize::PacketDeserializable, packet_serialize::PacketSerializable}}, player::player::GameProfile};


pub(super) struct ReplayPacket {
    pub since_start: Duration,
    pub game_profile: GameProfile,
    pub packet: Bytes,
}

impl ReplayPacket {
    // im not sure i like the custom serialization handling ive done here for these. it should probably reuse existing networking stuff, but i cant be bothered rn.
    // it could also not do an allocation and write directly into the file buffer...
    pub fn serialize(&self) -> Bytes {
        let data_size = self.data_size();
        let full_size = var_int_size(data_size as i32) + data_size;
        
        let mut buffer = BytesMut::with_capacity(full_size);
        let buf = &mut buffer;
        
        VarInt(data_size as i32).write(buf);
        
        self.since_start.as_secs().write(buf);
        self.since_start.subsec_nanos().write(buf);
        self.game_profile.uuid.write(buf);
        self.game_profile.username.write(buf);
        VarInt(self.packet.len() as i32).write(buf);
        buffer.put_slice(&self.packet);
        
        buffer.freeze()
    }
    
    pub fn deserialize(buffer: &mut impl Buf) -> Option<Self> {
        let secs = buffer.get_u64();
        let nanos = buffer.get_u32();
        let since_start = Duration::new(secs, nanos);
        
        let uuid = Uuid::from_u128(buffer.get_u128());
        
        let username = FString::read(buffer).ok()?;
        let game_profile = GameProfile {
            uuid,
            username,
            properties: HashMap::new(),
        };
        
        let data_len = read_var_int(buffer)? as usize;
        let data = buffer.copy_to_bytes(data_len);
        
        Some(Self {
            since_start,
            game_profile,
            packet: data
        })
    }
    
    fn data_size(&self) -> usize {
        size_of::<Duration>() + 
        size_of::<Uuid>() +
        self.game_profile.username.write_size() +
        var_int_size(self.packet.len() as i32) + 
        self.packet.len()
    }
}