use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::room::room::Room;
use crate::dungeon::room::room_implementation::RoomImplementation;
use glam::IVec3;
use server::{Player, World};

pub struct QuizPuzzle;

impl RoomImplementation for QuizPuzzle {
    fn discover(&mut self, room: &mut Room, _: &mut World<Dungeon>) {
        // for player in room.players() {
        //     player.write_packet(&Chat {
        //         component: "quiz opened msg or something",
        //         chat_type: 0,
        //     })
        // }
    }
    fn interact(&mut self, _: &mut Room, player: &mut Player<DungeonPlayer>, _: IVec3) {
        // player.write_packet(&Chat {
        //     component: "right clicked block in quiz",
        //     chat_type: 0,
        // })
    }
}