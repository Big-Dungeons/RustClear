use crate::build_packet;
use crate::net::packets::packet::ClientBoundPacketImpl;
use crate::net::var_int::VarInt;
use crate::server::entity::entity::Entity;
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

#[derive(Debug, Clone, Copy)]
pub struct EntityRelMove {
    entity_id: i32,
    pos_x: i8,
    pos_y: i8,
    pos_z: i8,
    on_ground: bool,
}

impl EntityRelMove {
    pub fn from_entity(entity: &Entity) -> Self {
        println!("pos x {}", entity.last_position.x - entity.position.x);
        println!("pos y {}", entity.last_position.y - entity.position.y);
        println!("pos z {}", entity.last_position.z - entity.position.z);
        Self {
            entity_id: entity.id,
            pos_x: ((entity.last_position.x - entity.position.x) * 32.0) as i8,
            pos_y: ((entity.last_position.y - entity.position.y) * 32.0) as i8,
            pos_z: ((entity.last_position.z - entity.position.z) * 32.0) as i8,
            on_ground: entity.on_ground,
        }
    }
}

#[async_trait::async_trait]
impl ClientBoundPacketImpl for EntityRelMove {
    async fn write_to<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        let buf = build_packet!(
            0x15,
            VarInt(self.entity_id),
            self.pos_x,
            self.pos_y,
            self.pos_z,
            self.on_ground
        );
        writer.write_all(&buf).await
    }
}