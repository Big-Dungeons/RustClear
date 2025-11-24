use crate::assets::load_asset::{DoorDataAssets, DungeonStorageAssets, FaviconAssets, LoadAsset, RoomDataAssets};
use crate::dungeon::door::door::DoorType;
use crate::dungeon::room::room_data::RoomData;
use anyhow::anyhow;
use server::block::Block;
use server::utils::hasher::deterministic_hasher::DeterministicHashMap;
use std::collections::HashMap;
use std::{io::{Cursor, Read}, path::{Path, PathBuf}, sync::OnceLock};
use tokio::fs;
use zip::ZipArchive;

static ASSETS: OnceLock<Assets> = OnceLock::new();

pub struct Assets {
    pub dungeon_seeds: Vec<&'static str>,
    pub door_data: HashMap<DoorType, Vec<Vec<Block>>>,
    pub room_data: DeterministicHashMap<usize, RoomData>,
    pub icon_data: &'static str,
}

impl Assets {
    pub async fn try_load(asset_path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            dungeon_seeds: DungeonStorageAssets::load_asset(asset_path).await?,
            door_data: DoorDataAssets::load_asset(asset_path).await?,
            room_data: RoomDataAssets::load_asset(asset_path).await?,
            icon_data: FaviconAssets::load_asset(asset_path).await?,
        })
    }
}

pub async fn load_assets(asset_path: &str, repo: &str) -> anyhow::Result<()> {
    println!("Loading assets!");
    let asset_path = Path::new(asset_path);
    
    let assets = match Assets::try_load(asset_path).await {
        Ok(assets) => assets,
        Err(_) => {
            download_assets(asset_path, repo).await?;
            Assets::try_load(asset_path).await?
        }
    };
    
    ASSETS.set(assets).map_err(|_| anyhow!("Failed to set assets!"))?;
    
    println!("Finished loading assets!");
    Ok(())
}

pub fn get_assets() -> &'static Assets {
    ASSETS.get().expect("load_assets should've been called first!")
}

async fn download_assets(base: &Path, url: &str) -> anyhow::Result<()> {
    println!("downloading assets...");
    fs::create_dir_all(base).await?;
    
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?.to_vec();
    
    let reader = Cursor::new(bytes);
    let mut zip = ZipArchive::new(reader)?;
    
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let path = file.mangled_name();
        
        let components = path.components().collect::<Vec<_>>();
        if let Some(pos) = components.iter().position(|c| c.as_os_str() == "assets") {
            let rel_path: PathBuf = components[pos + 1..].iter().collect();
            let out = base.join(rel_path);
            
            if file.is_dir() {
                fs::create_dir_all(&out).await?;
            } else if !out.exists() {
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent).await?;
                }
                let mut contents = Vec::new();
                Read::read_to_end(&mut file, &mut contents)?;
                fs::write(&out, contents).await?;
            }
        }
    }
    
    Ok(())
}