use crate::block::Block;
use crate::dungeon::rooms::RoomSegment;
use bevy::prelude::{Component, Deref, Resource};
use include_dir::include_dir;
use rand::prelude::IteratorRandom;
use rand::rng;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};

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
            block_data: vec![],
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum RoomShape {
    #[serde(rename = "1x1")]   OneByOne,           // Varying doors (fairy room)
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
pub struct RoomDataLookup(pub HashMap<usize, RoomData>);

impl Default for RoomDataLookup {
    fn default() -> Self {
        let rooms_directory = include_dir!("DungeonData/room_data/");
        let room_data: HashMap<usize, RoomData> = rooms_directory
            .entries()
            .iter()
            .map(|file| {
                let file = file.as_file().unwrap();

                let name = file.path().file_name().unwrap().to_str().unwrap();
                let name_parts: Vec<&str> = name.split(",").collect();
                let room_id = name_parts.first().unwrap().parse::<usize>().unwrap();

                let contents = file.contents_utf8().unwrap();
                let room_data: RoomData = serde_json::from_str(contents).unwrap();

                (room_id, room_data)
            }).collect();
        
        Self(room_data)
    }
}

pub fn random_room_data(
    room_lookup: &RoomDataLookup,
    room_type: RoomType,
    room_shape: RoomShape,
    existing_room_ids: &HashSet<String>,
) -> RoomData {
    room_lookup
        .values()
        .filter(|data| {
            data.room_type == room_type
            && data.shape == room_shape
            && !existing_room_ids.contains(&data.name)
        })
        .choose(&mut rng())
        // todo:
        // .choose(&mut seeded_rng())
        .unwrap_or(&RoomData::dummy())
        .clone()
}