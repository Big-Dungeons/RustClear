use crate::server::block::block_interact_action::BlockInteractAction;
use crate::server::block::block_position::BlockPos;
use crate::server::block::blocks::Blocks;
use crate::server::entity::entity::NoEntityImpl;
use crate::server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use crate::server::items::item_stack::ItemStack;
use crate::server::player::player::Player;
use crate::server::utils::aabb::AABB;
use crate::server::utils::direction::Direction;
use crate::server::world::World;
use glam::DVec3;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub enum SecretType {
    WitherEssence {
        // have rotation here
    },
    Chest {
        direction: Direction
    },
    Item {
        item: ItemStack
    },
    // ill do this one later
    Bat,
}

pub struct DungeonSecret {
    secret_type: SecretType,
    block_pos: BlockPos, // might not even need?
    spawn_aabb: AABB,
    pub has_spawned: bool,
    pub obtained: bool,
}

// when this is integrated into rooms, remove this and just inline the spawning stuff
pub fn tick(dungeon_secret: &Rc<RefCell<DungeonSecret>>, player: &Player) {
    let mut secret = dungeon_secret.borrow_mut();
    if !secret.has_spawned && player.collision_aabb().intersects(&secret.spawn_aabb) {
        secret.has_spawned = true;
        DungeonSecret::spawn_into_world(dungeon_secret, secret, player.world_mut())
    }
}

impl DungeonSecret {

    pub fn new(secret_type: SecretType, position: BlockPos, spawn_radius: f64) -> Self {
        Self {
            secret_type,
            block_pos: position,
            spawn_aabb: {
                let (x, y, z) = (position.x as f64, position.y as f64, position.z as f64);
                AABB {
                    min: DVec3::new(x - spawn_radius, y - spawn_radius, z - spawn_radius),
                    max: DVec3::new(x + spawn_radius, y + spawn_radius, z + spawn_radius),
                }
            },
            has_spawned: false,
            obtained: false,
        }
    }
    
    fn spawn_into_world(
        secret_rc: &Rc<RefCell<DungeonSecret>>,
        secret: RefMut<DungeonSecret>,
        world: &mut World
    ) {
        match &secret.secret_type {
            SecretType::WitherEssence { .. } => {
                world.set_block_at(
                    Blocks::Skull { direction: Direction::Up, no_drop: false },
                    secret.block_pos.x,
                    secret.block_pos.y,
                    secret.block_pos.z
                );
                world.interactable_blocks.insert(secret.block_pos, BlockInteractAction::WitherEssence {
                    secret: secret_rc.clone()
                });
            }
            SecretType::Chest { direction } => {
                world.set_block_at(
                    Blocks::Chest { direction: direction.clone() },
                    secret.block_pos.x,
                    secret.block_pos.y,
                    secret.block_pos.z,
                );
                world.interactable_blocks.insert(secret.block_pos, BlockInteractAction::Chest {
                    secret: secret_rc.clone()
                });
            }
            SecretType::Item { item } => {
                world.spawn_entity(
                    secret.block_pos.as_dvec3_centered(),
                    EntityMetadata::new(EntityVariant::DroppedItem {
                        item: item.clone()
                    }),
                    NoEntityImpl,
                ).unwrap();
            }
            SecretType::Bat => {}
        }
    }

    pub fn player_collides(&mut self, player: &Player) -> bool {
        player.collision_aabb().intersects(&self.spawn_aabb)
    }
}

// pub struct ItemSecretEntity;
// 
// // this isn't necessarily a secret, simply an animation for one, can be re-used for blessings
// impl EntityImpl for ItemSecretEntity {
//     fn spawn(&mut self, entity: &mut Entity, buffer: PacketBuffer) {
//         let metadata_packet = &PacketEntityMetadata {
//             entity_id: VarInt(entity.id),
//             metadata: entity.metadata.clone(),
//         };
//         let velocity_packet = &EntityVelocity {
//             entity_id: VarInt(entity.id),
//             velocity_x: 0,
//             velocity_y: 0,
//             velocity_z: 0,
//         };
//         for player in entity.world_mut().players.values_mut() {
//             player.write_packet(metadata_packet);
//             player.write_packet(velocity_packet);
//         }
//     }
//     
//     fn tick(&mut self, entity: &mut Entity) {
//         if entity.ticks_existed == 20 {
//             // this makes sure entity is on ground and prevents it jitter-ing in air
//             entity.position.y -= 0.5;
//         } else if entity.ticks_existed % 20 == 0 {
//             // re-sync position and velocity,
//             // since it is really easy for item drop to de-sync
//             entity.last_position = DVec3::ZERO;
// 
//             for player in entity.world_mut().players.values_mut() {
//                 player.write_packet(&EntityVelocity {
//                     entity_id: VarInt(entity.id),
//                     velocity_x: 0,
//                     velocity_y: 0,
//                     velocity_z: 0,
//                 });
//             }
//         }
//         
//         // todo get correct values
//         const W: f64 = 3.0;
//         const H: f64 = 3.0;
//         
//         let aabb = AABB::new(
//             DVec3::new(entity.position.x - W, entity.position.y - H, entity.position.z - W),
//             DVec3::new(entity.position.x + W, entity.position.y + H, entity.position.z + W),
//         );
//         for player in entity.world_mut().players.values_mut() {
//             if player.collision_aabb().intersects(&aabb) { 
//                 player.write_packet(&CollectItem {
//                     item_entity_id: VarInt(entity.id),
//                     entity_id: VarInt(player.entity_id),
//                 });
//                 player.write_packet(&SoundEffect {
//                     sound: "random.pop",
//                     pos_x: player.position.x,
//                     pos_y: player.position.y,
//                     pos_z: player.position.z,
//                     volume: 0.2,
//                     pitch: 1.7619047,
//                 });
//                 entity.world_mut().despawn_entity(entity.id);
//                 break;
//             }
//         }
//     }
//     
// }
// 
// pub struct EssenceEntity;
// 
// impl EntityImpl for EssenceEntity {
//     
//     fn spawn(&mut self, entity: &mut Entity) {
//         for player in entity.world_mut().players.values_mut() {
//             player.write_packet(&EntityEquipment {
//                 entity_id: VarInt(entity.id),
//                 item_slot: 4,
//                 item_stack: Some(ItemStack {
//                     item: 397,
//                     stack_size: 1,
//                     metadata: 3,
//                     tag_compound: Some(NBT::with_nodes(vec![
//                         NBT::compound("SkullOwner", vec![
//                             NBT::string("Name", ""),
//                             NBT::string("Id", "e0f3e929-869e-3dca-9504-54c666ee6f23"),
//                             NBT::compound("Properties", vec![
//                                 NBT::list("textures", TAG_COMPOUND_ID,vec![
//                                     NBTNode::Compound({
//                                         let mut map = HashMap::new();
//                                         map.insert("Value".into(), NBTNode::String("ewogICJ0aW1lc3RhbXAiIDogMTYwMzYxMDQ0MzU4MywKICAicHJvZmlsZUlkIiA6ICIzM2ViZDMyYmIzMzk0YWQ5YWM2NzBjOTZjNTQ5YmE3ZSIsCiAgInByb2ZpbGVOYW1lIiA6ICJEYW5ub0JhbmFubm9YRCIsCiAgInNpZ25hdHVyZVJlcXVpcmVkIiA6IHRydWUsCiAgInRleHR1cmVzIiA6IHsKICAgICJTS0lOIiA6IHsKICAgICAgInVybCIgOiAiaHR0cDovL3RleHR1cmVzLm1pbmVjcmFmdC5uZXQvdGV4dHVyZS9lNDllYzdkODJiMTQxNWFjYWUyMDU5Zjc4Y2QxZDE3NTRiOWRlOWIxOGNhNTlmNjA5MDI0YzRhZjg0M2Q0ZDI0IgogICAgfQogIH0KfQ==".into()));
//                                         map.insert("Signature".into(), NBTNode::String("Mnf7PDLe+FPiO+wQ2St6XNRiiIXtZ3GuPTcLlM7pNQ6d6MXuzI7xXG24qaAMFuVwMB+F3dLYcaFlc+bWyi3Qm9msSq2mMUXdvzTamAslZHcdcTFNpppkYgdvkOhWK7W/amQyd2Q+pLDECe8Mg6gxBY17+xfaWlIynzEWEmHR+ye+hTC44kgiTZaYiRg7gpU002deY8WpX875cc5zJIroxVR52qHIV+suIMPwq47mpCp520J9R1HuYvvP/V3+PwL7skMlC1F/HHkG5A13fvSKMqq9XMsdqXR8qvWlcL5IQTS7ijtD9TZo8jcmhz/7HCXuJ912I1GqJp4hZ0Lqa0NB0TuI/giHr2i4yNzORe6oan47bpMXLoZWIrYZIOsF6wSObhwniF1jM/zUEkum9XswRImIvYYlmyLH+Kkh5uQJm244rOLPXmOZEid6PW5bhaSRpMOMpxboeOtjLbGC56Ev+DwoI37SrAYY6/LC7HwjVhvkcsLd/9BrF+Wl10bdLdsJEbd+TII59/45MM1x7+xgeAFU/ip0TjkMPfRLdNmfxOGssMFZOaM55iOb+8t4tOvXxnqeXpFCByDgPnqKV5zPXS1XMF2+5qEAv7ZKrqK8BLAHbWsKHHOMt1hJ8K+EgYfRDKq72YvN01ST288ysUv8b5stRu8O5uC+KvZXtnlGrKc=".into()));
//                                         map
//                                     })
//                                 ])
//                             ])
//                         ]),
//                     ])),
//                 }),
//             });
//         }
//     }
//     
//     fn tick(&mut self, entity: &mut Entity) {
//         entity.position.y += 0.04;
//         entity.yaw += 15.0;
//         
//         if entity.ticks_existed % 5 == 0 {
//             // todo, constants
//             let particle_packet = Particles {
//                 particle_id: 29,
//                 long_distance: true,
//                 x: entity.position.x as f32,
//                 y: entity.position.y as f32 + 1.5,
//                 z: entity.position.z as f32,
//                 offset_x: 0.0,
//                 offset_y: 0.0,
//                 offset_z: 0.0,
//                 speed: 0.06,
//                 count: 5,
//             };
//             let sound_packet = SoundEffect {
//                 sound: "note.harp",
//                 pos_x: entity.position.x,
//                 pos_y: entity.position.y + 1.5,
//                 pos_z: entity.position.z,
//                 volume: 1.0,
//                 pitch: 0.8 + ((entity.ticks_existed / 5) as f32 * 0.1),
//             };
//             for player in entity.world_mut().players.values_mut() {
//                 player.write_packet(&particle_packet);
//                 player.write_packet(&sound_packet);
//             }
//         }
//         
//         if entity.ticks_existed == 20 {
//             let sound_packet = SoundEffect {
//                 sound: "random.orb",
//                 pos_x: entity.position.x,
//                 pos_y: entity.position.y,
//                 pos_z: entity.position.z,
//                 volume: 1.0,
//                 pitch: 1.5,
//             };
//             for player in entity.world_mut().players.values_mut() {
//                 // for whatever reason, this sent twice on hypixel
//                 player.write_packet(&sound_packet);
//                 player.write_packet(&sound_packet);
//             }
//             entity.world_mut().despawn_entity(entity.id);
//         }
//     }
// }

/*
door success
[12:55:46] sound note.pling, 8.0 4.047619 -140.75 69.0 -154.75
[12:55:47]  Stivais opened a WITHER door!

door fail
[12:58:06] sound mob.endermen.portal, 8.0 0.0 -58.0 69.0 -171.75
[12:58:06]  You do not have the key for this door!
*/

/*
unrelated to secrets, but this is dungeon countdown
[12:43:22]  Stivais is now ready!
[12:43:22]  sound random.click, 0.55 2.0 -119.625 69.0 -173.375
[12:43:22]  Starting in 4 seconds.
[12:43:23]  sound random.click, 0.55 2.0 -117.75 69.0 -172.375
[12:43:23]  Starting in 3 seconds.
[12:43:24]  sound random.click, 0.55 2.0 -118.75 69.0 -173.0
[12:43:24]  Starting in 2 seconds.
[12:43:25]  sound random.click, 0.55 2.0 -118.75 69.0 -173.0
[12:43:25]  Starting in 1 second.
[12:43:26]  sound mob.enderdragon.growl, 1.0 1.0 -118.75 69.0 -173.0
*/

/*
[12:43:26]  sound mob.villager.haggle, 1.0 0.6984127 -118.75 69.0 -173.0
[12:43:26]  §e[NPC] §bMort§f: Here, I found this map when I first entered the dungeon.

[12:43:28]  sound mob.villager.haggle, 1.0 0.6984127 -117.875 69.0 -175.875
[12:43:28]  §e[NPC] §bMort§f: You should find it useful if you get lost.
[12:43:29]  sound mob.villager.haggle, 1.0 0.6984127 -110.875 71.0 -184.375
[12:43:29]  §e[NPC] §bMort§f: Good luck.

*/