use crate::chunk::chunk_grid::ChunkGrid;
use crate::dungeon::items::DungeonItem;
use crate::entity::components::transform::Transform;
use crate::network::protocol::play::clientbound::Relative;
use crate::player::inventory::item_stack::ItemStack;
use crate::player::inventory::{Inventory, Item};
use crate::player::known_state::KnownState;
use crate::player::movement::SetPosition;
use crate::player::use_item::PlayerRightClick;
use bevy::prelude::{MessageReader, MessageWriter, Query, Res};
use glam::{dvec3, DVec3, IVec3};
use indoc::indoc;

pub struct AspectOfTheVoid;

impl Item for AspectOfTheVoid {
    fn item_stack(&self) -> ItemStack {
        ItemStack::new()
            .item_id(277)
            .name("§6Aspect of the Void")
            .lore(indoc! {r#"

                §6Ability: Instant Transmission §e§lRIGHT CLICK
                §7Teleport §a12 blocks §7ahead of you and
                §7gain §a+50 §r✦ Speed §7for §a3 seconds.

                §6Ability: Ether Transmission §e§lSNEAK RIGHT CLICK
                §7Teleport to your targeted block up
                §7to §a61 blocks §7away

                §6§l§kU§r§6§l LEGENDARY SWORD §kU
            "#})
            .skyblock_id("ASPECT_OF_THE_VOID")
            .unbreakable()
            .hide_all_flags()
    }
}

pub fn use_aspect_of_the_void(
    mut events: MessageReader<PlayerRightClick>,
    query: Query<(&Transform, &Inventory, &KnownState)>,
    // maybe custom event specifically for checking if position allows teleporting to (like trap)
    mut set_position: MessageWriter<SetPosition>,
    chunks: Res<ChunkGrid>,
) {
    for PlayerRightClick { client, .. } in events.read() {
        let (transform, inventory, state) = query.get(*client).unwrap();

        // kind of suboptimal, but there aren't many items
        if !matches!(inventory.held_item(), Some(DungeonItem::AspectOfTheVoid(_))) {
            return;
        }

        if state.sneaking {
            let mut start = transform.position;
            start.y += 1.54;
            let mut end = transform.rotation_vec().normalize().as_dvec3() * 61.0;
            end += start;

            if let EtherResult::Valid(result) = traverse_voxels(&chunks, start, end) {
                let position = result.as_dvec3() + dvec3(0.5, 1.05, 0.5);

                set_position.write(SetPosition {
                    client: *client,
                    transform: Transform { position, yaw: 0.0, pitch: 0.0 },
                    relative_flags: Relative::Yaw | Relative::Pitch,
                });
            }
        } else {
            let InstantTransmissionResult {
                position,
                blocks_in_the_way
            } = instant_transmission(12.0, transform, &chunks);

            if let Some(position) = position {
                let position = position.as_dvec3() + dvec3(0.5, 0.0, 0.5);
                set_position.write(SetPosition {
                    client: *client,
                    transform: Transform { position, yaw: 0.0, pitch: 0.0 },
                    relative_flags: Relative::Yaw | Relative::Pitch,
                });

                if blocks_in_the_way {
                    // todo:
                }
            }
        }
    }
}

enum EtherResult {
    Valid(IVec3),
    Failed,
}

fn traverse_voxels(chunks: &ChunkGrid, start: DVec3, end: DVec3) -> EtherResult {
    let dir = end - start;
    let step = dir.signum().as_ivec3();
    let inv_dir = dir.map(|f| if f != 0.0 { 1.0 / f } else { f64::MAX });
    let t_delta = (inv_dir * step.as_dvec3()).abs();

    let mut current = start.floor().as_ivec3();
    let end = end.floor().as_ivec3();

    let mut t_max = {
        let next = current.as_dvec3() + step.as_dvec3().max(DVec3::ZERO);
        ((next - start) * inv_dir).abs()
    };

    for _ in 0..1000 {
        let current_block = (chunks.get_block_at(current).get_blockstate_id() >> 4) as usize;
        if !is_valid_ether_warp_block(current_block) {
            return if is_valid(chunks, current + IVec3::Y) {
                EtherResult::Valid(current)
            } else {
                EtherResult::Failed
            };
        }

        if current == end {
            return EtherResult::Failed;
        }

        if t_max.x <= t_max.y && t_max.x <= t_max.z {
            t_max.x += t_delta.x;
            current.x += step.x;
        } else if t_max.y <= t_max.z {
            t_max.y += t_delta.y;
            current.y += step.y;
        } else {
            t_max.z += t_delta.z;
            current.z += step.z;
        }
    }
    EtherResult::Failed
}

pub struct InstantTransmissionResult {
    pub position: Option<IVec3>,
    pub blocks_in_the_way: bool,
}

pub fn instant_transmission(
    distance: f64,
    player_transform: &Transform,
    chunks: &ChunkGrid,
) -> InstantTransmissionResult {
    let mut start = player_transform.position;
    start.y += 1.62;

    const STEP_LEN: f64 = 0.5;
    let step_amount = (distance / STEP_LEN).ceil() as usize;
    let direction_vec = player_transform.rotation_vec().normalize().as_dvec3() * STEP_LEN;

    let mut current_position = start;
    let mut last_checked: IVec3;
    let mut current_block: Option<IVec3> = None;
    let mut blocks_in_the_way: bool = false;

    // this small section acts as the first iteration.
    // because even if it is invalid teleport position,
    // it still sets last checked, and won't stop the ray-cast.
    last_checked = mc_floor_vec3(current_position);
    current_position += direction_vec;

    if is_valid(chunks, last_checked) {
        current_block = Some(last_checked);
    }

    for _ in 1..step_amount {
        let block_pos = mc_floor_vec3(current_position);
        current_position += direction_vec;

        if block_pos == last_checked {
            continue;
        }
        if !is_valid(chunks, block_pos) {
            blocks_in_the_way = true;
            break;
        }

        last_checked = block_pos;
        current_block = Some(block_pos);
    }

    InstantTransmissionResult {
        position: current_block,
        blocks_in_the_way
    }
}

fn mc_floor(value: f64) -> i32 {
    let i = value as i32;
    if value < i as f64 { i - 1 } else { i }
}

fn mc_floor_vec3(v: DVec3) -> IVec3 {
    IVec3::new(mc_floor(v.x), mc_floor(v.y), mc_floor(v.z))
}

// should compile to a lookup
const fn is_valid_ether_warp_block(id: usize) -> bool {
    matches!(id,
        0 | 6 | 9 | 11 | 30 | 31 | 32 | 36 | 37 | 38 | 39 | 40 | 50 | 51 | 55 | 59 |
        65 | 66 | 69 | 76 | 77 | 78 | 93 | 94 | 104 | 105 | 106 | 111 | 115 | 131 |
        132 | 140 | 141 | 142 | 143 | 144 | 149 | 150 | 157 | 171 | 175
    )
}

fn is_valid(chunk_grid: &ChunkGrid, position: IVec3) -> bool {
    let block1 = chunk_grid.get_block_at(position).get_blockstate_id() >> 4;
    let block2 = chunk_grid.get_block_at(position + IVec3::Y).get_blockstate_id() >> 4;
    is_valid_ether_warp_block(block1 as usize) && is_valid_ether_warp_block(block2 as usize)
}