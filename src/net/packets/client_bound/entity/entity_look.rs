use crate::build_packet;
use crate::net::packets::packet::ClientBoundPacketImpl;
use crate::net::var_int::VarInt;
use crate::server::entity::entity::Entity;
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

#[derive(Debug, Clone, Copy)]
pub struct EntityLook {
    entity_id: i32,
    yaw: i8,
    pitch: i8,
    on_ground: bool,
}

impl EntityLook {
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            entity_id: entity.id,
            yaw: ((entity.last_yaw - entity.yaw) * 256.0 / 360.0) as i8,
            pitch: ((entity.last_pitch - entity.pitch) * 256.0 / 360.0) as i8,
            on_ground: entity.on_ground,
        }
    }
}

#[async_trait::async_trait]
impl ClientBoundPacketImpl for EntityLook {
    async fn write_to<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        let buf = build_packet!(
            0x16,
            VarInt(self.entity_id),
            self.yaw,
            self.pitch,
            self.on_ground
        );
        writer.write_all(&buf).await
    }
}