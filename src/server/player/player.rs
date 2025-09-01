use crate::net::internal_packets::NetworkThreadMessage;
use crate::net::packets::packet::IdentifiedPacket;
use crate::net::packets::packet_buffer::PacketBuffer;
use crate::net::packets::packet_serialize::PacketSerializable;
use crate::net::protocol::play::clientbound::{Chat, OpenWindow, SetSlot, WindowItems};
use crate::server::entity::entity::EntityId;
use crate::server::player::container_ui::UI;
use crate::server::player::inventory::{Inventory, ItemSlot};
use crate::server::player::scoreboard::Scoreboard;
use crate::server::server::Server;
use crate::server::utils::aabb::AABB;
use crate::server::utils::chat_component::chat_component_text::ChatComponentTextBuilder;
use crate::server::world::World;
use glam::DVec3;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

/// type alias to represent a client's user id.
///
/// alias for a u32
pub type ClientId = u32;

// add uuid
#[derive(Debug, Clone)]
pub struct GameProfileProperty {
    pub value: String,
    pub signature: Option<String>
}

#[derive(Debug, Clone)]
pub struct GameProfile {
    pub uuid: Uuid,
    pub username: String,
    pub properties: HashMap<String, GameProfileProperty>
}

#[derive(Debug)]
pub struct Player {
    pub server: *mut Server,
    pub packet_buffer: PacketBuffer,
    pub network_tx: UnboundedSender<NetworkThreadMessage>,
    
    pub profile: GameProfile,
    pub client_id: ClientId,
    pub entity_id: EntityId,
    
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    
    pub last_position: DVec3,   
    pub last_yaw: f32,
    pub last_pitch: f32,

    pub ticks_existed: u32,
    pub last_keep_alive: i32,
    pub ping: i32,

    pub is_sneaking: bool,

    pub inventory: Inventory,
    pub held_slot: u8,
    
    pub window_id: i8,
    pub current_ui: UI,
    // pub current_ui: UI,

    pub sidebar: Scoreboard,
}

impl Player {
    
    pub fn new(
        server: &mut Server,
        client_id: ClientId,
        profile: GameProfile,
        position: DVec3,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self {
            server,
            packet_buffer: PacketBuffer::new(),
            network_tx: server.network_tx.clone(),
            profile,
            client_id,
            entity_id: server.world.new_entity_id(),

            position,
            on_ground: false,
            yaw,
            pitch,
            last_position: DVec3::ZERO,
            last_yaw: 0.0,
            last_pitch: 0.0,

            ticks_existed: 0,
            last_keep_alive: -1,
            ping: -1,
            is_sneaking: false,

            inventory: Inventory::empty(),
            held_slot: 0,
            
            window_id: 1,
            current_ui: UI::None,
            
            sidebar: Scoreboard::new(),
            
            // observed_entities: HashSet::new(),
        }
    }

    /// gets a reference to the server
    pub fn server_mut<'a>(&self) -> &'a mut Server {
        unsafe { self.server.as_mut().expect("Server is null") }
    }

    /// gets a reference to the world
    pub fn world_mut<'a>(&self) -> &'a mut World {
        &mut self.server_mut().world
    }
    
    pub fn write_packet<P : IdentifiedPacket + PacketSerializable>(&mut self, packet: &P) {
        self.packet_buffer.write_packet(packet);
    }
    
    pub fn flush_packets(&mut self) {
        if !self.packet_buffer.buffer.is_empty() {
            let _ = self.network_tx.send(self.packet_buffer.get_packet_message(&self.client_id));
        }
    }
    
    // todo: tick function here?, 
    // pub fn tick(&mut self) -> anyhow::Result<()> {
    //     self.ticks_existed += 1;
    //     self.send_packet(ConfirmTransaction::new())?;
    //     Ok(())
    // }
    
    /// updates player position
    pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
        // self.last_position = self.position;
        self.position = DVec3::new(x, y, z);
    }
    
    pub fn collision_aabb(&self) -> AABB {
        let w = 0.3;
        let h = 1.8;
        AABB::new(
            DVec3::new(self.position.x - w, self.position.y, self.position.z - w),
            DVec3::new(self.position.x + w, self.position.y + h, self.position.z + w),
        )
    }

    pub fn handle_left_click(&mut self) {
        
    }
    
    pub fn handle_right_click(&mut self) {
        if let Some(ItemSlot::Filled(item)) = self.inventory.get_hotbar_slot(self.held_slot as usize) {
            item.on_right_click(self).unwrap()
        }
    }
    
    pub fn open_ui(&mut self, ui: UI) {
        self.current_ui = ui;
        // kind of temporary solution,
        // instead of just putting the item in an available slot if it is dragged
        if ui == UI::Inventory { 
            if let ItemSlot::Filled(item) = self.inventory.dragged_item { 
                self.write_packet(&SetSlot {
                    window_id: -1,
                    slot: 0,
                    item_stack: Some(item.get_item_stack()),
                })
            }
        }
        if let Some(container_data) = ui.get_container_data() {
            self.window_id += 1;
            self.write_packet(&OpenWindow {
                window_id: self.window_id,
                inventory_type: "minecraft:container".into(),
                window_title: ChatComponentTextBuilder::new(container_data.title).build(),
                slot_count: container_data.slot_amount,
            });
            self.sync_inventory();
        }
    }
    
    pub fn sync_inventory(&mut self) {
        let mut inv_items = Vec::new();
        
        for item in &self.inventory.items {
            inv_items.push(item.get_item_stack())
        }
        
        if let Some(items) = self.current_ui.get_container_contents(self.server_mut(), &self.client_id)  {
            self.write_packet(&WindowItems {
                window_id: self.window_id,
                items,
            });
        }
        
        self.write_packet(&WindowItems {
            window_id: 0,
            items: inv_items,
        });
        if let UI::Inventory = self.current_ui {
            self.write_packet(&SetSlot {
                window_id: -1,
                slot: 0,
                item_stack: self.inventory.dragged_item.get_item_stack(),
            })
        } else {
            self.write_packet(&SetSlot {
                window_id: -1,
                slot: 0,
                item_stack: None,
            })
        }
    }
    
    pub fn send_message(&mut self, msg: &str) {
        self.write_packet(&Chat {
            component: ChatComponentTextBuilder::new(msg).build(),
            chat_type: 0 
        })
    }
}