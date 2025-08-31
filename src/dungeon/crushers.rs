use crate::server::block::block_position::BlockPos;
use crate::server::block::blocks::Blocks;
use crate::server::player::player::Player;
use crate::server::utils::direction::Direction;
use crate::server::world::World;
use serde_json::Value;

#[derive(Debug)]
pub struct Crusher {
    pub block_pos: BlockPos,
    pub direction: Direction,
    pub width: i32,
    pub height: i32,
    pub max_length: u16,
    tick_per_block: u16,
    pause_duration: u16,

    current_length: u16,
    tick: u16,

    is_paused: bool,
    is_reversed: bool,
}

impl Crusher {

    pub fn new(
        block_pos: BlockPos,
        direction: Direction,
        width: i32,
        height: i32,
        max_length: u16,
        tick_per_block: u16,
        pause_duration: u16,
    ) -> Crusher {
        Crusher {
            block_pos,
            direction,
            width,
            height,
            max_length,
            tick_per_block,
            pause_duration,
            current_length: 0,
            tick: 0,
            is_paused: false,
            is_reversed: false,
        }
    }

    pub fn from_json(json_entry: &Value) -> Crusher {

        let pos_vec = json_entry["position"].as_array().unwrap().iter().map(|x| {
            x.as_i64().unwrap() as i32
        }).collect::<Vec<i32>>();

        let direction = json_entry["direction"].as_number().unwrap().as_i64().unwrap() as usize;
        let width = json_entry["width"].as_number().unwrap().as_i64().unwrap() as i32;
        let height = json_entry["height"].as_number().unwrap().as_i64().unwrap() as i32;
        let max_length = json_entry["max_length"].as_number().unwrap().as_i64().unwrap() as u16;
        let tick_per_block = json_entry["tick_per_block"].as_number().unwrap().as_i64().unwrap() as u16;
        let pause_duration = json_entry["pause_duration"].as_number().unwrap().as_i64().unwrap() as u16;

        let direction = Direction::from_index(direction);
        let block_pos = BlockPos {
            x: pos_vec[0],
            y: pos_vec[1],
            z: pos_vec[2]
        };

        Crusher::new(block_pos, direction, width, height, max_length, tick_per_block, pause_duration)
    }

    pub fn tick(&mut self, player: &Player/*world: &mut World, network_tx: UnboundedSender<NetworkThreadMessage>*/) {
        let server = player.server_mut();
        self.tick += 1;
        server.world.set_block_at(Blocks::RedstoneBlock, self.block_pos.x, self.block_pos.y - 1, self.block_pos.z);

        let world = &mut server.world;

        if self.is_paused {
            if self.tick == self.pause_duration {
                self.is_reversed = !self.is_reversed;
                self.is_paused = false;
                self.tick = 0;
            }
        } else {
            if self.tick % self.tick_per_block == 0 {
                let (dx, _, dz) = self.direction.get_offset();
                
                if !self.is_reversed {
                    let x = self.block_pos.x + (self.current_length as i32 * dx);
                    let y = self.block_pos.y;
                    let z = self.block_pos.z + (self.current_length as i32 * dz);
                    self.set_blocks(world, Blocks::Stone { variant: 2 }, x, y, z, dx, dz);

                    for (id, player) in &server.world.players {
                        // let entity = player.get_entity_mut(world).unwrap();

                        if self.is_in_way(player, x, y, z) {
                            // PositionLook {
                            //     x: player.position.x + dx as f64,
                            //     y: player.position.y,
                            //     z: player.position.z + dz as f64,
                            //     yaw: player.yaw,
                            //     pitch: player.pitch,
                            //     flags: 0,
                            // }.send_packet(*id, &server.network_tx).unwrap();
                        }
                    }
                } else {
                    let x = self.block_pos.x + ((self.max_length - self.current_length) as i32 * dx);
                    let y = self.block_pos.y;
                    let z = self.block_pos.z + ((self.max_length - self.current_length) as i32 * dz);
                    self.set_blocks(world, Blocks::Air, x, y, z, dx, dz)
                }
                self.current_length += 1;
            }   
            if self.is_reversed && self.current_length == self.max_length || !self.is_reversed && self.current_length > self.max_length {
                if self.pause_duration != 0 {
                    self.is_paused = true
                } else {
                    self.is_reversed = !self.is_reversed
                }
                self.tick = 0;
                self.current_length = 0;
            }
        }
    }

    fn set_blocks(&self, world: &mut World, block: Blocks, x: i32, y: i32, z: i32, dx: i32, dz: i32) {
        for w in 0..self.width {
            for h in 0..self.height {
                world.set_block_at(
                    block,
                    x + (w * dz),
                    y + h,
                    z + (w * dx)
                );
            }
        }
    }

    fn is_in_way(&self, entity: &Player, x: i32, y: i32, z: i32) -> bool {
        let (x_offset, z_offset) = match self.direction {
            Direction::North => (1, 0),
            Direction::East => (0, 0),
            Direction::South => (0, 0),
            Direction::West => (0, 1),
            _ => unreachable!(),
        };
        let (width, length) = match self.direction {
            Direction::North => (-self.width, 1),
            Direction::East => (1, self.width),
            Direction::South => (self.width, 1),
            Direction::West => (1, -self.width),
            _ => unreachable!(),
        };
        // TODO: FINISH
        false
        // entity.is_in_box_i32(x + x_offset, y, z + z_offset, width, self.height, length)
    }
}