use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::handshake::serverbound::Handshake;
use crate::core::network::protocol::login::clientbound::LoginSuccess;
use crate::core::network::protocol::login::serverbound::LoginStart;
use crate::core::network::protocol::status::clientbound::{StatusPong, StatusResponse};
use crate::core::network::protocol::status::serverbound::StatusPing;
use crate::core::network::protocol::var_int::{peek_var_int, VarInt};
use crate::core::network::{MainMessage, NetworkMessage, UReceiver, USender};
use crate::core::player::PlayerSkin;
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

            const FLAME_OF_WAR: &str = "ewogICJ0aW1lc3RhbXAiIDogMTc1OTQzODI1MzM2OCwKICAicHJvZmlsZUlkIiA6ICI4YTdhZDkyMzc3MjI0ZjIyOGMwNDI4Y2I1YmQ5NzJkYSIsCiAgInByb2ZpbGVOYW1lIiA6ICJGbGFtZU9mV2FyIiwKICAic2lnbmF0dXJlUmVxdWlyZWQiIDogdHJ1ZSwKICAidGV4dHVyZXMiIDogewogICAgIlNLSU4iIDogewogICAgICAidXJsIiA6ICJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlL2JiNDg4Njc1YjMxYTQyZTc5MDI0ZGUzOGY1YmQ3ODZhMzlmNzVhMmE2ZGJhMDk0NDc5MmQ0NDNjNjA1ZDE4ZjkiCiAgICB9CiAgfQp9";
            const FLAME_OF_WAR_SIG: &str = "UvRQflcS0w4KTJSN+fpqYxVBTwo6wb66JMp6seThrmSGwUmbPfs8WEK2TPBIcipG0kBjWWdDMUpXFZ5YMBshnb7kHh588oPeL0gja/m9yHGEgtfucyqudL3m4sq3iZnJbdO3yKnF/00WqelBI5fZ3zc9SDyAjLUL4QHIXPm4U/z3UH1ZnVjGc5bZbV7qXILw7pF00al8ks1kpOUeds8zjSpVMRMTF9WQww89jNjbpvzcKP97KOOBXPJB1cuTUi3DEe3/9omZhcfgDyZDDJkmF3hTVZx1ijKtknlKRJqFcUEmsL1XUgRxqLSYNt1D1XCjEJeWAyT5YDVtvuj3Oa/zEeWQa9WVSXaUTGpVpQBRJrTJmtLH4O4hDMz4j7M2T0lsbOg7sIqvWVRvmKptKlLWKSWk8tlYXrx+Ef4YN5iva8/xhnKZmfe/JmT8uIKtNiv8Zcrj1WXasJ4wz0JCEQBOJDJXnEU548Sk1nxAcmX/W8jHkMnXArE3LKkLdxD7e++Hw60pv3GcyvTou5Mlrmgo6rHk188Li4CU826i+z0OuodRtdY+vsQIoFWLnnHu4HdqKA3IevcV7+Gl3FDzbzPXiSbUmSAV4drpLELTTPMnhhvMK85zS8138LTuScBiFRKVaSuXZJS7UIJ6VtjYK+iEuVblN9BJihP2NiuubCeL484=";


            // todo: in future, if using mojang auth
            // get skin of player and await async
            
            network_tx.send(packet_buffer.get_packet_message(client_id))?;
            main_tx.send(MainMessage::AddPlayer {
                client_id,
                username,
                uuid,
                skin: PlayerSkin {
                    texture: FLAME_OF_WAR.to_string(),
                    _signature: Some(FLAME_OF_WAR_SIG.to_string()),
                },
            })?;
        }
        _ => bail!("Unknown packet id during login")
    }

    Ok(())
}
