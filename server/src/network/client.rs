use crate::network::binary::var_int::{peek_var_int, VarInt};
use crate::network::connection_state::ConnectionState;
use crate::network::connection_state::ConnectionState::*;
use crate::network::internal_packets::{ClientHandlerMessage, MainThreadMessage, NetworkThreadMessage};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::protocol::handshake::serverbound::Handshake;
use crate::network::protocol::login::clientbound::LoginSuccess;
use crate::network::protocol::login::serverbound::LoginStart;
use crate::network::protocol::play::serverbound::Play;
use crate::network::protocol::status::clientbound::{StatusPong, StatusResponse};
use crate::network::protocol::status::serverbound::StatusPing;
use crate::player::player::{ClientId, GameProfile};
use crate::GameProfileProperty;
use anyhow::bail;
use bytes::{Buf, Bytes, BytesMut};
use fstr::FString;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: ClientId,
    pub connection_state: ConnectionState,
    
    pub game_profile: Option<GameProfile>,
}

impl Client {
    pub const fn new(client_id: ClientId) -> Self {
        Self {
            id: client_id,
            connection_state: Handshaking,
            
            game_profile: None,
        }
    }
}

// main thread tx errors can be ignored since the network/client threads will close eachother properly regardless of client status, 
// which will in turn close the client handlers.

pub async fn handle_client(
    client_id: ClientId,
    mut socket: TcpStream,
    mut rx: UnboundedReceiver<ClientHandlerMessage>,
    main_tx: UnboundedSender<MainThreadMessage>,
    network_tx: UnboundedSender<NetworkThreadMessage>,
) {
    let mut client = Client::new(client_id);
    let mut bytes = BytesMut::new();

    loop {
        tokio::select! {
            result = socket.read_buf(&mut bytes) => {
                match result {
                    Ok(0) => break, // channel closed normally
                    Ok(_) => {
                        // channel closes, if client sends invalid packets.
                        if let Err(err) = read_packets(&mut bytes, &mut client, &network_tx, &main_tx).await {
                            eprintln!("client {client_id:?} errored: {err}");
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("Client {client_id:?} read error: {e}");
                        break;
                    }
                }
            }

            Some(message) = rx.recv() => {
                match message {
                    ClientHandlerMessage::Send(data) => {
                        if let Err(e) = socket.write_all(&data).await {
                            eprintln!("write error: {e}");
                            break
                        }
                    }
                    ClientHandlerMessage::CloseHandler => break,
                }
            }
        }
    }

    let _ = network_tx.send(NetworkThreadMessage::ConnectionClosed { client_id });
    println!("handle client for {client_id:?} closed.");
}

async fn read_packets(
    buffer: &mut BytesMut,
    client: &mut Client,
    network_thread_tx: &UnboundedSender<NetworkThreadMessage>,
    main_thread_tx: &UnboundedSender<MainThreadMessage>
) -> anyhow::Result<()> {
    while let Some(mut buffer) = try_read_packet_slice(buffer) {
        match client.connection_state {
            Handshaking => {
                handle_handshake(&mut buffer, client)?;
            }
            Status => {
                handle_status(&mut buffer, client, network_thread_tx)?;
            }
            Login => {
                handle_login(&mut buffer, client, network_thread_tx, main_thread_tx)?;
            }
            Play => {
                match Play::read(&mut buffer) {
                    Ok(packet) => {
                        // if let Err(err) = packet.process(client, context).await {
                        //     eprintln!("error processing {err}");
                        //     continue
                        // }
                        let _ = main_thread_tx.send(
                            MainThreadMessage::PacketReceived {
                                client_id: client.id,
                                packet
                            }
                        );
                    }
                    Err(err) => eprintln!("Failed to parse packet from {err}"),
                }
            }
        }
    }
    Ok(())
}

fn try_read_packet_slice(buf: &mut impl Buf) -> Option<Bytes> {
    if !buf.has_remaining() {
        return None;
    }

    let (packet_len, var_int_len) = peek_var_int(buf)?;
    let packet_len = packet_len as usize;

    if buf.remaining() < packet_len + var_int_len {
        // packet incomplete
        return None;
    }

    buf.advance(var_int_len);
    Some(buf.copy_to_bytes(packet_len))
}

pub enum HandleResult {
    Ok,
    Disconnect,
}

fn handle_handshake(buffer: &mut impl Buf, client: &mut Client) -> anyhow::Result<()> {
    if let Ok(packet_id) = VarInt::read(buffer) {
        match *packet_id {
            0x00 => {
                let handshake = Handshake::read(buffer)?;
                client.connection_state = ConnectionState::from_id(handshake.next_state.0)?;
                Ok(())
            },
            _ => bail!("Unknown packet id during handshake")
        }
    } else {
        bail!("Failed to read var_int packet id.")
    }
}

fn handle_status(buffer: &mut impl Buf, client: &mut Client, network_tx: &UnboundedSender<NetworkThreadMessage>) -> anyhow::Result<()> {
    let packet_id = *VarInt::read(buffer)?;
    let mut packet_buffer = PacketBuffer::new();
    match packet_id {
        0x00 => {
            // todo: customizable response
            packet_buffer.write_packet(&StatusResponse::default());
        }
        0x01 => {
            let status_ping = StatusPing::read(buffer)?;
            packet_buffer.write_packet(&StatusPong {
                client_time: status_ping.client_time,
            });
        }
        _ => bail!("Unknown packet id during status")
    }
    network_tx.send(packet_buffer.get_packet_message(client.id))?;
    Ok(())
}

const FLAME_OF_WAR: &str = "ewogICJ0aW1lc3RhbXAiIDogMTc1OTQzODI1MzM2OCwKICAicHJvZmlsZUlkIiA6ICI4YTdhZDkyMzc3MjI0ZjIyOGMwNDI4Y2I1YmQ5NzJkYSIsCiAgInByb2ZpbGVOYW1lIiA6ICJGbGFtZU9mV2FyIiwKICAic2lnbmF0dXJlUmVxdWlyZWQiIDogdHJ1ZSwKICAidGV4dHVyZXMiIDogewogICAgIlNLSU4iIDogewogICAgICAidXJsIiA6ICJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlL2JiNDg4Njc1YjMxYTQyZTc5MDI0ZGUzOGY1YmQ3ODZhMzlmNzVhMmE2ZGJhMDk0NDc5MmQ0NDNjNjA1ZDE4ZjkiCiAgICB9CiAgfQp9";
const FLAME_OF_WAR_SIG: &str = "UvRQflcS0w4KTJSN+fpqYxVBTwo6wb66JMp6seThrmSGwUmbPfs8WEK2TPBIcipG0kBjWWdDMUpXFZ5YMBshnb7kHh588oPeL0gja/m9yHGEgtfucyqudL3m4sq3iZnJbdO3yKnF/00WqelBI5fZ3zc9SDyAjLUL4QHIXPm4U/z3UH1ZnVjGc5bZbV7qXILw7pF00al8ks1kpOUeds8zjSpVMRMTF9WQww89jNjbpvzcKP97KOOBXPJB1cuTUi3DEe3/9omZhcfgDyZDDJkmF3hTVZx1ijKtknlKRJqFcUEmsL1XUgRxqLSYNt1D1XCjEJeWAyT5YDVtvuj3Oa/zEeWQa9WVSXaUTGpVpQBRJrTJmtLH4O4hDMz4j7M2T0lsbOg7sIqvWVRvmKptKlLWKSWk8tlYXrx+Ef4YN5iva8/xhnKZmfe/JmT8uIKtNiv8Zcrj1WXasJ4wz0JCEQBOJDJXnEU548Sk1nxAcmX/W8jHkMnXArE3LKkLdxD7e++Hw60pv3GcyvTou5Mlrmgo6rHk188Li4CU826i+z0OuodRtdY+vsQIoFWLnnHu4HdqKA3IevcV7+Gl3FDzbzPXiSbUmSAV4drpLELTTPMnhhvMK85zS8138LTuScBiFRKVaSuXZJS7UIJ6VtjYK+iEuVblN9BJihP2NiuubCeL484=";


fn handle_login(
    buffer: &mut impl Buf,
    client: &mut Client,
    network_tx: &UnboundedSender<NetworkThreadMessage>,
    main_tx: &UnboundedSender<MainThreadMessage>,
) -> anyhow::Result<()> {
    let packet_id = *VarInt::read(buffer)?;
    match packet_id {
        0x00 => {
            // todo: skin authentication whatever
            let login = LoginStart::read(buffer)?;
            let mut packet_buffer = PacketBuffer::new();

            let uuid = Uuid::new_v4();

            let game_profile = GameProfile {
                uuid,
                username: FString::new(&*login.username),
                properties: HashMap::from(
                    [(
                        "textures".into(),
                        GameProfileProperty {
                            value: FLAME_OF_WAR.into(),
                            signature: Some(FLAME_OF_WAR_SIG.into()),
                        }
                    )]
                ),
            };

            packet_buffer.write_packet(&LoginSuccess {
                uuid: FString::new(&*uuid.hyphenated().to_string()),
                name: game_profile.username.clone(),
            });
            network_tx.send(packet_buffer.get_packet_message(client.id))?;
            main_tx.send(MainThreadMessage::NewPlayer {
                client_id: client.id,
                profile: game_profile,
            })?;
            client.connection_state = ConnectionState::Play;
        }
        _ => bail!("Unknown packet id during login")
    }
    Ok(())
}