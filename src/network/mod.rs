use crate::entity::{EntityExt, OldTransform, Transform};
use crate::network::client::{run_client, ClientId, ConnectionState};
use crate::network::packets::BytesMutExt;
use crate::network::protocol::play::clientbound::JoinGame;
use crate::network::protocol::play::serverbound::{register_play_packet, PlayPacketWriters};
use crate::player::inventory::Inventory;
use crate::player::known_state::KnownState;
use crate::player::use_item::SentInteract;
use crate::player::{Player, PlayerJoinEvent, PlayerPacketBuffer, Username};
use bevy::app::Last;
use bevy::prelude::{App, Commands, Component, Deref, DerefMut, Entity, First, MessageWriter, Plugin, Query, ResMut, Resource, With};
use bytes::{Bytes, BytesMut};
use slotmap::{SecondaryMap, SlotMap};
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub mod packets;
pub mod protocol;
pub mod client;

pub enum NetworkMessage {
    SendPackets {
        client_id: ClientId,
        buffer: Bytes,
    },
    ConnectionClosed {
        client_id: ClientId,
        connection_state: ConnectionState,
    },
}

pub enum MainMessage {
    ClientDisconnected {
        client_id: ClientId
    },
    AddPlayer {
        client_id: ClientId,
        username: String,
    },
    PacketReceived {
        client_id: ClientId,
        buffer: Bytes,
    }
}

type USender<T> = UnboundedSender<T>;
type UReceiver<T> = UnboundedReceiver<T>;

#[derive(Resource, Deref, DerefMut)]
pub struct EntityLookup(SecondaryMap<ClientId, Entity>);


#[derive(Resource, Deref, DerefMut)]
pub struct NetworkSender(pub USender<NetworkMessage>);

#[derive(Resource, Deref)]
pub struct NetworkReceiver(pub UReceiver<MainMessage>);

pub struct NetworkPlugin {
    pub addr: &'static str,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let start = Instant::now();
        let (main_tx, main_rx) = unbounded_channel::<MainMessage>();
        let (network_tx, mut network_rx) = unbounded_channel::<NetworkMessage>();

        let addr = self.addr.to_string();
        
        app.insert_resource(NetworkSender(network_tx.clone()));
        app.insert_resource(NetworkReceiver(main_rx));
        app.insert_resource(EntityLookup(SecondaryMap::new()));

        app.add_message::<PlayerJoinEvent>();
        app.add_systems(First, recv_network_messages);
        // deferred, to not crash when unwrapping packet events for example
        app.add_systems(Last, despawn_clients);


        register_play_packet(app);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let listener = TcpListener::bind(addr).await.unwrap();

                let mut clients: SlotMap<ClientId, USender<Bytes>> = SlotMap::with_key();

                loop {
                    tokio::select! {
                        result = listener.accept() => {
                            let Ok((socket, _)) = result else { continue };
    
                            clients.insert_with_key(|key| {
                                let (tx, rx) = unbounded_channel::<Bytes>();
                                println!("client connected");
                                tokio::spawn(run_client(key, socket, rx, network_tx.clone(), main_tx.clone()));
                                tx
                            });
                        }
    
                        Some(message) = network_rx.recv() => {
                            match message {
                                NetworkMessage::ConnectionClosed { client_id, connection_state } => {
                                    if clients.remove(client_id).is_some() && connection_state == ConnectionState::Play {
                                        let _ = main_tx.send(MainMessage::ClientDisconnected { client_id });
                                    }
                                }
                                NetworkMessage::SendPackets { client_id, buffer } => {
                                    if let Some(tx) = clients.get(client_id) && let Err(e) = tx.send(buffer) {
                                        eprintln!("client {client_id:?} dropped it's receiver {e}");
                                        clients.remove(client_id);
                                        let _ = main_tx.send(MainMessage::ClientDisconnected { client_id });
                                    }
                                }
                            }
                        }
                    }
                }
            })
        });

        println!("time elapsed {:?}", start.elapsed())
    }
}

fn recv_network_messages(
    mut rx: ResMut<NetworkReceiver>,
    mut commands: Commands,
    mut lookup: ResMut<EntityLookup>,
    mut events: MessageWriter<'_, PlayerJoinEvent>,
    mut packet_writers: PlayPacketWriters,
) {
    loop {
        match rx.0.try_recv() {
            Ok(message) => {
                match message {
                    MainMessage::ClientDisconnected { client_id } => {
                        if let Some(player) = lookup.remove(client_id) {
                            // packets
                            commands.entity(player).insert(PendingDespawn);
                        }
                    }
                    MainMessage::AddPlayer { client_id, username } => {
                        let mut entity = commands.spawn_empty();
                        let id = entity.id();

                        // must run first packet, so do it now
                        let mut packet_buffer = PlayerPacketBuffer(BytesMut::new());
                        packet_buffer.write_packet(&JoinGame {
                            entity_id: id.mc_id(),
                            gamemode: 0,
                            dimension: 0,
                            difficulty: 0,
                            max_players: 0,
                            level_type: "",
                            reduced_debug_info: false,
                        });

                        entity.insert((
                            Player,
                            client_id,
                            packet_buffer,
                            Username(username),
                            Transform::default(),
                            OldTransform(Transform::default()),
                            Inventory::default(),
                            KnownState::default(),
                            SentInteract(false),
                        ));

                        lookup.insert(client_id, id);
                        events.write(PlayerJoinEvent(id));
                    }
                    MainMessage::PacketReceived { client_id, mut buffer } => {
                        let Some(entity) = lookup.get(client_id) else {
                            continue;
                        };
                        if let Err(e) = packet_writers.dispatch(*entity, &mut buffer) {
                            eprintln!("err parsing: {e}")
                        };
                    }
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("network thread dropped its receiver")
        }
    }
}

#[derive(Component)]
struct PendingDespawn;

fn despawn_clients(
    query: Query<Entity, With<PendingDespawn>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}