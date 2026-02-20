use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use bevy_ecs::component::Component;
use server::entity::components::EntityBehaviour;
use server::entity::entity::MinecraftEntity;
use server::Player;

/// Behaviour for NPCs that stand still and look at the nearest player.
#[derive(Component)]
pub struct NPCBehaviour {
    pub default_yaw: f32,
    pub default_pitch: f32,
}

impl EntityBehaviour<Dungeon> for NPCBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        if entity.ticks_existed.is_multiple_of(2) {
            return;
        }

        // todo, search only nearby chunks
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
            entity.yaw = component.default_yaw;
            entity.pitch = component.default_pitch;
        }
    }
}
