use crate::network::packets::packet_deserializable::PacketDeserializable;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::handshake::serverbound::Handshake;
use crate::network::protocol::login::clientbound::LoginSuccess;
use crate::network::protocol::login::serverbound::LoginStart;
use crate::network::protocol::status::clientbound::{StatusPong, StatusResponse};
use crate::network::protocol::status::serverbound::StatusPing;
use crate::network::protocol::var_int::{peek_var_int, VarInt};
use crate::network::{MainMessage, NetworkMessage, UReceiver, USender};
use anyhow::bail;
use bevy::prelude::Component;
use bytes::{Buf, Bytes, BytesMut};
use slotmap::new_key_type;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

new_key_type! {
    #[derive(Component)]
    pub struct ClientId;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ConnectionState {
    Handshaking,
    Play,
    Status,
    Login,
}

impl ConnectionState {
    pub fn from_id(id: i32) -> anyhow::Result<ConnectionState> {
        Ok(match id {
            -1 => ConnectionState::Handshaking,
            0 => ConnectionState::Play,
            1 => ConnectionState::Status,
            2 => ConnectionState::Login,
            _ => bail!("Invalid connection state id: {}", id),
        })
    }
}

pub(super) async fn run_client(
    client_id: ClientId,
    mut socket: TcpStream,
    mut write_rx: UReceiver<Bytes>,
    network_tx: USender<NetworkMessage>,
    main_tx: USender<MainMessage>
) {
    let mut connection_state = ConnectionState::Handshaking;
    let mut bytes = BytesMut::new();

    loop {
        tokio::select! {
            result = socket.read_buf(&mut bytes) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                          if let Err(err) = read_packets(
                            &mut bytes,
                            client_id,
                            &mut connection_state,
                            &network_tx,
                            &main_tx,
                            // &status
                        ).await {
                            eprintln!("client {client_id:?} errored reading packet: {err}");
                            break;
                        }
                    },
                    Err(err) => {
                        eprintln!("Client {client_id:?} errored reading socket: {err}");
                        break;
                    }
                }
            }
            option = write_rx.recv() => {
                match option {
                    Some(bytes) => {
                        if let Err(err) = socket.write_all(&bytes).await {
                            eprintln!("Socket write error: {err}");
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    println!("client connection closed");
    let _ = network_tx.send(NetworkMessage::ConnectionClosed {
        client_id,
        connection_state
    });
}

async fn read_packets(
    buffer: &mut BytesMut,
    client_id: ClientId,
    connection_state: &mut ConnectionState,
    network_tx: &UnboundedSender<NetworkMessage>,
    main_tx: &UnboundedSender<MainMessage>,
) -> anyhow::Result<()> {
    while let Some(mut buffer) = try_read_packet_slice(buffer) {
        match connection_state {
            ConnectionState::Handshaking => handle_handshake(&mut buffer, connection_state)?,
            ConnectionState::Status => handle_status(client_id, &mut buffer, network_tx)?,
            ConnectionState::Login => handle_login(client_id, connection_state, &mut buffer, network_tx, main_tx)?,
            ConnectionState::Play => {
                main_tx.send(MainMessage::PacketReceived {
                    client_id,
                    buffer: buffer.copy_to_bytes(buffer.remaining()),
                })?;
            }
        }
    }
    Ok(())
}

fn try_read_packet_slice(buf: &mut BytesMut) -> Option<Bytes> {
    if !buf.has_remaining() {
        return None;
    }

    let (packet_len, var_int_len) = peek_var_int(buf)?;
    let packet_len = (*packet_len) as usize;

    if buf.remaining() < packet_len + var_int_len {
        // packet incomplete
        return None;
    }

    buf.advance(var_int_len);
    Some(buf.copy_to_bytes(packet_len))
}

fn handle_handshake(buffer: &mut Bytes, connection_state: &mut ConnectionState,) -> anyhow::Result<()> {
    match *VarInt::read(buffer)? {
        0x00 => {
            let handshake = Handshake::read(buffer)?;
            *connection_state = ConnectionState::from_id(handshake.next_state.0)?;
        }
        _ => bail!("Unknown pack id during handshake."),
    }
    Ok(())
}

fn handle_status(
    client_id: ClientId,
    buffer: &mut Bytes,
    network_tx: &USender<NetworkMessage>,
) -> anyhow::Result<()> {
    let packet_id = *VarInt::read(buffer)?;
    let mut packet_buffer = BytesMut::new();
    match packet_id {
        0x00 => {
            packet_buffer.write_packet(&StatusResponse {
                // zero reason for it to be mutable
                status: r#"{"version":{"name":"1.8.9","protocol":47},"players":{"max":1,"online":0},"description":"Dungeon Recreation"}"#
            });
        }
        0x01 => {
            let status_ping = StatusPing::read(buffer)?;
            packet_buffer.write_packet(&StatusPong {
                client_time: status_ping.client_time,
            });
        }
        _ => bail!("Unknown packet id during status")
    }

    network_tx.send(packet_buffer.get_packet_message(client_id))?;
    Ok(())
}

// await retrieve skin before accepting login
fn handle_login(
    client_id: ClientId,
    connection_state: &mut ConnectionState,
    buffer: &mut Bytes,
    network_tx: &USender<NetworkMessage>,
    main_tx: &USender<MainMessage>
) -> anyhow::Result<()> {
    let packet_id = *VarInt::read(buffer)?;
    match packet_id {
        0x00 => {
            let LoginStart { username } = LoginStart::read(buffer)?;
            let username = username.to_string();
            let uuid = Uuid::new_v4();

            *connection_state = ConnectionState::Play;

            let mut packet_buffer = BytesMut::new();
            packet_buffer.write_packet(&LoginSuccess {
                uuid: uuid.hyphenated().to_string(),
                name: username.clone(),
            });
            
            network_tx.send(packet_buffer.get_packet_message(client_id))?;
            main_tx.send(MainMessage::AddPlayer {
                client_id,
                username,
            })?;
        }
        _ => bail!("Unknown packet id during login")
    }

    Ok(())
}