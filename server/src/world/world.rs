use crate::constants::{Gamemode, Particle};
use crate::entity::entity::{Entity, EntityExtension, EntityId};
use crate::entity::entity_appearance::EntityAppearance;
use crate::network::binary::var_int::VarInt;
use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::network::packets::packet::ProcessPacket;
use crate::network::protocol::play::clientbound::{DestroyEntites, JoinGame, Particles, PlayerData, PlayerListItem, PositionLook};
use crate::player::player::{ClientId, GameProfile, Player, PlayerExtension};
use crate::types::status::StatusUpdate;
use crate::world::chunk::chunk_grid::ChunkGrid;
use crate::world::chunk::get_chunk_position;
use enumset::EnumSet;
use glam::{DVec3, Vec3};
use slotmap::SecondaryMap;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
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

    pub players: Vec<Player<W::Player>>,
    pub player_map: SecondaryMap<ClientId, usize>,

    entity_id: i32,
    pub entities: Vec<Entity<W>>,
    pub entity_map: HashMap<EntityId, usize>,
    entities_for_removal: Vec<EntityId>,

    pub chunk_grid: ChunkGrid,

    pub extension: W,
}

impl<W: WorldExtension> World<W> {

    pub fn new(network_tx: UnboundedSender<NetworkThreadMessage>, extension: W) -> Self {
        Self {
            network_tx,
            players: Vec::new(),
            player_map: SecondaryMap::new(),
            entity_id: 0,
            entities: Vec::new(),
            entity_map: HashMap::new(),
            entities_for_removal: Vec::new(),
            chunk_grid: ChunkGrid::new(16, 13, 13),
            extension,
        }
    }

    pub fn new_entity_id(&mut self) -> i32 {
        self.entity_id += 1;
        self.entity_id
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
        let entity_id = self.new_entity_id();

        let mut player = Player::new(
            self,
            profile,
            client_id,
            entity_id,
            position,
            yaw,
            pitch,
            gamemode,
            extension
        );

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
            chunk.insert_player(client_id)
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
                chunk.write_spawn_entities(&mut player);
            });

        player.flush_packets();

        let index = self.players.len();
        self.players.push(player);
        self.player_map.insert(client_id, index);
        &mut self.players[index]
    }

    pub fn spawn_entity<A: EntityAppearance<W> + 'static, E: EntityExtension<W> + 'static>(
        &mut self,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        appearance: A,
        extension: E,
    ) -> &mut Entity<W> {
        let entity_id = self.new_entity_id();
        let mut entity = Entity::new(self, entity_id, position, yaw, pitch, appearance, extension);

        let (chunk_x, chunk_z) = get_chunk_position(position);

        if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.insert_entity(entity_id);

            self.chunk_grid
                .for_each_in_view(chunk_x, chunk_z, VIEW_DISTANCE, |chunk, _, _| {
                    if chunk.has_players() {
                        for player in self.players.iter_mut().filter(|p| chunk.players.contains(&p.client_id)) {
                            entity.enter_view(player)
                        }
                    }
                });
        }

        let index = self.entities.len();
        self.entities.push(entity);
        self.entity_map.insert(entity_id, index);
        &mut self.entities[index]
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

            let mut player = self.players.swap_remove(index);
            W::on_player_leave(self, &mut player);

            if last_index != index {
                let moved_id = self.players[index].client_id;
                self.player_map.insert(moved_id, index);
            }

            let (chunk_x, chunk_z) = get_chunk_position(player.position);

            if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
                chunk.remove_player(client_id)
            }
        }
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        self.entities_for_removal.push(entity_id);
    }

    pub fn tick(&mut self) {
        // tick extension
        W::tick(self);

        let mut packet_destroy_entities = DestroyEntites { entities: vec![] };
        for entity_id in self.entities_for_removal.drain(..) {
            if let Some(index) = self.entity_map.remove(&entity_id) {
                let last_index = self.entities.len() - 1;
                let mut entity = self.entities.swap_remove(index);

                
                entity.destroy(&mut packet_destroy_entities);

                if last_index != index {
                    let moved_id = self.entities[index].base.id;
                    self.entity_map.insert(moved_id, index);
                }
            }
        }

        for entity in self.entities.iter_mut() {
            entity.tick()
        }

        for player in self.players.iter_mut() {
            player.write_packet(&packet_destroy_entities);
            player.tick();
        }

        for chunk in self.chunk_grid.chunks.iter_mut() {
            chunk.packet_buffer.clear()
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
                    let player = &mut self.players[*index];
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
