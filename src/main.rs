use crate::chunk::ChunkPlugin;
use crate::debug_world::DebugWorld;
use crate::dungeon::DungeonPlugin;
use crate::entity::MobPlugin;
use crate::network::packets::PacketEvent;
use crate::network::protocol::play::serverbound::ChatMessage;
use crate::network::NetworkPlugin;
use crate::player::inventory::SyncInventory;
use crate::player::PlayerPlugin;
use bevy::app::{App, ScheduleRunnerPlugin, Update};
use bevy::prelude::{Commands, MessageReader};
use glam::IVec2;
use std::time::Duration;

mod block;
mod chunk;
mod debug_world;
mod dungeon;
mod entity;
mod network;
mod player;
mod types;

// temp
const TEST_WORLD: bool = false;

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_millis(50)),
            NetworkPlugin {
                addr: "127.0.0.1:8080",
            },
            ChunkPlugin {
                size: 16,
                offset: IVec2::splat(13),
            },
            MobPlugin,
            PlayerPlugin,
            DungeonPlugin,
            DebugWorld,
        ))
        .add_systems(Update, chat_test)
        .run();
}

fn chat_test(mut chat_messages: MessageReader<PacketEvent<ChatMessage>>, mut commands: Commands) {
    for PacketEvent { packet, client } in chat_messages.read() {
        println!("chat message: {}", packet.string);
        commands.trigger(SyncInventory { entity: *client })
    }
}