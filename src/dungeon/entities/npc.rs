use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use server::entity::entity::{EntityBase, EntityExtension};
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::serverbound::EntityInteractionType;
use server::Player;

pub struct InteractableNPC {
    pub default_yaw: f32,
    pub default_pitch: f32,
    pub interact_callback: fn(player: &mut Player<DungeonPlayer>),
}

impl EntityExtension<Dungeon> for InteractableNPC {

    fn tick(&mut self, entity: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
        if entity.ticks_existed.is_multiple_of(5) {
            return;
        }

        let player: Option<&Player<DungeonPlayer>> = entity
            .world()
            .players()
            .filter(|p| entity.position.distance(p.position) <= 5.0)
            .min_by(|a, b| {
                let dist_a = entity.position.distance(a.position);
                let dist_b = entity.position.distance(b.position);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some(player) = player {
            let direction = player.position - entity.position;
            let horizontal_dist = (direction.x.powi(2) + direction.z.powi(2)).sqrt();
            
            entity.yaw = (direction.z.atan2(direction.x).to_degrees() - 90.0) as f32;
            entity.pitch = (-direction.y.atan2(horizontal_dist).to_degrees()) as f32;
        } else {
            entity.yaw = self.default_yaw;
            entity.pitch = self.default_pitch;
        }
    }

    fn interact(
        &mut self,
        _: &mut EntityBase<Dungeon>,
        player: &mut Player<DungeonPlayer>,
        _: EntityInteractionType
    ) {
        (self.interact_callback)(player)
    }
}
