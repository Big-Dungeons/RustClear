use crate::build_packet;
use crate::net::packets::packet::ClientBoundPacketImpl;
use crate::net::var_int::VarInt;
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

#[repr(i8)]
#[derive(Debug, Copy, Clone)]
pub enum Effects {
    Haste = 3,
    NightVision = 16,
}

#[derive(Clone, Debug)]
pub struct EntityEffect {
    pub entity_id: i32,
    pub effect: Effects,
    pub amplifier: i8,
    pub duration: i32,
    pub hide_particles: bool,
}

pub const HASTEID: i8 = 3;
pub const NIGHTVISIONID: i8 = 16;

#[async_trait::async_trait]
impl ClientBoundPacketImpl for EntityEffect {
    async fn write_to<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        let buf = build_packet!(
            0x1D,
            VarInt(self.entity_id),
            self.effect as i8,
            self.amplifier,
            VarInt(self.duration),
            self.hide_particles as u8
        );
        writer.write_all(&buf).await
    }
}