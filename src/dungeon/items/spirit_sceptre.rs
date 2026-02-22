use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use bevy_ecs::prelude::Component;
use server::block::block_collision::check_block_collisions;
use server::constants::{EntityVariant, Sound};
use server::entity::components::{EntityBehaviour, MobAppearance};
use server::entity::entity::MinecraftEntity;
use server::entity::entity_metadata::{BatMetadata, EntityMetadata};
use server::inventory::item_stack::ItemStack;
use server::player::packet_processing::BlockInteractResult;
use server::types::aabb::AABB;
use server::{ClientId, Player};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SpiritSceptre;

impl DungeonItem for SpiritSceptre {
    fn on_interact(&self, player: &mut Player<DungeonPlayer>, block: Option<BlockInteractResult>) {
        if block.is_some() {
            player.sync_inventory();
        }

        player.play_sound(Sound::GhastFireball, 0.3, 1.0);

        player.world_mut().spawn_entity(
            player.player_eye_position(),
            player.yaw,
            player.pitch,
            MobAppearance {
                variant: EntityVariant::Bat,
                metadata: EntityMetadata::Bat(BatMetadata { hanging: false, })
            },
            SceptreBatBehaviour { player_id: player.client_id }
        );
    }

    fn item_stack(&self) -> ItemStack {
        ItemStack {
            item: 38,
            stack_size: 1,
            metadata: 2,
            tag_compound: None,
        }
    }
}

#[derive(Component)]
pub struct SceptreBatBehaviour {
    player_id: ClientId
}

impl EntityBehaviour<Dungeon> for SceptreBatBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        let world = entity.world_mut();
        let Some(player_index) = world.player_map.get(component.player_id) else {
            entity.destroy();
            return;
        };

        let player_rc = &mut world.players[*player_index];
        let player = unsafe { &mut *player_rc.get() };

        // there is a random offset occasionally, so maybe add that
        entity.position += player.rotation_vec().as_dvec3();
        entity.yaw = player.yaw;
        entity.pitch = player.pitch;

        let aabb = AABB::from_width_height(0.9, 0.5).offset(entity.position);
        if check_block_collisions(world, &aabb) {
            // query nearby entities with dungeon mob and kill them
            world.play_sound_at(Sound::RandomExplode, 1.0, 0.9, entity.position);
            entity.destroy()
        }
    }
}