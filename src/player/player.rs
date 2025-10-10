use crate::constants::Sound;
use crate::entity::entity::EntityId;
use crate::get_chunk_position;
use crate::inventory::item::{get_item_stack, Item};
use crate::inventory::item_stack::ItemStack;
use crate::inventory::menu::OpenContainer;
use crate::inventory::Inventory;
use crate::network::binary::var_int::VarInt;
use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::network::protocol::play::clientbound;
use crate::network::protocol::play::clientbound::{ConfirmTransaction, PlayerData, PlayerListItem, SoundEffect, WindowItems};
use crate::network::protocol::play::serverbound::PlayerDiggingAction;
use crate::player::packet_handling::BlockInteractResult;
use crate::types::aabb::AABB;
use crate::world::chunk::chunk_grid::ChunkDiff;
use crate::world::world::VIEW_DISTANCE;
use crate::world::world::{World, WorldExtension};
use glam::{dvec3, DVec3, IVec3, Vec3};
use fstr::FString;
use std::collections::HashMap;
use std::f32::consts::PI;
use uuid::Uuid;

pub type ClientId = usize;

#[derive(Debug, Clone)]
pub struct GameProfileProperty {
    pub value: FString,
    pub signature: Option<FString>
}

#[derive(Debug, Clone)]
pub struct GameProfile {
    pub uuid: Uuid,
    pub username: FString,
    pub properties: HashMap<FString, GameProfileProperty>
}

#[allow(unused_variables)]
pub trait PlayerExtension : Sized {
    type World: WorldExtension<Player = Self>;
    type Item: Item;
    
    fn tick(player: &mut Player<Self>);

    // maybe make a separate enum that actually only has player digging actions
    fn dig(player: &mut Player<Self>, position: IVec3, action: &PlayerDiggingAction) {

    }

    fn interact(player: &mut Player<Self>, item: Option<ItemStack>, block: Option<BlockInteractResult>) {

    }
}

pub struct Player<E : PlayerExtension> {
    world: *mut World<E::World>,

    pub packet_buffer: PacketBuffer,

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
    
    pub is_sneaking: bool,

    pub window_id: i8,
    pub open_container: OpenContainer<E>,
    pub inventory: Inventory<E::Item>,
    pub held_slot: u8,

    // todo: do this for other packets too
    pub sent_block_placement: bool,
    pub ticks_existed: u32,

    pub extension: E
}

impl<E : PlayerExtension> Player<E> {

    pub fn new(
        world: *mut World<E::World>,
        game_profile: GameProfile,
        client_id: ClientId,
        entity_id: EntityId,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        extension: E,
    ) -> Self {
        Self {
            world,

            packet_buffer: PacketBuffer::new(),

            profile: game_profile,
            client_id,
            entity_id,

            position,
            yaw,
            pitch,
            on_ground: false,
            last_position: position,
            last_yaw: yaw,
            last_pitch: pitch,
            
            is_sneaking: false,

            open_container: OpenContainer::None,
            window_id: 0,
            inventory: Inventory::new(),
            held_slot: 0,

            sent_block_placement: false,
            ticks_existed: 0,

            extension,
        }
    }

    pub fn world<'a>(&self) -> &'a World<E::World> {
        unsafe { self.world.as_ref().expect("world is null") }
    }

    pub fn world_mut<'a>(&self) -> &'a mut World<E::World> {
        unsafe { self.world.as_mut().expect("world is null") }
    }

    pub fn write_packet<P : IdentifiedPacket + PacketSerializable>(&mut self, packet: &P) {
        self.packet_buffer.write_packet(packet)
    }

    pub fn flush_packets(&mut self) {
        if !self.packet_buffer.is_empty() {
            let tx = &self.world().network_tx;
            let _ = tx.send(self.packet_buffer.get_packet_message(self.client_id));
        }
    }

    pub fn tick(&mut self) {
        // must be done here instead of instantly after chunks are sent,
        // otherwise entities occasionally appear invisible
        self.spawn_entities();
        self.remove_npc_profiles();

        self.ticks_existed += 1;
        self.write_packet(&ConfirmTransaction {
            window_id: 0,
            action_number: -1,
            accepted: false,
        });

        // tick extension
        E::tick(self);

        // send new and remove chunks (and entities)
        let (chunk_x, chunk_z) = get_chunk_position(self.position);
        let (last_chunk_x, last_chunk_z) = get_chunk_position(self.last_position);
        
        if chunk_x != last_chunk_x || chunk_z != last_chunk_z {

            let world = self.world_mut();
            let chunk_grid = &mut world.chunk_grid;
            
            if let Some(old_chunk) = chunk_grid.get_chunk_mut(last_chunk_x, last_chunk_z) {
                old_chunk.remove_player(self.client_id)
            }
            if let Some(new_chunk) = chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
                new_chunk.insert_player(self.client_id)
            }
            
            // iterate over new/old chunks
            
            self.world().chunk_grid.for_each_diff(
                (chunk_x, chunk_z),
                (last_chunk_x, last_chunk_z),
                VIEW_DISTANCE,
                |x, z, diff| {
                    let Some(chunk) = chunk_grid.get_chunk_mut(x, z) else {
                        return;
                    };
                    if diff == ChunkDiff::New {
                        self.write_packet(&chunk.get_chunk_data(x, z, true));
                        for entity_id in chunk.entities.iter_mut() {
                            if let Some(index) = world.entity_map.get(entity_id) {
                                let entity = &mut world.entities[*index];
                                entity.base.write_spawn_packet(&mut self.packet_buffer);
                                entity.entity_impl.spawn(&mut entity.base, &mut self.packet_buffer);
                            }
                        }
                        return;
                    } else {
                        // could maybe just rely on client to despawn?
                        for entity_id in chunk.entities.iter_mut() {
                            if let Some(index) = world.entity_map.get(entity_id) {
                                let entity = &mut world.entities[*index];
                                entity.base.write_despawn_packet(&mut self.packet_buffer);
                                entity.entity_impl.despawn(&mut entity.base, &mut self.packet_buffer);
                            }
                        }
                    }
                    let chunk_data = chunk_grid.empty_chunk.get_chunk_data(x, z, true);
                    self.write_packet(&chunk_data);
                }
            )
        }
        
        // copy packet buffers from chunks around
        let min_x = chunk_x - VIEW_DISTANCE;
        let min_z = chunk_z - VIEW_DISTANCE;
        let max_x = chunk_x + VIEW_DISTANCE;
        let max_z = chunk_z + VIEW_DISTANCE;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if let Some(chunk) = self.world_mut().chunk_grid.get_chunk(x, z) {
                    self.packet_buffer.copy_from(&chunk.packet_buffer);
                }
            }
        }
        
        self.sent_block_placement = false;
        self.last_position = self.position;
        self.flush_packets();
    }
    
    pub fn play_sound_at(&mut self, sound: Sound, volume: f32, pitch: f32, position: DVec3) {
        self.write_packet(&SoundEffect {
            sound,
            pos_x: position.x,
            pos_y: position.y,
            pos_z: position.z,
            volume,
            pitch,
        })
    }
    
    pub fn play_sound(&mut self, sound: Sound, volume: f32, pitch: f32) {
        self.play_sound_at(sound, volume, pitch, self.position)
    }

    pub fn collision_aabb(&self) -> AABB {
        let w = 0.3;
        let h = 1.8;
        AABB::new(
            dvec3(self.position.x - w, self.position.y, self.position.z - w),
            dvec3(self.position.x + w, self.position.y + h, self.position.z + w),
        )
    }

    pub fn collision_aabb_at(&self, position: &DVec3) -> AABB {
        let w = 0.3;
        let h = 1.8;
        AABB::new(
            dvec3(position.x - w, position.y, position.z - w),
            dvec3(position.x + w, position.y + h, position.z + w),
        )
    }
    
    pub fn open_container(&mut self, mut container: OpenContainer<E>) {
        if let OpenContainer::Menu(_) = self.open_container {
            self.write_packet(&clientbound::CloseWindow {
                window_id: self.window_id,
            })
        }
        self.window_id += 1;
        container.open(self);
        self.open_container = container;
    }
    
    pub fn sync_inventory(&mut self) {
        let mut items = Vec::new();
        for item in self.inventory.items.iter() {
            items.push(get_item_stack(item));
        }
        self.write_packet(&WindowItems {
            window_id: 0,
            items,
        });
        // take ownership
        let mut container = std::mem::replace(&mut self.open_container, OpenContainer::None);
        container.sync_container(self);
        self.open_container = container;
    }
    
    pub fn rotation_vec(&self) -> Vec3 {
        let (yaw_sin, yaw_cos) = (-self.yaw.to_radians() - PI).sin_cos();
        let (pitch_sin, pitch_cos) = (-self.pitch.to_radians()).sin_cos();
        Vec3::new(yaw_sin * -pitch_cos, pitch_sin, yaw_cos * -pitch_cos)
    }
    
    // this function spawns entities in area,
    // this runs on first tick to allow client to load chunks
    #[cold]
    fn spawn_entities(&mut self) {
        if self.ticks_existed == 0 {
            let (chunk_x, chunk_z) = get_chunk_position(self.position);

            let world = self.world_mut();
            world.chunk_grid.for_each_in_view(
                chunk_x,
                chunk_z,
                VIEW_DISTANCE,
                |chunk, _, _| {
                    for entity_id in chunk.entities.iter_mut() {
                        if let Some(index) = world.entity_map.get(entity_id) {
                            let entity = &mut world.entities[*index];
                            entity.base.write_spawn_packet(&mut self.packet_buffer);
                            entity.entity_impl.spawn(&mut entity.base, &mut self.packet_buffer);
                        }
                    }
                },
            );
        }
    }

    // to not appear in tab list, it must be removed
    #[cold]
    fn remove_npc_profiles(&mut self) {
        if self.ticks_existed == 10 {
            let world = self.world_mut();
            let npc_data: Vec<PlayerData> = world.npc_profiles
                .iter()
                .map(|(_, v)| {
                    PlayerData {
                        ping: 0,
                        game_mode: 0,
                        profile: &v,
                        display_name: None,
                    }
                })
                .collect();
            let npc_data_refs: Vec<&PlayerData> = npc_data.iter().collect();

            self.write_packet(&PlayerListItem {
                action: VarInt(4),
                players: npc_data_refs
            });

        }
    }
}

