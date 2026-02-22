use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room_data::RoomType;
use glam::{dvec3, vec3, DVec3, IVec3};
use indoc::indoc;
use server::constants::{Particle, Sound};
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::network::protocol::play::clientbound::{PositionLook, Relative};
use server::player::packet_processing::BlockInteractResult;
use server::utils::bitset::BitSet;
use server::world::chunk::chunk_grid::ChunkGrid;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct AspectOfTheVoid;

impl DungeonItem for AspectOfTheVoid {

    fn on_interact(&self, player: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {
        if player.is_sneaking {
            etherwarp(player)
        } else {
            instant_transmission(player, 12.0)
        }
    }

    fn item_stack(&self) -> ItemStack {
        ItemStack {
            item: 277,
            stack_size: 1,
            metadata: 0,
            tag_compound: Some(NBT::with_nodes(vec![
                NBT::compound("display", vec![
                    NBT::string("Name", "§6Aspect of the Void"),
                    NBT::list_from_string("Lore", indoc! {r#"

                            §6Ability: Instant Transmission §e§lRIGHT CLICK
                            §7Teleport §a12 blocks §7ahead of you and
                            §7gain §a+50 §r✦ Speed §7for §a3 seconds.

                            §6Ability: Ether Transmission §e§lSNEAK RIGHT CLICK
                            §7Teleport to your targeted block up
                            §7to §a61 blocks §7away

                            §6§l§kU§r§6§l LEGENDARY SWORD §kU
                        "#})
                ]),
                NBT::compound("ExtraAttributes", vec![
                    NBT::string("id", "ASPECT_OF_THE_VOID"),
                ]),
                NBT::byte("Unbreakable", 1),
                NBT::byte("HideFlags", 127),
            ])),
        }
    }
}

pub(super) const VALID_ETHER_WARP_BLOCK_IDS: BitSet<3> = BitSet::new(
    &[
        0, 6, 9, 11, 30, 31, 32, 36, 37, 38, 39, 40, 50, 51, 55, 59, 65, 66, 69, 76, 77, 78,
        93, 94, 104, 105, 106, 111, 115, 131, 132, 140, 141, 142, 143, 144, 149, 150, 157, 171, 175
    ]
);

enum EtherResult {
    Valid(i32, i32, i32),
    Failed,
}

pub fn etherwarp(player: &mut Player<DungeonPlayer>) {
    // temporary, but just to test,
    // since some puzzles let you teleport in them, and others don't
    if let Some(room_rc) = player.extension.get_current_room() {
        match room_rc.borrow().data.room_type {
            RoomType::Trap | RoomType::Puzzle => return,
            _ => {}
        }
    };

    let mut start_pos = player.position;
    start_pos.y += 1.54; // assume always sneaking
    let mut end_pos = player.rotation_vec().normalize().as_dvec3() * 61.0;
    end_pos += start_pos;

    if let EtherResult::Valid(x, y, z) = traverse_voxels(&player.world().chunk_grid, start_pos, end_pos) {

        let position = dvec3(x as f64 + 0.5, y as f64 + 1.05, z as f64 + 0.5);
        let dungeon = &player.world().extension;

        if let Some((room_rc, _)) = dungeon.get_room(&position, player.collision_aabb_at(&position)) {
            if room_rc.borrow().data.room_type == RoomType::Puzzle {
                return;
            }
        }
        player.world_mut().spawn_particle(
            Particle::SpellWitch,
            player.position.as_vec3(),
            vec3(0.25, 1.0, 0.25),
            25
        );
        player.write_packet(&PositionLook {
            x: position.x,
            y: position.y,
            z: position.z,
            yaw: 0.0,
            pitch: 0.0,
            flags: Relative::Yaw | Relative::Pitch,
        });
        player.play_sound_at(Sound::EnderDragonHit, 1.0, 0.54, position);
    }
}

fn traverse_voxels(chunk_grid: &ChunkGrid<Dungeon>, start: DVec3, end: DVec3) -> EtherResult {
    let (x0, y0, z0) = (start.x, start.y, start.z);
    let (x1, y1, z1) = (end.x, end.y, end.z);


    let (mut x, mut y, mut z) = start.floor().as_ivec3().into();
    let (end_x, end_y, end_z) = end.floor().as_ivec3().into();

    let dir_x = x1 - x0;
    let dir_y = y1 - y0;
    let dir_z = z1 - z0;

    let step_x = dir_x.signum() as i32;
    let step_y = dir_y.signum() as i32;
    let step_z = dir_z.signum() as i32;

    let inv_dir_x = if dir_x != 0.0 { 1.0 / dir_x } else { f64::MAX };
    let inv_dir_y = if dir_y != 0.0 { 1.0 / dir_y } else { f64::MAX };
    let inv_dir_z = if dir_z != 0.0 { 1.0 / dir_z } else { f64::MAX };

    let t_delta_x = (inv_dir_x * step_x as f64).abs();
    let t_delta_y = (inv_dir_y * step_y as f64).abs();
    let t_delta_z = (inv_dir_z * step_z as f64).abs();

    // t_max initialization follows the "next voxel boundary" logic
    let mut t_max_x = ((x as f64 + if step_x > 0 { 1.0 } else { 0.0 } - x0) * inv_dir_x).abs();
    let mut t_max_y = ((y as f64 + if step_y > 0 { 1.0 } else { 0.0 } - y0) * inv_dir_y).abs();
    let mut t_max_z = ((z as f64 + if step_z > 0 { 1.0 } else { 0.0 } - z0) * inv_dir_z).abs();

    for _ in 0..1000 {
        // Check block at current voxel coordinates
        let current_block = chunk_grid.get_block_at(x, y, z);

        if !VALID_ETHER_WARP_BLOCK_IDS.contains((current_block.get_blockstate_id() >> 4) as usize) {
            let block_up1 = chunk_grid.get_block_at(x, y + 1, z).get_blockstate_id() >> 4;
            let block_up2 = chunk_grid.get_block_at(x, y + 2, z).get_blockstate_id() >> 4;

            return if VALID_ETHER_WARP_BLOCK_IDS.contains(block_up1 as usize) && VALID_ETHER_WARP_BLOCK_IDS.contains(block_up2 as usize) {
                EtherResult::Valid(x, y, z)
            } else {
                EtherResult::Failed
            }
        }

        if x == end_x && y == end_y && z == end_z {
            return EtherResult::Failed;
        }

        if t_max_x <= t_max_y && t_max_x <= t_max_z {
            t_max_x += t_delta_x;
            x += step_x;
        } else if t_max_y <= t_max_z {
            t_max_y += t_delta_y;
            y += step_y;
        } else {
            t_max_z += t_delta_z;
            z += step_z;
        }
    }

    EtherResult::Failed
}




pub fn instant_transmission(
    player: &mut Player<DungeonPlayer>,
    distance: f64,
) {
    if let Some(room_rc) = player.extension.get_current_room() {
        match room_rc.borrow().data.room_type {
            RoomType::Trap | RoomType::Puzzle => return,
            _ => {}
        }
    };

    let chunk_grid = &player.world().chunk_grid;

    let mut start = player.position;
    start.y += 1.62;

    const STEP_LEN: f64 = 0.5;
    let step_amount = (distance / STEP_LEN).ceil() as usize;
    let direction_vec = player.rotation_vec().normalize().as_dvec3() * STEP_LEN;

    let mut current_position = start;
    let mut last_checked: IVec3;
    let mut current_block: Option<IVec3> = None;
    let mut block_in_way: bool = false;

    // this small section acts as the first iteration.
    // because even if it is invalid teleport position,
    // it still sets last checked, and won't stop the ray-cast.
    last_checked = mc_floor_vec3(current_position);
    current_position += direction_vec;

    if is_valid(chunk_grid, last_checked) {
        current_block = Some(last_checked);
    }

    for _ in 1..step_amount {
        let block_pos = mc_floor_vec3(current_position);
        current_position += direction_vec;

        if block_pos == last_checked {
            continue;
        }
        if !is_valid(chunk_grid, block_pos) {
            block_in_way = true;
            break;
        }

        last_checked = block_pos;
        current_block = Some(block_pos);
    }

    // todo: sounds
    if let Some(position) = current_block {
        let position = dvec3(position.x as f64 + 0.5, position.y as f64, position.z as f64 + 0.5);
        let dungeon = &player.world().extension;

        if let Some((room_rc, _)) = dungeon.get_room(&position, player.collision_aabb_at(&position)) {
            if room_rc.borrow().data.room_type == RoomType::Puzzle {
                return;
            }
        }
        player.write_packet(&PositionLook {
            x: position.x,
            y: position.y,
            z: position.z,
            yaw: 0.0,
            pitch: 0.0,
            flags: Relative::Yaw | Relative::Pitch,
        })
    }
    if block_in_way {
        // play block in the way sound
    }
}

fn is_valid(chunk_grid: &ChunkGrid<Dungeon>, position: IVec3) -> bool {
    let block1 = chunk_grid.get_block_at(position.x, position.y, position.z).get_blockstate_id() >> 4;
    let block2 = chunk_grid.get_block_at(position.x, position.y + 1, position.z).get_blockstate_id() >> 4;
    VALID_ETHER_WARP_BLOCK_IDS.contains(block1 as usize) && VALID_ETHER_WARP_BLOCK_IDS.contains(block2 as usize)
}

fn mc_floor(value: f64) -> i32 {
    let i = value as i32;
    if value < i as f64 { i - 1 } else { i }
}

fn mc_floor_vec3(v: DVec3) -> IVec3 {
    IVec3::new(mc_floor(v.x), mc_floor(v.y), mc_floor(v.z))
}

