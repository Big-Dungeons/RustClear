use anyhow::Context;
use base64::{engine::general_purpose, Engine};
use std::path::Path;
use tokio::fs;

use crate::{block::blocks::Blocks, dungeon::room::room_data::RoomData, utils::hasher::deterministic_hasher::DeterministicHashMap};

pub(super) trait LoadAsset {
    const SUBPATH: &'static str;
    type Output;
    
    async fn load_asset(path: &Path) -> anyhow::Result<Self::Output>;
}

pub(super) struct RoomDataAssets;
pub(super) struct DoorDataAssets;
pub(super) struct FaviconAssets;
pub(super) struct DungeonStorageAssets;

impl LoadAsset for RoomDataAssets {
    const SUBPATH: &'static str = "room_data";
    type Output = DeterministicHashMap<usize, RoomData>;
    
    async fn load_asset(path: &Path) -> anyhow::Result<Self::Output> {
        let path = path.join(Self::SUBPATH);
        let mut map: DeterministicHashMap<usize, RoomData> = DeterministicHashMap::default();
        let mut entries = fs::read_dir(path).await?;
        while let Some(file) = entries.next_entry().await? {
            let name = file.file_name();
            let file = fs::read(file.path()).await?;
            let contents = std::str::from_utf8(&file)?;
            let room_data = RoomData::from_raw_json(contents);

            let name_parts: Vec<&str> = name.to_str().context("Failed to convert file name to str")?.split(",").collect();
            let room_id = name_parts.first().unwrap().parse::<usize>().unwrap();
            
            map.insert(room_id, room_data);
        }
        Ok(map)
    }
}

impl LoadAsset for DoorDataAssets {
    const SUBPATH: &'static str = "door_data/doors.txt";
    type Output = Vec<Vec<Blocks>>;
    
    async fn load_asset(path: &Path) -> anyhow::Result<Self::Output> {
        let path = path.join(Self::SUBPATH);
        let storage = String::from_utf8(fs::read(path).await?)?;
        
        Ok(storage.split("\n").map(|line| {
            let mut blocks: Vec<Blocks> = Vec::new();
    
            for i in (0..line.len() - 1).step_by(4) {
                let substr = line.get(i..i + 4).unwrap();
                let state = u16::from_str_radix(substr, 16).unwrap();
                blocks.push(Blocks::from(state));
            }
    
            blocks
        }).collect())
    }
}

impl LoadAsset for DungeonStorageAssets {
    const SUBPATH: &'static str = "dungeon_storage/dungeons.txt";
    type Output = Vec<&'static str>;
    
    async fn load_asset(path: &Path) -> anyhow::Result<Self::Output> {
        let path = path.join(Self::SUBPATH);
        let storage = Box::leak(String::from_utf8(fs::read(path).await?)?.into_boxed_str());
        Ok(storage.split(',').collect())
    }
}

impl LoadAsset for FaviconAssets {
    const SUBPATH: &'static str = "favicon.png";
    type Output = String;
    
    async fn load_asset(path: &Path) -> anyhow::Result<Self::Output> {
        let bytes = fs::read(path.join(Self::SUBPATH)).await?;
        Ok(general_purpose::STANDARD.encode(&bytes))
    }
}