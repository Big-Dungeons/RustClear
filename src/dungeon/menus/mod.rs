use bevy::app::{App, Plugin};
use bevy::prelude::{Add, ChildOf, Commands, Component, On, Query, With};
use crate::dungeon::dungeon_player::{PlayerReadyEvent, ReadyStatus};
use crate::player::inventory::item_stack::ItemStack;
use crate::player::inventory::menu::{Menu, MenuClick, UpdateMenu};
use crate::player::inventory::SyncInventory;

pub struct DungeonMenuPlugin;

impl Plugin for DungeonMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_menu_update)
            .add_observer(on_click);
    }
}

// marker
#[derive(Component)]
pub struct MortMenu;

pub fn on_menu_update(
    event: On<UpdateMenu>, // shame you can't filter here
    mut menu_query: Query<(&mut Menu, &ChildOf), With<MortMenu>>,
    player_query: Query<&ReadyStatus>,
    mut commands: Commands,
) {
    let Ok((mut menu, child_of)) = menu_query.get_mut(event.menu_entity) else {
        return;
    };

    let status = player_query
        .get(child_of.parent())
        .unwrap();

    menu.items.fill(Some(ItemStack::new().item_id(160).metadata(15).name("")));

    let (item_name, color) = if status.0 {
        ("§aReady", 13)
    } else {
        ("§cNot Ready", 14)
    };

    menu.items[13] = Some(ItemStack::new().item_id(95).metadata(color).name(item_name));
    commands.trigger(SyncInventory { entity: child_of.parent() });
}

pub fn on_click(
    event: On<MenuClick>,
    menu_query: Query<(), With<MortMenu>>,
    mut commands: Commands
) {
    if !menu_query.contains(event.menu_entity) {
        return;
    }

    match event.slot {
        4 | 13 => {
            commands.trigger(PlayerReadyEvent { entity: event.client });
            commands.trigger(UpdateMenu { menu_entity: event.menu_entity });
        }
        _ => {}
    }
}