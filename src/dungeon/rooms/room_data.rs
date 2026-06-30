use crate::core::block::block_entity::SkullRotation;
use crate::core::block::block_rotation::Rotation;
use crate::core::block::Block;
use crate::core::entity::entity_metadata::ArmorStandFlags;
use crate::core::network::protocol::nbt::int;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::PlayerSkin;
use crate::dungeon::rng::{DHashMap, DHashSet, SeededRng};
use crate::dungeon::rooms::RoomSegment;
use bevy::prelude::{Component, Deref, Resource};
use enumset::EnumSet;
use glam::{DVec3, IVec3, Vec3};
use include_dir::include_dir;
use rand::prelude::IteratorRandom;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Deserialize, Component, Clone)]
pub struct RoomData {
    pub name: String,
    pub id: String,
    pub shape: RoomShape,
    #[serde(rename = "type")]
    pub room_type: RoomType,
    pub bottom: i32,
    pub width: i32,
    pub length: i32,
    pub height: i32,

    #[serde(default)]
    pub secrets: Vec<SecretData>,
    #[serde(default, deserialize_with = "deserialize_block_entities")]
    pub block_entities: DHashMap<IVec3, BlockEntityData>,
    #[serde(default)]
    pub prop_entities: Vec<PropEntity>,

    // do we need to keep this once loaded into world?
    #[serde(deserialize_with = "deserialize_blocks")]
    pub block_data: Vec<Block>,
}

impl RoomData {
    pub fn dummy() -> RoomData {
        RoomData {
            name: String::from("Dummy"),
            id: String::from(""),
            shape: RoomShape::OneByOne,
            room_type: RoomType::Normal,
            bottom: 68,
            width: 31,
            length: 31,
            height: 30,
            secrets: Vec::new(),
            block_entities: DHashMap::default(),
            prop_entities: Vec::new(),
            block_data: Vec::new(),
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum RoomShape {
    #[serde(rename = "1x1")]   OneByOne,         // Varying doors (fairy room)
    #[serde(rename = "1x1_E")] OneByOneEnd,      // A dead end, only one door
    #[serde(rename = "1x1_X")] OneByOneCross,    // Four doors
    #[serde(rename = "1x1_I")] OneByOneStraight, // Two doors opposite each other
    #[serde(rename = "1x1_L")] OneByOneBend,     // Two doors making an L bend
    #[serde(rename = "1x1_3")] OneByOneTriple,   // Two opposite with one in the middle

    #[serde(rename = "1x2")] OneByTwo,
    #[serde(rename = "1x3")] OneByThree,
    #[serde(rename = "1x4")] OneByFour,
    #[serde(rename = "2x2")] TwoByTwo,
    #[serde(rename = "L")] L,

    Invalid, // Shouldn't happen probably
}

impl RoomShape {
    pub fn from_segments(segments: &[(RoomSegment, u8)]) -> RoomShape {
        let unique_x = segments
            .iter()
            .map(|(segment, _)| segment.x)
            .collect::<HashSet<usize>>();

        let unique_z = segments
            .iter()
            .map(|(segment, _)| segment.z)
            .collect::<HashSet<usize>>();

        let not_long = unique_x.len() > 1 && unique_z.len() > 1;

        // Impossible for rooms to have < 1 or > 4 segments
        match segments.len() {
            1 => {
                let bitmask: u8 = segments[0].1;
                match bitmask {
                    // Doors on all sides, never changes
                    0b1111 => RoomShape::OneByOneCross,
                    // Dead end 1x1
                    0b1000 => RoomShape::OneByOneEnd,
                    0b0100 => RoomShape::OneByOneEnd,
                    0b0010 => RoomShape::OneByOneEnd,
                    0b0001 => RoomShape::OneByOneEnd,
                    // Opposite doors
                    0b0101 => RoomShape::OneByOneStraight,
                    0b1010 => RoomShape::OneByOneStraight,
                    // L bend
                    0b0011 => RoomShape::OneByOneBend,
                    0b1001 => RoomShape::OneByOneBend,
                    0b1100 => RoomShape::OneByOneBend,
                    0b0110 => RoomShape::OneByOneBend,
                    // Triple door
                    0b1011 => RoomShape::OneByOneTriple,
                    0b1101 => RoomShape::OneByOneTriple,
                    0b1110 => RoomShape::OneByOneTriple,
                    0b0111 => RoomShape::OneByOneTriple,

                    _ => RoomShape::OneByOne,
                }
            }
            2 => RoomShape::OneByTwo,
            3 => match not_long {
                true => RoomShape::L,
                false => RoomShape::OneByThree,
            },
            4 => match not_long {
                true => RoomShape::TwoByTwo,
                false => RoomShape::OneByFour,
            },
            _ => RoomShape::Invalid,
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RoomType {
    Normal,
    Puzzle,
    Trap,
    Fairy,
    Entrance,
    Blood,
    Yellow,
    Rare,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretType {
    Essence {
        rotation: u8
    },
    Chest {
        rotation: Rotation,
    },
    Item,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretSpawnCondition {
    EnterArea {
        width: i32,
        height: i32,
    },
    EnterRoom,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct SecretData {
    pub secret: SecretType,
    pub position: IVec3,
    pub spawn_condition: SecretSpawnCondition,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BlockEntityData {
    Skull {
        #[serde(default)]
        skull_type: u8,
        #[serde(default)]
        texture: Option<String>,
        #[serde(default)]
        rotation: SkullRotation
    },
    Sign {
        lines: Vec<String>
    },
    Banner {
        patterns: Vec<String>
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct ArmorStandPose {
    #[serde(default)]
    pub head: Vec3,
    #[serde(default)]
    pub body: Vec3,
    #[serde(default = "left_arm")]
    pub left_arm: Vec3,
    #[serde(default = "right_arm")]
    pub right_arm: Vec3,
    #[serde(default = "left_leg")]
    pub left_leg: Vec3,
    #[serde(default = "right_leg")]
    pub right_leg: Vec3,
}

// scuffed ngl but whatever
const fn left_arm() -> Vec3 {
    Vec3 { x: -10.0, y: 0.0, z: -10.0 }
}

const fn right_arm() -> Vec3 {
    Vec3 { x: -15.0, y: 0.0, z: 10.0 }
}

const fn left_leg() -> Vec3 {
    Vec3 { x: -1.0, y: 0.0, z: -1.0 }
}

const fn right_leg() -> Vec3 {
    Vec3 { x: 1.0, y: 0.0, z: 1.0 }
}

#[derive(Deserialize)]
struct Item {
    item_id: usize,
    item_metadata: usize,
    leather_color: Option<i32>,
    skull_texture: Option<String>,
}

pub fn deserialize_item_stack<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<ItemStack>, D::Error> {
    let Item { item_id, item_metadata, leather_color, skull_texture } = Item::deserialize(deserializer)?;
    let mut stack = ItemStack::new()
        .item_id(item_id)
        .metadata(item_metadata);

    if let Some(color) = leather_color {
        stack.nbt_builder().get_or_insert_compound("display", [
            int("color", color)
        ]);
    }
    if let Some(texture) = skull_texture {
        stack = stack.skull_owner(
            Uuid::new_v4(),
            PlayerSkin::new(texture)
        );
    }
    Ok(Some(stack))
}

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PropEntity {
    ArmorStand {
        position: DVec3,
        yaw: f32,

        is_invisible: bool,
        #[serde(default)]
        flags: EnumSet<ArmorStandFlags>,

        #[serde(flatten)]
        pose: ArmorStandPose,

        #[serde(default, deserialize_with = "deserialize_item_stack")]
        hand: Option<ItemStack>,
        #[serde(default, deserialize_with = "deserialize_item_stack")]
        helmet: Option<ItemStack>,
        #[serde(default, deserialize_with = "deserialize_item_stack")]
        chestplate: Option<ItemStack>,
        #[serde(default, deserialize_with = "deserialize_item_stack")]
        leggings: Option<ItemStack>,
        #[serde(default, deserialize_with = "deserialize_item_stack")]
        boots: Option<ItemStack>,
    },
    Painting {
        position: IVec3,
        rotation: Rotation,
        variant: String,
    },
    Minecart {
        position: DVec3,
        yaw: f32,
        pitch: f32,
    }
}

// reason: glam serializes IVec3 as json array, but it needs to be string as key,
#[derive(Deserialize)]
struct BlockEntityEntry<V> {
    position: IVec3,
    block_entity: V,
}

fn deserialize_block_entities<'de, D: Deserializer<'de>>(d: D) -> Result<DHashMap<IVec3, BlockEntityData>, D::Error> {
    let entries = Vec::<BlockEntityEntry<BlockEntityData>>::deserialize(d)?;
    Ok(entries.into_iter().map(|e| (e.position, e.block_entity)).collect())
}


fn deserialize_blocks<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Block>, D::Error> {
    let hex_data = String::deserialize(d)?;
    let mut block_data: Vec<Block> = Vec::new();
    for index in (0..hex_data.len()).step_by(4) {
        let hex_str = hex_data.get(index..index + 4).ok_or_else(|| Error::custom("invalid hex length"))?;
        let num = u16::from_str_radix(hex_str, 16).map_err(Error::custom)?;
        block_data.push(Block::from(num));
    }
    Ok(block_data)
}

#[test]
fn room_data_test() {
    let rooms_directory = include_dir!("DungeonData/room_data/");
    rooms_directory
        .entries()
        .iter()
        .for_each(|file| {
            let file = file.as_file().unwrap();
            let contents = file.contents_utf8().unwrap();
            serde_json::from_str::<RoomData>(contents).expect("failed to serialize");
        });
}

// maybe have it static so roomdata can be singleton?, or drop once loaded
#[derive(Resource, Deref)]
pub struct RoomDataLookup(pub DHashMap<String, RoomData>);

impl Default for RoomDataLookup {
    fn default() -> Self {
        let rooms_directory = include_dir!("DungeonData/room_data/");
        let room_data: DHashMap<String, RoomData> = rooms_directory
            .entries()
            .iter()
            .map(|file| {
                let file = file.as_file().unwrap();
                let contents = file.contents_utf8().unwrap();
                let room_data: RoomData = serde_json::from_str(contents).unwrap();
                (room_data.id.clone(), room_data)
            }).collect();
        
        Self(room_data)
    }
}

pub fn random_room_data(
    room_lookup: &RoomDataLookup,
    room_type: RoomType,
    room_shape: RoomShape,
    existing_room_ids: &DHashSet<String>,
    mut rng: &mut SeededRng
) -> RoomData {
    room_lookup
        .values()
        .filter(|data| {
            data.room_type == room_type
            && data.shape == room_shape
            && !existing_room_ids.contains(&data.name)
        })
        .choose(&mut rng)
        .unwrap_or(&RoomData::dummy())
        .clone()
}