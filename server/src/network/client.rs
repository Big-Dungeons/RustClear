use std::collections::HashMap;

use anyhow::bail;
use bytes::{Buf, Bytes, BytesMut};
use fstr::FString;
use slotmap::new_key_type;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::mpsc::{UnboundedReceiver, UnboundedSender, error::SendError, unbounded_channel}, task::JoinHandle};
use uuid::Uuid;

use crate::{ClientId, GameProfile, GameProfileProperty, network::{binary::var_int::{VarInt, peek_var_int}, connection_state::ConnectionState, internal_packets::{MainThreadMessage, NetworkThreadMessage}, packets::{packet_buffer::PacketBuffer, packet_deserialize::PacketDeserializable}, protocol::{handshake::serverbound::Handshake, login::{clientbound::LoginSuccess, serverbound::LoginStart}, play::serverbound::Play, status::{clientbound::{StatusPong, StatusResponse}, serverbound::StatusPing}}}, types::status::StatusBytes};

new_key_type! {
    pub struct ClientKey;
}

#[derive(Debug, Clone)]
pub struct Client {
    pub id: ClientId,
    pub connection_state: ConnectionState,
    // pub game_profile: Option<GameProfile>,
}

pub struct ClientHandler {
    writer: UnboundedSender<Bytes>,
    handle: JoinHandle<()>
}

impl ClientHandler {
    pub fn spawn(
        client_id: ClientId, 
        socket: TcpStream, 
        network_tx: UnboundedSender<NetworkThreadMessage>, 
        main_tx: UnboundedSender<MainThreadMessage>,        
        status: StatusBytes
    ) -> Self {
        let (tx, rx) = unbounded_channel();
        let handle = tokio::spawn(run_client(client_id, socket, rx, network_tx, main_tx, status));
        Self { writer: tx, handle }
    }
    
    pub fn send(&self, data: Bytes) -> Result<(), SendError<Bytes>> {
        self.writer.send(data)
    }
    
    pub fn abort(self) {
        self.handle.abort();
    }
}

async fn run_client(
    client_id: ClientId, 
    mut socket: TcpStream, 
    mut rx: UnboundedReceiver<Bytes>,
    network_tx: UnboundedSender<NetworkThreadMessage>, 
    main_tx: UnboundedSender<MainThreadMessage>,
    status: StatusBytes
) {
    let mut client = Client {
        id: client_id,
        connection_state: ConnectionState::Handshaking,
        // game_profile: None,
    };
    let mut bytes = BytesMut::new();
    
    loop {
        tokio::select! {
            res = socket.read_buf(&mut bytes) => {
                match res {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Err(err) = read_packets(
                            &mut bytes,
                            &mut client,
                            &network_tx,
                            &main_tx,
                            &status
                        ).await {
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
            
            // we dont need a drop task if we just drop the tx...
            opt = rx.recv() => {
                match opt {
                    Some(bytes) => {
                        if let Err(e) = socket.write_all(&bytes).await {
                            eprintln!("Socket write error: {}", e);
                            break
                        }
                    }
                    None => break,
                }
            }
        }
    }
    
    println!("client reader closed");
    let _ = network_tx.send(NetworkThreadMessage::ConnectionClosed { client_id, connection_state: client.connection_state });
}

async fn read_packets(
    buffer: &mut BytesMut, 
    client: &mut Client, 
    network_tx: &UnboundedSender<NetworkThreadMessage>,
    main_tx: &UnboundedSender<MainThreadMessage>,
    status: &StatusBytes
) -> anyhow::Result<()> {
    while let Some(mut buffer) = try_read_packet_slice(buffer) {
        match client.connection_state {
            ConnectionState::Handshaking => handle_handshake(&mut buffer, client)?,
            ConnectionState::Status => handle_status(client.id, &mut buffer, network_tx, status)?,
            ConnectionState::Login => handle_login(&mut buffer, client, network_tx, main_tx)?,
            ConnectionState::Play => {
                let packet = Play::read(&mut buffer)?;
                if let Play::Invalid(packet_id) = packet {
                    eprintln!("invalid packet: 0x{packet_id:02x}");
                    continue;
                }
                main_tx.send(MainThreadMessage::PacketReceived { client_id: client.id, packet })?;
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

fn handle_handshake(buffer: &mut impl Buf, client: &mut Client) -> anyhow::Result<()> {
    match *VarInt::read(buffer)? {
        0x00 => {
            let handshake = Handshake::read(buffer)?;
            client.connection_state = ConnectionState::from_id(handshake.next_state.0)?;
        }
        _ => bail!("Unknown pack id during handshake."),
    }
    Ok(())
}
fn handle_status(
    client_id: ClientId,
    buffer: &mut impl Buf,
    network_tx: &UnboundedSender<NetworkThreadMessage>,
    status: &StatusBytes
) -> anyhow::Result<()> {
    let packet_id = *VarInt::read(buffer)?;
    let mut packet_buffer = PacketBuffer::new();
    match packet_id {
        0x00 => {
            packet_buffer.write_packet(&StatusResponse {
                status: status.get_str(),
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
    
    // we cannot directly send to socket since the read task cant own the write half, at least not without Arc<Mutex<>> i think
    network_tx.send(NetworkThreadMessage::SendPackets { client_id, buffer: packet_buffer.split_into_bytes() })?;
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
                username: FString::new(&login.username),
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

            client.connection_state = ConnectionState::Play;
            
            packet_buffer.write_packet(&LoginSuccess {
                uuid: uuid.hyphenated().to_string(),
                name: game_profile.username.clone(),
            });
            network_tx.send(NetworkThreadMessage::SendPackets { client_id: client.id, buffer: packet_buffer.split_into_bytes() })?;
            
            main_tx.send(MainThreadMessage::NewPlayer {
                client_id: client.id,
                profile: game_profile,
            })?;
        }
        _ => bail!("Unknown packet id during login")
    }
    Ok(())
}