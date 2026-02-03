use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::room::room::{Room, RoomStatus};
use glam::IVec3;
use server::{Player, World};

// Once done implementing all rooms,
// and like finalized how I want to do them, make this use enum_dispatch or something
pub trait RoomImplementation {
    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>);
    
    fn tick(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        
    }

    fn interact(&mut self, room: &mut Room, player: &mut Player<DungeonPlayer>, position: IVec3) {

    }
}

pub struct MobRoom;

impl RoomImplementation for MobRoom {
    fn discover(&mut self, room: &mut Room, _: &mut World<Dungeon>) {
        room.status = RoomStatus::Complete
    }
}