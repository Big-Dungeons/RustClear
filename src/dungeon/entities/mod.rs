use bevy::app::{App, Plugin, Update};

pub mod npc;

pub struct DungeonEntityPlugin;

impl Plugin for DungeonEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, npc::update_npcs);
    }
}