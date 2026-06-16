use crate::dungeon::dungeon_player::{PlayerReadyEvent, ReadyStatus};
use crate::dungeon::DungeonState;
use crate::network::protocol::nbt::{NBTNode, TAG_COMPOUND_ID};
use crate::player::inventory::item_stack::ItemStack;
use crate::player::inventory::menu::{Menu, MenuClick, UpdateMenu};
use crate::player::inventory::{CloseMenu, SyncInventory};
use crate::player::{PlayerSkin, Username, Uuid};
use bevy::app::{App, Plugin};
use bevy::prelude::{ChildOf, Commands, Component, On, OnEnter, Query, With};
use std::collections::HashMap;

pub struct DungeonMenuPlugin;

impl Plugin for DungeonMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(DungeonState::Started { ticks: 0 }), on_dungeon_start)
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
    player_query: Query<(&ReadyStatus, &Username, &Uuid, &PlayerSkin)>,
    mut commands: Commands,
) {
    let Ok((mut menu, child_of)) = menu_query.get_mut(event.menu_entity) else {
        return;
    };

    let (status, username, uuid, skin) = player_query
        .get(child_of.parent())
        .unwrap();

    menu.items.fill(Some(ItemStack::new().item_id(160).metadata(15).name("")));

    let (item_name, color) = if status.0 {
        ("§aReady", 13)
    } else {
        ("§cNot Ready", 14)
    };


    let mut player_head = ItemStack::new()
        .item_id(397)
        .metadata(3)
        .name(&format!("§7{}", username.0));

    player_head.nbt_get_or_insert().nodes.insert("SkullOwner".to_string(), NBTNode::Compound({
        let mut map = HashMap::new();
        map.insert("Id".to_string(), NBTNode::String(uuid.hyphenated().to_string()));
        let vec = vec![NBTNode::Compound(HashMap::from([("Value".to_string(), NBTNode::String(skin.texture.clone()))]))];
        map.insert("textures".to_string(), NBTNode::List { type_id: TAG_COMPOUND_ID, children: vec });
        map
    }));

    menu.items[4] = Some(player_head);
    menu.items[13] = Some(ItemStack::new().item_id(95).metadata(color).name(item_name));
    menu.items[49] = Some(ItemStack::new().item_id(166).name("§cClose"));

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
            commands.trigger(PlayerReadyEvent {
                entity: event.client
            });
            commands.trigger(UpdateMenu {
                menu_entity: event.menu_entity
            });
        }
        49 => {
            commands.trigger(CloseMenu {
                entity: event.client,
            });
        }
        _ => {}
    }
}

fn on_dungeon_start(
    query: Query<&ChildOf, (With<MortMenu>, With<Menu>)>,
    mut commands: Commands,
) {
    for menu in query.iter() {
        commands.trigger(CloseMenu { entity: menu.parent() })
    }
}