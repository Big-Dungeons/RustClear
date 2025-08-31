use crate::dungeon::door::{Door, DoorType};
use crate::dungeon::room::room::Room;
use crate::dungeon::room::room_data::{RoomData, RoomShape, RoomType::*};
use crate::server::block::block_parameter::Axis;
use std::cmp::{max, min};

const RED: u8 = 4 * 4 + 2;
const GREEN: u8 = 7 * 4 + 2;
const GRAY: u8 = 11 * 4 + 3;
const WHITE: u8 = 14 * 4 + 2;
const ORANGE: u8 = 15 * 4 + 2; 
const PURPLE: u8 = 16 * 4 + 2;
const PINK: u8 = 20 * 4 + 2;
const BLACK: u8 = 29 * 4;
const YELLOW: u8 = 30 * 4 + 2;
const BROWN: u8 = 63;

const QUESTION_MARK_POSITIONS: [(usize, usize); 11] = [
    (0, 1), (1, 0), (2, 0), (3, 0), (4, 1), (4, 2), (3, 3), (2, 4), (2, 5), (2, 7), (2, 8),
];

const CHECKMARK_POSITIONS: [(usize, usize); 30] = [
    (7, 0), (8, 0), (6, 1), (7, 1), (8, 1), (5, 2), (6, 2), (7, 2), (4, 3), (5, 3),
    (6, 3), (0, 4), (1, 4), (3, 4), (4, 4), (5, 4), (0, 5), (1, 5), (2, 5), (3, 5),
    (4, 5), (0, 6), (1, 6), (2, 6), (3, 6), (1, 7), (2, 7), (3, 7), (1, 8), (2, 8),
];

pub struct DirtyMapRegion {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

pub struct DungeonMap {
    pub map_data: [u8; 128 * 128],
    offset_x: usize,
    offset_y: usize,
    dirty_region: Option<DirtyMapRegion>
}

// room is 16x16 px
// gap is 4 px
// door is 5x4 px
impl DungeonMap {

    pub fn new(offset_x: usize, offset_y: usize) -> Self {
        Self {
            map_data: [0; 128 * 128],
            offset_x,
            offset_y,
            dirty_region: None,
        }
    }

    pub fn get_updated_area(&mut self) -> Option<(DirtyMapRegion, Vec<u8>)> {
        if let Some(region) = self.dirty_region.take() {
            let width = region.max_x - region.min_x;
            let height = region.max_y - region.min_y;
            let mut pixels = Vec::with_capacity(width * height);

            for row in region.min_y..region.max_y {
                let start = row * 128 + region.min_x;
                let end = row * 128 + region.max_x;

                pixels.extend_from_slice(&self.map_data[start..end]);
            }
            return Some((region, pixels))
        }
        None
    }

    fn set_px(&mut self, x: usize, y: usize, color: u8) {
        let x = x + self.offset_x;
        let y = y + self.offset_y;
        debug_assert!(x < 128);
        debug_assert!(y < 128);

        let region = self.dirty_region.get_or_insert(DirtyMapRegion {
            min_y: 128,
            min_x: 128,
            max_x: 0,
            max_y: 0,
        });
        region.min_x = min(region.min_x, x);
        region.min_y = min(region.min_y, y);
        region.max_x = max(region.max_x, x + 1);
        region.max_y = max(region.max_y, x + 1);

        self.map_data[y * 128 + x] = color
    }

    fn fill_px(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: u8,
    ) {
        let x = x + self.offset_x;
        let y = y + self.offset_y;

        debug_assert!(x + width < 128);
        debug_assert!(y + height < 128);

        let region = self.dirty_region.get_or_insert(DirtyMapRegion {
            min_y: 128,
            min_x: 128,
            max_x: 0,
            max_y: 0,
        });
        region.min_x = min(region.min_x, x);
        region.min_y = min(region.min_y, y);
        region.max_x = max(region.max_x, x + width);
        region.max_y = max(region.max_y, y + height);

        for x in x..x+width {
            for y in y..y+height {
                self.map_data[y * 128 + x] = color
            }
        }
    }
    
    pub fn draw_room(&mut self, rooms: &[Room], doors: &[Door], room_index: usize) {
        let room = &rooms[room_index];
        let color = get_room_color(&room.room_data);
        
        for segment in room.segments.iter() {
            let x = segment.x * 20;
            let y = segment.z * 20;
            
            self.fill_px(x, y, 16, 16, color);
            if room.segments.iter().find(|seg| seg.x == segment.x + 1 && seg.z == segment.z).is_some() {
                self.fill_px(x + 16, y, 4, 16, color);
            }
            if room.segments.iter().find(|seg| seg.x == segment.x && seg.z == segment.z + 1).is_some() {
                self.fill_px(x, y + 16, 16, 4, color);
            }
            
            for (index, neighbour) in segment.neighbours.iter().enumerate() {
                if neighbour.is_none() {
                    continue;
                }
                
                let (neighbour_room, door) = {
                    let neighbour = neighbour.as_ref().unwrap();
                    (&rooms[neighbour.room_index], &doors[neighbour.door_index])
                };
                
                let mut x = segment.x * 20 + 6;
                let mut y = segment.z * 20 + 6;
                
                match index {
                    0 => y -= 10,
                    1 => x += 10,
                    2 => y += 10,
                    3 => x -= 10,
                    _ => unreachable!()
                }
                
                let (width, height) = match door.direction {
                    Axis::X => (4, 5),
                    Axis::Z => (5, 4),
                    _ => unreachable!()
                };
                
                if neighbour_room.entered {
                    let color = get_door_color(room, neighbour_room);
                    self.fill_px(x, y, width, height, color);
                } else {
                    let color = match door.door_type {
                        DoorType::WITHER => BLACK,
                        DoorType::BLOOD => RED,
                        _ => GRAY,
                    };
                    self.fill_px(x, y, width, height, color);

                    let mut x = segment.x * 20;
                    let mut y = segment.z * 20;

                    match index {
                        0 => y -= 20,
                        1 => x += 20,
                        2 => y += 20,
                        3 => x -= 20,
                        _ => unreachable!()
                    }

                    self.fill_px(x ,y, 16, 16, GRAY);

                    for (qx, qy) in QUESTION_MARK_POSITIONS {
                        self.set_px(x + qx + 5, y + qy + 5, BLACK);
                    }
                }
            }
        }
        
        // fill in hole
        if room.room_data.shape == RoomShape::TwoByTwo {
            let x = room.segments[0].x * 20 + 16;
            let y = room.segments[0].z * 20 + 16;
            self.fill_px(x, y, 4, 4, color)
        }
        
        {
            let x = room.segments[0].x * 20 + 4;
            let y = room.segments[0].z * 20 + 4;

            for (cx, cy) in CHECKMARK_POSITIONS {
                self.set_px(x + cx, y + cy, GREEN)
            }
        }
    }
}

fn get_room_color(room_data: &RoomData) -> u8 {
    match room_data.room_type {
        Normal | Rare => BROWN,
        Puzzle => PURPLE,
        Trap => ORANGE,
        Fairy => PINK,
        Entrance => GREEN,
        Blood => RED,
        Yellow =>  YELLOW,
    }
}

fn get_door_color(room: &Room, neighbour: &Room) -> u8 {
    match room.room_data.room_type {
        Puzzle | Trap | Blood | Yellow | Fairy => {
            return get_room_color(&room.room_data)
        }
        _ => {}
    };
    match neighbour.room_data.room_type {
        Puzzle | Trap | Blood | Yellow => {
            return get_room_color(&neighbour.room_data)
        }
        _ => {}
    };
    BROWN
}