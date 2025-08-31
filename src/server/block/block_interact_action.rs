use crate::dungeon::dungeon_state::DungeonState;
use crate::dungeon::room::secrets::DungeonSecret;
use crate::net::protocol::play::clientbound::{BlockAction, SoundEffect};
use crate::server::block::block_position::BlockPos;
use crate::server::block::blocks::Blocks;
use crate::server::entity::entity::NoEntityImpl;
use crate::server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use crate::server::player::player::Player;
use crate::server::utils::dvec3::DVec3;
use std::cell::RefCell;
use std::rc::Rc;

pub enum BlockInteractAction {
    WitherDoor {
        door_index: usize
    },
    BloodDoor {
        door_index: usize
    },
    Chest {
        secret: Rc<RefCell<DungeonSecret>>,
    },
    WitherEssence {
        secret: Rc<RefCell<DungeonSecret>>
    },
    // mainly for quick debug,
    Callback(fn(&Player, &BlockPos)),
}


impl BlockInteractAction {
    pub fn interact(&self, player: &mut Player, block_pos: &BlockPos) {
        match self {
            Self::WitherDoor { door_index: id } |
            Self::BloodDoor { door_index: id } => {

                let world = &mut player.server_mut().world;
                let dungeon = &mut player.server_mut().dungeon;

                if let DungeonState::Started { .. } = dungeon.state {
                    // todo check if player has a key
                    let door = &dungeon.doors[*id];
                    door.open_door(world);
                }
            }

            Self::Chest { secret } => {
                let mut secret = secret.borrow_mut();
                if !secret.obtained {
                    // maybe make this a packet where it is sent to all players
                    player.write_packet(&BlockAction {
                        block_pos: block_pos.clone(),
                        event_id: 1,
                        event_data: 1,
                        block_id: 54,
                    });
                    player.write_packet(&SoundEffect {
                        sound: "random.chestopen",
                        pos_x: block_pos.x as f64,
                        pos_y: block_pos.y as f64,
                        pos_z: block_pos.z as f64,
                        volume: 1.0,
                        pitch: 0.975,
                    });
                    secret.obtained = true;
                }
                /*
[19:37:29] sound random.chestopen, 0.5 0.9206349 -94.0 82.0 -51.0
[19:37:29] sound random.chestopen, 0.5 0.984127 -93.5 82.5 -50.5

// if its a blessing, can likely re-use wither essence as these values appear to be the same

[19:37:29] sound note.harp, 1.0 0.7936508 -94.0 82.0 -51.0
[19:37:29] sound note.harp, 1.0 0.8888889 -94.0 82.0 -51.0
[19:37:30] sound note.harp, 1.0 1.0 -94.0 82.0 -51.0
[19:37:30] sound note.harp, 1.0 1.0952381 -94.0 82.0 -51.0
[19:37:30] sound note.harp, 1.0 1.1904762 -94.0 82.0 -51.0
                */
                // player.send_msg("hi").unwrap();
            }
            
            Self::WitherEssence { secret } => {
                let mut secret = secret.borrow_mut();
                debug_assert!(!secret.obtained);

                let world = player.world_mut();

                world.set_block_at(Blocks::Air, block_pos.x, block_pos.y, block_pos.z);
                world.interactable_blocks.remove(block_pos);

                world.spawn_entity(
                    DVec3::from(block_pos).add_x(0.5).add_y(-1.4).add_z(0.5),
                    EntityMetadata {
                        variant: EntityVariant::ArmorStand,
                        is_invisible: true,
                    },
                    NoEntityImpl,
                ).unwrap();

                secret.obtained = true;
            }
            
            Self::Callback(func) => {
                func(player, block_pos);
            }
        }
    }
}