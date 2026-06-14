#![allow(unused_imports)]
// enum_dispatch changed or something, because it now needs these imports, but ide can't see
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::etherwarp::AspectOfTheVoid;

use crate::dungeon::items::{get_item_stack, DungeonItem};
use crate::network::packets::{BytesMutExt, PacketEvent};
use crate::network::protocol::play::clientbound::{SetSlot, WindowItems};
use crate::network::protocol::play::serverbound;
use crate::network::protocol::play::serverbound::{ClickMode, ClickWindow, ClientStatus, HeldItemChange};
use crate::player::inventory::item_stack::ItemStack;
use crate::player::inventory::menu::{Menu, MenuClick, UpdateMenu};
use crate::player::{PacketReader, PlayerPacketBuffer};
use bevy::app::{App, Plugin, PreUpdate};
use bevy::prelude::{Commands, Component, Deref, Entity, EntityEvent, IntoScheduleConfigs, On, Query};
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::ops::Deref;

pub mod item_stack;
pub mod item_stack_builder;
pub mod menu;

#[enum_dispatch]
pub trait Item {
    fn item_stack(&self) -> ItemStack;
}

#[derive(Copy, Clone)]
pub enum OpenInventory {
    None,
    Inventory,
    Menu(Entity),
}

#[derive(Component)]
pub struct InventoryState {
    pub window_id: i8,
    pub open_inventory: OpenInventory
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            window_id: 1,
            open_inventory: OpenInventory::None,
        }
    }
}

#[derive(Component)]
pub struct Inventory {
    // boxed for smaller struct size
    pub items: Box<[Option<DungeonItem>; 45]>,
    pub dragged_item: Option<DungeonItem>,

    pub held_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: Box::new([const { None }; 45]),
            dragged_item: None,
            held_slot: 0,
        }
    }
}

impl Inventory {
    pub fn set_slot(&mut self, index: usize, item: Option<DungeonItem>) {
        if index >= 45 {
            return;
        }
        self.items[index] = item;
    }

    pub fn held_item(&self) -> &Option<DungeonItem> {
        let index = self.held_slot + 36;
        if (36..=44).contains(&index) {
            return &self.items[index];
        }
        &None
    }
}

fn on_player_open_inv(
    mut packets: PacketReader<ClientStatus>,
    mut query: Query<&mut InventoryState>,
) {
    for packet in packets.read() {
        if let ClientStatus::OpenInventory = packet.packet {
            let mut state = query
                .get_mut(packet.client())
                .unwrap();

            state.open_inventory = OpenInventory::Inventory
        }
    }
}

fn on_player_close_inv(
    mut packets: PacketReader<serverbound::CloseWindow>,
    mut query: Query<&mut InventoryState>,
) {
    for packet in packets.read() {
        let mut state = query
            .get_mut(packet.client())
            .unwrap();

        if packet.window_id == state.window_id {
            state.open_inventory = OpenInventory::None
        }
    }
}

#[derive(EntityEvent)]
pub struct OpenMenu {
    pub menu_entity: Entity,
    pub entity: Entity,
}

fn open_menu(
    event: On<OpenMenu>,
    menu_query: Query<&Menu>,
    mut player_query: Query<(&mut PlayerPacketBuffer, &mut InventoryState)>,
) {
    let menu = menu_query
        .get(event.menu_entity)
        .expect("used open menu event, but menu entity is invalid");

    let (mut buffer, mut state) = player_query
        .get_mut(event.entity)
        .expect("used open menu event on invalid player");

    state.open_inventory = OpenInventory::Menu(event.menu_entity);
    state.window_id += 1;
    menu.open_menu(state.window_id, &mut buffer);
}

#[derive(EntityEvent, Deref)]
pub struct SyncInventory {
    pub entity: Entity
}

fn sync_player_inventory(
    event: On<SyncInventory>,
    mut player_query: Query<(&Inventory, &InventoryState, &mut PlayerPacketBuffer)>,
    menu_query: Query<&Menu>,
) {
    let (inventory, state, mut buffer) = player_query
        .get_mut(event.entity)
        .unwrap();

    let mut items = Vec::with_capacity(36);
    for item in inventory.items.iter() {
        items.push(get_item_stack(item))
    }
    buffer.write_packet(&WindowItems {
        window_id: 0,
        items: &items,
    });

    match state.open_inventory {
        OpenInventory::Inventory => {
            buffer.write_packet(&SetSlot {
                window_id: -1,
                slot: -1,
                item_stack: get_item_stack(&inventory.dragged_item),
            })
        }
        OpenInventory::Menu(menu_entity) => {
            let menu = menu_query
                .get(menu_entity)
                .expect("open inventory is a menu, but no menu found");

            menu.sync_menu(state.window_id, &mut buffer);
        }
        _ => {}
    }
}

fn on_change_item(
    mut packets: PacketReader<HeldItemChange>,
    mut query: Query<&mut Inventory>
) {
    for PacketEvent { client, packet } in packets.read() {
        let new_slot = packet.slot_id as usize;
        if !(0..=8).contains(&new_slot) {
            continue
        }

        let mut inventory = query.get_mut(*client).unwrap();
        inventory.held_slot = new_slot;
    }
}

// todo: support menus,
//in future validate clicks, allow item splitting if they're in a stack etc
pub(super) fn handle_click_window(
    mut packets: PacketReader<ClickWindow>,
    mut query: Query<(&mut Inventory, &InventoryState, &mut PlayerPacketBuffer)>,
    menu_query: Query<&Menu>,
    mut commands: Commands,
) {
    for packet in packets.read() {
        let (mut inventory, state, mut packet_buffer) = query
            .get_mut(packet.client())
            .unwrap();

        let mut should_resync = true;

        match state.open_inventory {
            OpenInventory::Inventory => {
                should_resync = handle_inventory_click(&mut inventory, &mut packet_buffer, packet);
            }
            OpenInventory::Menu(menu_entity) => {
                let menu = menu_query
                    .get(menu_entity)
                    .expect("open inventory is a menu, but no menu found");

                if packet.slot_id as usize > menu.items.len() {
                    continue;
                }

                commands.trigger(MenuClick {
                    menu_entity,
                    client: packet.client(),
                    slot: packet.slot_id as usize,
                    _click_mode: packet.mode,
                });
            }
            _ => {}
        }

        if should_resync {
            commands.trigger(SyncInventory { entity: packet.client })
        }
    }
}

fn handle_inventory_click(
    inventory: &mut Inventory,
    buffer: &mut BytesMut,
    packet: &ClickWindow,
) -> bool {
    let mut needs_resync = false;

    match packet.mode {
        ClickMode::NormalClick => 'a: {
            // doesn't take into consideration items that can be split
            if packet.slot_id < 0 {
                buffer.write_packet(&SetSlot {
                    window_id: -1,
                    slot: 0,
                    item_stack: get_item_stack(&inventory.dragged_item),
                });
            } else {
                let slot = packet.slot_id as usize;
                if is_valid_range(slot) {
                    // borrow check needs this
                    let Inventory { items, dragged_item, .. } = &mut *inventory;

                    let item = &mut items[slot];
                    let dragged = dragged_item;

                    // only item that shouldn't move
                    if let Some(DungeonItem::SkyblockMenu(_)) = &item {
                        needs_resync = true;
                        break 'a;
                    }

                    if get_item_stack(item) != packet.clicked_item {
                        needs_resync = true;
                    }
                    std::mem::swap(item, dragged);
                }
            }
        }
        ClickMode::ShiftClick => 'a: {
            let slot = packet.slot_id as usize;
            if is_valid_range(slot) {
                // check if it is moveable first
                let item = &mut inventory.items[slot];
                if let Some(DungeonItem::SkyblockMenu(_)) = &item {
                    needs_resync = true;
                    break 'a;
                }

                let item = item.take();
                if get_item_stack(&item) != packet.clicked_item {
                    needs_resync = true;
                }

                let range = if slot >= 36 { 9..36 } else { 36..45 };
                for index in range {
                    if inventory.items[index].is_none() {
                        inventory.items[index] = item;
                        break;
                    }
                }
            }
        }
        ClickMode::NumberKey => 'a: {
            let slot = packet.slot_id as usize;
            let button = packet.used_button as usize;

            if is_valid_range(slot) && button <= 9 {

                let to_slot = 36 + button;
                let item = &mut inventory.items[slot];

                if let Some(DungeonItem::SkyblockMenu(_)) = &item {
                    needs_resync = true;
                    break 'a;
                }

                // this is what hypixel did to allow ghost pickaxes
                if to_slot == slot {
                    buffer.write_packet(&SetSlot {
                        window_id: 0,
                        slot: slot as i16,
                        item_stack: get_item_stack(item),
                    });
                } else {
                    let item = item.take();
                    let item_to = inventory.items[to_slot].take();
                    inventory.items[to_slot] = item;
                    inventory.items[slot] = item_to;
                }
            }
        }
        ClickMode::Drop => {
            needs_resync = true;
        }
        _ => {}
    }

    needs_resync
}

fn is_valid_range(index: usize) -> bool {
    (9..=44).contains(&index)
}

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin  {
    fn build(&self, app: &mut App) {
        app
            .add_observer(sync_player_inventory)
            .add_observer(open_menu)
            .add_observer(menu::on_menu_init)
            .add_systems(PreUpdate, (
                on_change_item, 
                handle_click_window,
                (
                    on_player_open_inv,
                    on_player_close_inv,
                ).chain()
            ));
    }
}