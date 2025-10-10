use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;
use glam::Vec3;

impl PacketSerializable for Vec3 {
    fn write_size(&self) -> usize {
        const { size_of::<f32>() * 3 }
    }
    fn write(&self, buf: &mut BytesMut) {
        self.x.write(buf);
        self.y.write(buf);
        self.z.write(buf);
    }
}