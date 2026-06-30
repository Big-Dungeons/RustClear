use crate::core::block::block_entity::{BlockEntity, BlockEntityType, SkullType};
use crate::core::block::block_rotation::{Rotate, Rotation};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::entity::components::equipment::Equipment;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::ArmorStandMetadata;
use crate::core::entity::object_metadata::{ObjectMetadata, PaintingVariant};
use crate::core::entity::Mob;
use crate::core::player::PlayerSkin;
use crate::dungeon::rng::DHashMap;
use crate::dungeon::rooms::room_data::{BlockEntityData, PropEntity, RoomData};
use crate::dungeon::DUNGEON_ORIGIN;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::{Commands, Component, Entity, Query, ResMut, Resource};
use glam::{ivec3, DVec3, IVec3, USizeVec2, Vec3Swizzles};
use std::cmp::{max, min};
use uuid::Uuid;

pub mod room_data;
pub mod secrets;
pub mod room_enter;

#[derive(Component)]
pub struct Room {
    pub rotation: Rotation,
    pub corner: IVec3,

    pub players: EntityHashSet,

    pub secrets: EntityHashSet,
    pub found_secrets: usize,
}

impl Room {

    pub fn new(segments: &[(RoomSegment, u8)], data: &RoomData) -> Self {
        let rotation = rotation_from_segments(segments);
        let segments: &[&RoomSegment] = &segments.iter().map(|(s, _)| s).collect::<Vec<_>>();
        let corner = corner_position(segments, data, rotation);
        Self {
            rotation,
            corner,
            players: EntityHashSet::new(),
            secrets: EntityHashSet::new(),
            found_secrets: 0,
        }
    }

    pub fn relative_to_world(&self, position: IVec3) -> IVec3 {
        let mut position = position.rotate(self.rotation.inverse());
        position.x += self.corner.x;
        position.z += self.corner.z;
        position
    }

    pub fn relative_dvec_to_world(&self, position: DVec3) -> DVec3 {
        let mut position = position.rotate(self.rotation.inverse());
        position.x += self.corner.x as f64;
        position.z += self.corner.z as f64;
        position
    }

    pub fn insert_player(&mut self, entity: Entity) {
        debug_assert!(!self.players.contains(&entity), "player already in this room");
        self.players.insert(entity);
    }

    pub fn remove_player(&mut self, entity: Entity) {
        debug_assert!(self.players.contains(&entity), "player was never in this room");
        self.players.remove(&entity);
    }
}

#[derive(Component, Copy, Clone)]
pub struct RoomSegment {
    pub x: usize,
    pub z: usize,
}

#[derive(Component)]
pub struct RoomNeighbours {
    pub room: Entity,
    pub door: Entity,
}

#[derive(Resource)]
pub struct RoomGridLookup([Option<Entity>; 36]);

impl Default for RoomGridLookup {
    fn default() -> Self {
        Self([const { None }; 36])
    }
}

impl RoomGridLookup {
    pub fn get(&self, grid_position: USizeVec2) -> Option<Entity> {
        let index = grid_position.x + grid_position.y * 6;
        if index >= 36 { return None; }
        self.0[index]
    }

    pub fn set(&mut self, grid_position: USizeVec2, entity: Entity) {
        let index = grid_position.x + grid_position.y * 6;
        if index >= 36 { return; }
        self.0[index] = Some(entity);
    }

    // maybe make a custom system param,
    // that has room grid lookup and room aabb query and gets rooms
    // from world position given that it should be inside a room
    pub fn get_from_world(&self, position: DVec3) -> Option<Entity> {
        let position = position.as_ivec3();
        if position.x < DUNGEON_ORIGIN.x || position.z < DUNGEON_ORIGIN.y {
            return None;
        }

        let grid_position = ((position.xz() - DUNGEON_ORIGIN) / 32).as_usizevec2();
        self.get(grid_position)
    }
}

// since y-axis with room segment positions is flipped (y+ is south)
// cw 90 and ccw 90 are flipped
pub fn rotation_from_segments(segments: &[(RoomSegment, u8)]) -> Rotation {
    let mut min_x = usize::MAX;
    let mut min_z = usize::MAX;
    let mut max_x = usize::MIN;
    let mut max_z = usize::MIN;

    for (segment, _) in segments {
        min_x = min(min_x, segment.x);
        min_z = min(min_z, segment.z);
        max_x = max(max_x, segment.x);
        max_z = max(max_z, segment.z);
    }

    let width = (max_x - min_x) + 1;
    let length = (max_z - min_z) + 1;

    match segments.len() {
        1 => {
            let bitmask = &segments[0].1;
            match bitmask {
                // Doors on all sides, never changes
                0b1111 => Rotation::None,
                // Opposite doors
                0b1010 => Rotation::None,
                0b0101 => Rotation::CounterClockwise90,
                // Dead end | L Bend | Triple Door
                0b1000 | 0b0011 | 0b1011 => Rotation::None,
                0b0100 | 0b1001 | 0b1101 => Rotation::Clockwise90,
                0b0010 | 0b1100 | 0b1110 => Rotation::Clockwise180,
                0b0001 | 0b0110 | 0b0111 => Rotation::CounterClockwise90,
                _ => Rotation::None,
            }
        }
        2 => match width == 1 {
            true => Rotation::None,
            false => Rotation::CounterClockwise90,
        },
        3 => {
            // L room
            if width == 2 && length == 2 {
                let (corner, _) = segments.iter().find(|(a, _)| {
                    segments.iter().all(|(b, _)| {
                        a.x.abs_diff(b.x) + a.z.abs_diff(b.z) <= 1
                    })
                }).expect("Invalid L room: Segments:");

                match (corner.x, corner.z) {
                    (x, z) if x == min_x && z == min_z => Rotation::Clockwise90,
                    (x, z) if x == max_x && z == min_z => Rotation::Clockwise180,
                    (x, z) if x == max_x && z == max_z => Rotation::CounterClockwise90,
                    _ => Rotation::None,
                }
            } else {
                match width == 1 {
                    true => Rotation::None,
                    false => Rotation::CounterClockwise90,
                }
            }
        },
        4 => {
            if width == 2 && length == 2 {
                Rotation::None
            } else {
                match width == 1 {
                    true => Rotation::None,
                    false => Rotation::CounterClockwise90,
                }
            }
        },
        _ => unreachable!(),
    }
}

fn corner_position(segments: &[&RoomSegment], data: &RoomData, rotation: Rotation) -> IVec3 {
    let min_x = segments.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
    let min_z = segments.iter().min_by(|a, b| a.z.cmp(&b.z)).unwrap().z;
    let x = min_x as i32 * 32 + DUNGEON_ORIGIN.x;
    let y = 68;
    let z = min_z as i32 * 32 + DUNGEON_ORIGIN.y;
    match rotation {
        Rotation::None => ivec3(x, y, z),
        Rotation::Clockwise90 => ivec3(x + data.length - 1, y, z),
        Rotation::Clockwise180 => ivec3(x + data.length - 1, y, z + data.width - 1),
        Rotation::CounterClockwise90 => ivec3(x, y, z + data.width - 1),
    }
}

// might have issue where rooms overlap, but im pretty sure its fixed
pub fn load_rooms_into_world(
    query: Query<(&Room, &RoomData)>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands
) {
    // maybe make this a global resource
    let mut uuid_map: DHashMap<String, Uuid> = DHashMap::default();

    for (room, data) in query.iter() {
        for (index, block) in data.block_data.iter().enumerate() {
            if *block == Block::Air {
                continue;
            }

            let index = index as i32;
            let x = index % data.width;
            let z = (index / data.width) % data.length;
            let y = data.bottom + index / (data.width * data.length);

            let position = ivec3(x, y, z).rotate(room.rotation.inverse());
            let block = block.rotate(room.rotation);
            chunks.set_block_at(block, (room.corner.x + position.x, y, room.corner.z + position.z));
        }

        for (position, block) in data.block_entities.iter() {
            let mut position = position.rotate(room.rotation.inverse());
            position.y += data.bottom;
            position.x += room.corner.x;
            position.z += room.corner.z;

            match block {
                BlockEntityData::Skull { skull_type, texture, rotation } => {
                    let rotation = rotation.rotate(room.rotation);

                    let skull_type = match skull_type {
                        0 => SkullType::Skeleton,
                        1 => SkullType::WitherSkeleton,
                        2 => SkullType::Zombie,
                        3 => {
                            let Some(texture) = texture else {
                                continue;
                            };

                            let uuid = *uuid_map.entry(texture.clone()).or_insert(Uuid::new_v4());
                            SkullType::PlayerHead {
                                uuid,
                                skin: PlayerSkin::new(texture.clone()),
                            }
                        },
                        4 => SkullType::Creeper,
                        _ => unreachable!()
                    };

                    commands.spawn(BlockEntity::new(
                        position, BlockEntityType::Skull { rotation, skull_type })
                    );
                }
                BlockEntityData::Banner { .. } => {}
                _ => {}
            }
        }

        for prop in data.prop_entities.iter() {
            match prop {
                PropEntity::ArmorStand {
                    position, yaw,
                    is_invisible, flags,
                    pose,
                    hand, helmet, chestplate, leggings, boots,
                } => {
                    let position = room.relative_dvec_to_world(*position);
                    let yaw = yaw.rotate(room.rotation);

                    let mut equipment = Equipment::new();
                    equipment.hand = hand.clone();
                    equipment.armor[3] = helmet.clone();
                    equipment.armor[2] = chestplate.clone();
                    equipment.armor[1] = leggings.clone();
                    equipment.armor[0] = boots.clone();

                    commands.spawn((
                        Mob::new(
                            ArmorStandMetadata::new()
                                .invisible(*is_invisible)
                                .armor_stand_flags(*flags)
                                .head(pose.head)
                                .body(pose.body)
                                .left_arm(pose.left_arm)
                                .right_arm(pose.right_arm)
                                .left_leg(pose.left_leg)
                                .right_leg(pose.right_leg)
                        ),
                        Transform {
                            position,
                            yaw,
                            pitch: 0.0,
                        },
                        equipment
                    ));
                }
                // if more paintings are somehow needed, this will need to be adjusted
                PropEntity::Painting { position, rotation, variant } => {
                    commands.spawn((
                        Mob::new_object(ObjectMetadata::Painting {
                            painting: match variant.as_str() {
                                "Wanderer" => PaintingVariant::Wanderer,
                                _ => PaintingVariant::Graham,
                            },
                            rotation: rotation.rotate(room.rotation),
                        }),
                        Transform::new(room.relative_to_world(*position))
                    ));
                }
                PropEntity::Minecart { position, yaw, pitch } => {
                    commands.spawn((
                        Mob::new_object(ObjectMetadata::Minecart),
                        Transform {
                            position: room.relative_dvec_to_world(*position),
                            yaw: *yaw,
                            pitch: *pitch,
                        }
                    ));
                }
            }
        }
    }
    println!("Done loading")
}