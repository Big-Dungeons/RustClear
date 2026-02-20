use crate::constants::{Gamemode, Particle};
use crate::entity::components::EntityAppearance;
use crate::entity::entities::Entities;
use crate::entity::entity::MinecraftEntity;
use crate::network::binary::var_int::VarInt;
use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::network::packets::packet::{IdentifiedPacket, ProcessPacket};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::network::protocol::play::clientbound::{DestroyEntites, JoinGame, Particles, PlayerData, PlayerListItem, PositionLook};
use crate::player::player::{ClientId, GameProfile, Player, PlayerExtension};
use crate::types::status::StatusUpdate;
use crate::world::chunk::chunk_grid::ChunkGrid;
use crate::world::chunk::get_chunk_position;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use enumset::EnumSet;
use glam::{DVec3, Vec3};
use slotmap::SecondaryMap;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

pub const VIEW_DISTANCE: i32 = 6;

pub trait WorldExtension: Sized {
    type Player: PlayerExtension<World = Self>;

    fn tick(world: &mut World<Self>);
    fn on_player_join(world: &mut World<Self>, profile: GameProfile, client_id: ClientId);
    fn on_player_leave(world: &mut World<Self>, player: &mut Player<Self::Player>);
}

pub struct World<W: WorldExtension> {
    pub network_tx: UnboundedSender<NetworkThreadMessage>,
    pub global_packet_buffer: PacketBuffer,

    // maybe go back to only map for players?
    pub players: Vec<Rc<UnsafeCell<Player<W::Player>>>>,
    pub player_map: SecondaryMap<ClientId, usize>,

    pub entities: Entities<W>,
    entities_for_removal: Vec<Entity>,

    pub chunk_grid: ChunkGrid<W>,
    pub extension: W,
}

impl<W: WorldExtension + 'static> World<W> {

    pub fn new(network_tx: UnboundedSender<NetworkThreadMessage>, extension: W) -> Self {
        Self {
            network_tx,
            global_packet_buffer: PacketBuffer::new(),
            players: Vec::new(),
            player_map: SecondaryMap::new(),
            entities: Entities::new(),
            entities_for_removal: Vec::new(),
            chunk_grid: ChunkGrid::new(16, 13, 13),
            extension,
        }
    }

    pub fn spawn_player(
        &mut self,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        profile: GameProfile,
        client_id: ClientId,
        gamemode: Gamemode,
        extension: W::Player,
    ) -> &mut Player<W::Player> {

        let entity_id = self.entities.next_entity_id();
        let player_rc = Rc::new(UnsafeCell::new(Player::new(
            self,
            profile,
            client_id,
            entity_id,
            position,
            yaw,
            pitch,
            gamemode,
            extension
        )));
        let player = unsafe { &mut *player_rc.get() };

        player.write_packet(&JoinGame {
            entity_id: player.entity_id,
            gamemode,
            dimension: 0,
            difficulty: 0,
            max_players: 0,
            level_type: "",
            reduced_debug_info: false,
        });

        let (chunk_x, chunk_z) = get_chunk_position(player.position);
        if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.insert_player(client_id, player_rc.clone())
        }

        self.chunk_grid
            .for_each_in_view(chunk_x, chunk_z, VIEW_DISTANCE + 1, |chunk, x, z| {
                chunk.write_chunk_data(x, z, true, &mut player.packet_buffer);
            });

        player.write_packet(&PositionLook {
            x: player.position.x,
            y: player.position.y,
            z: player.position.z,
            yaw: player.yaw,
            pitch: player.pitch,
            flags: EnumSet::new(),
        });

        player.write_packet(&PlayerListItem {
            action: VarInt(0),
            players: &[PlayerData {
                ping: 0,
                game_mode: 0,
                profile: &player.profile.clone(),
                display_name: None,
            }],
        });

        self.chunk_grid
            .for_each_in_view(chunk_x, chunk_z, VIEW_DISTANCE, |chunk, _, _| {
                chunk.write_spawn_entities(player);
            });

        player.flush_packets();

        let index = self.players.len();
        self.players.push(player_rc);
        self.player_map.insert(client_id, index);
        // should always be present
        unsafe { self.players[index].get().as_mut().unwrap() }
    }

    pub fn spawn_entity<T : EntityAppearance<W>, B : Bundle>(
        &mut self,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        appearance: T,
        components: B,
    ) -> Entity {
        let entity_base = MinecraftEntity::new::<T>(
            self,
            position,
            yaw,
            pitch
        );
        
        // kinda scuffed ngl
        self.entities.register_appearance_update::<T>();
        
        let entity_id = self.entities.spawn((components, entity_base, appearance));
        let entity_ref = self.entities.get_entity(entity_id);

        let (chunk_x, chunk_z) = get_chunk_position(position);

        if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.insert_entity(entity_id);
    
            self.chunk_grid
                .for_each_in_view(chunk_x, chunk_z, VIEW_DISTANCE, |chunk, _, _| {
                    if let Some(mc_entity) = entity_ref.get::<MinecraftEntity<W>>() {
                        for player in chunk.players() {
                            (mc_entity.enter_view)(mc_entity, &entity_ref, player)
                        }
                    }
                });
        }

        entity_id
    }

    pub fn write_global_packet<P : IdentifiedPacket + PacketSerializable>(&mut self, packet: &P) {
        self.global_packet_buffer.write_packet(packet)
    }

    pub fn spawn_particle(&mut self, particle: Particle, position: Vec3, offset: Vec3, count: i32) {
        let chunk_x = (position.x.floor() as i32) >> 4;
        let chunk_z = (position.z.floor() as i32) >> 4;

        if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.packet_buffer.write_packet(&Particles {
                particle,
                long_distance: false,
                position,
                offset,
                speed: 0.0,
                count,
            })
        }
    }

    pub fn remove_player(&mut self, client_id: ClientId) {
        // order isn't preserved doing this.
        // however with old implementation, it did use std hashmap to iterate,
        // and order wasn't preserved there so it should be fine.
        if let Some(index) = self.player_map.remove(client_id) {
            let last_index = self.players.len() - 1;

            let player_rc = self.players.swap_remove(index);
            let player = unsafe { player_rc.get().as_mut().unwrap() };
            W::on_player_leave(self, player);

            if last_index != index {
                let moved_id = unsafe { self.players[index].get().as_ref().unwrap() }.client_id;
                self.player_map.insert(moved_id, index);
            }

            let (chunk_x, chunk_z) = get_chunk_position(player.position);

            if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
                chunk.remove_player(client_id)
            }

            assert_eq!(Rc::strong_count(&player_rc), 1, "player leaked")
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.entities_for_removal.push(entity);
    }

    pub fn tick(&mut self) {
        // tick extension
        W::tick(self);

        let mut packet_destroy_entities = DestroyEntites {
            entities: vec![]
        };

        for id in self.entities_for_removal.drain(..) {
            {
                let entity = self.entities.get_entity(id);
                if let Some(mc_entity) = entity.get::<MinecraftEntity<W>>() {
                    (mc_entity.destroy)(mc_entity, &entity, &mut packet_destroy_entities);

                    let (chunk_x, chunk_z) = get_chunk_position(mc_entity.position);
                    if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
                        chunk.remove_entity(id)
                    }
                }
            }
            self.entities.despawn(id)
        }
        self.entities.tick();

        for player in self.players_mut() {
            player.write_packet(&packet_destroy_entities);
            player.tick();
        }
        for chunk in self.chunk_grid.chunks.iter_mut() {
            chunk.packet_buffer.clear()
        }
        self.global_packet_buffer.clear()
    }

    pub fn add_player_to_chunk(&mut self, client_id: ClientId, chunk_x: i32, chunk_z: i32) {
        if let Some(index) = self.player_map.get(client_id) {
            let player = self.players[*index].clone();

            if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
                chunk.insert_player(client_id, player)
            }
        }
    }

    pub fn process_event(&mut self, event: MainThreadMessage) {
        match event {
            MainThreadMessage::NewPlayer { client_id, profile } => {
                W::on_player_join(self, profile, client_id);
                let _ = self.network_tx.send(NetworkThreadMessage::UpdateStatus(
                    StatusUpdate::Players(self.players.len() as u32),
                ));
            }
            MainThreadMessage::PacketReceived { client_id, packet } => {
                if let Some(index) = self.player_map.get(client_id) {
                    let player_rc = &mut self.players[*index];
                    let player = unsafe { &mut *player_rc.get() };
                    packet.process(player)
                }
            }
            MainThreadMessage::ClientDisconnected { client_id } => {
                self.remove_player(client_id);
                let _ = self.network_tx.send(NetworkThreadMessage::UpdateStatus(
                    StatusUpdate::Players(self.players.len() as u32),
                ));
            }
        }
    }

    pub fn players(&self) -> impl Iterator<Item = &Player<W::Player>> {
        self.players.iter().map(|it| unsafe { &*it.get() })
    }
    
    pub fn players_mut(&mut self) -> impl Iterator<Item = &mut Player<W::Player>> {
        self.players.iter().map(|it| unsafe { &mut *it.get() })
    }
}

impl<W: WorldExtension> Deref for World<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.extension
    }
}

impl<W: WorldExtension> DerefMut for World<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.extension
    }
}
