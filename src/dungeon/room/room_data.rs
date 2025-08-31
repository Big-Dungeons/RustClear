use crate::dungeon::door::Door;
use crate::dungeon::room::room::{Room, RoomSegment};
use crate::server::block::blocks::Blocks;
use crate::utils::hasher::deterministic_hasher::DeterministicHashMap;
use crate::utils::seeded_rng::seeded_rng;
use rand::seq::IteratorRandom;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum RoomShape {
    OneByOne, // Fairy room, doors can vary
    OneByOneEnd, // A dead end, only one door
    OneByOneCross, // Four doors
    OneByOneStraight, // Two doors opposite each other
    OneByOneBend, // Two doors making an L bend
    OneByOneTriple, // Two opposite with one in the middle

    OneByTwo,
    OneByThree,
    OneByFour,
    TwoByTwo,
    L,
    Empty, // Shouldn't happen probably
}

impl RoomShape {
    pub fn from_str(value: &str) -> RoomShape {
        match value {
            "1x1" => Self::OneByOne,
            "1x1_E" => Self::OneByOneEnd,
            "1x1_X" => Self::OneByOneCross,
            "1x1_I" => Self::OneByOneStraight,
            "1x1_L" => Self::OneByOneBend,
            "1x1_3" => Self::OneByOneTriple,
            "1x2" => Self::OneByTwo,
            "1x3" => Self::OneByThree,
            "1x4" => Self::OneByFour,
            "2x2" => Self::TwoByTwo,
            "L" => Self::L,
            _ => unimplemented!(),
        }
    }

    pub fn from_segments(segments: &[RoomSegment], dungeon_doors: &[Door]) -> RoomShape {

        let unique_x = segments.iter()
            .map(|segment| segment.x)
            .collect::<HashSet<usize>>();

        let unique_z = segments.iter()
            .map(|segment| segment.z)
            .collect::<HashSet<usize>>();

        let not_long = unique_x.len() > 1 && unique_z.len() > 1;

        // Impossible for rooms to have < 1 or > 4 segments
        match segments.len() {
            1 => {
                let (shape, _) = Room::get_1x1_shape_and_type(segments, dungeon_doors);

                shape
            },
            2 => RoomShape::OneByTwo,
            3 => match not_long {
                true => RoomShape::L,
                false => RoomShape::OneByThree,
            },
            4 => match not_long {
                true => RoomShape::TwoByTwo,
                false => RoomShape::OneByFour,
            },
            _ => RoomShape::Empty,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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

impl RoomType {
    pub fn from_str(value: &str) -> RoomType {
        match value {
            "normal" => RoomType::Normal,
            "puzzle" => RoomType::Puzzle,
            "yellow" => RoomType::Yellow,
            "blood" => RoomType::Blood,
            "fairy" => RoomType::Fairy,
            "entrance" => RoomType::Entrance,
            "trap" => RoomType::Trap,
            "rare" => RoomType::Rare,
            _ => RoomType::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomData {
    pub name: String,
    pub id: String,
    pub shape: RoomShape,
    pub room_type: RoomType,
    pub bottom: i32,
    pub width: i32,
    pub length: i32,
    pub height: i32,
    pub block_data: Vec<Blocks>,
    pub crusher_data: Vec<Value>, // Needs to be parsed when rooms are generated
}

impl RoomData {
    pub fn from_raw_json(raw_data: &str) -> RoomData {
        let json_data: Value = serde_json::from_str(raw_data).unwrap(); // surely we just parse into a struct instead of doing this indexing?

        let name = json_data["name"].as_str().unwrap().to_string();
        let id = json_data["id"].as_str().unwrap().to_string();
        let shape = RoomShape::from_str(json_data["shape"].as_str().unwrap());
        let room_type = RoomType::from_str(json_data["type"].as_str().unwrap());
        let bottom = json_data["bottom"].as_number().unwrap().as_u64().unwrap() as i32;
        let width = json_data["width"].as_number().unwrap().as_u64().unwrap() as i32;
        let length = json_data["length"].as_number().unwrap().as_u64().unwrap() as i32;
        let height = json_data["height"].as_number().unwrap().as_u64().unwrap() as i32;

        let crusher_data: Vec<Value> = json_data["crushers"].as_array().unwrap_or(&Vec::new()).to_vec();

        let hex_data = json_data["block_data"].as_str().unwrap();

        let mut block_data: Vec<Blocks> = Vec::new();

        for i in (0..hex_data.len()).step_by(4) {
            let hex_str = hex_data.get(i..i+4).unwrap();
    
            let num = u16::from_str_radix(hex_str, 16).unwrap();
            let block = Blocks::from(num);
            
            block_data.push(block)
        }

        RoomData {
            name,
            id,
            shape,
            room_type,
            bottom,
            width,
            length,
            height,
            block_data,
            crusher_data,
        }
    }

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
            crusher_data: vec![],
        }
    }
}

pub fn get_random_data_with_type(
    room_type: RoomType,
    room_shape: RoomShape,
    data_storage: &DeterministicHashMap<usize, RoomData>,
    current_rooms: &[Room],
) -> RoomData {
    data_storage.iter()
        .filter(|data| {
            data.1.room_type == room_type &&
                data.1.shape == room_shape &&
                !current_rooms.iter().any(|room| room.room_data == *data.1) // No duplicate rooms
        })
        .map(|x| x.1)
        .choose(&mut seeded_rng())
        .unwrap_or(&RoomData::dummy())
        .clone()
}