use crate::net::connection_state::ConnectionState;
use crate::net::connection_state::ConnectionState::*;
use crate::net::internal_packets::{ClientHandlerMessage, MainThreadMessage, NetworkThreadMessage};
use crate::net::packets::packet::{ProcessContext, ProcessPacket};
use crate::net::packets::packet_deserialize::PacketDeserializable;
use crate::net::protocol::handshake::serverbound::HandshakePacket;
use crate::net::protocol::login::serverbound::Login;
use crate::net::protocol::play::serverbound::Play;
use crate::net::protocol::status::serverbound::Status;
use crate::net::var_int::peek_var_int;
use crate::server::player::player::ClientId;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: ClientId,
    pub connection_state: ConnectionState,
}

impl Client {
    pub const fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            connection_state: Handshaking,
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
                    Ok(0) => { 
                        // Channel closed normally
                        break 
                    },
                    
                    Ok(_) => {
                        read_packets(&mut bytes, &mut client, &network_tx, &main_tx).await
                    }
                    Err(e) => {
                        eprintln!("Client {client_id} read error: {e}");
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

                    ClientHandlerMessage::CloseHandler => {
                        break
                    }
                }
            }
        }
    }

    let _ = network_tx.send(NetworkThreadMessage::ConnectionClosed { client_id });
    println!("handle client for {client_id} closed.");
}

async fn read_packets(
    buffer: &mut BytesMut,
    client: &mut Client,
    network_thread_tx: &UnboundedSender<NetworkThreadMessage>,
    main_thread_tx: &UnboundedSender<MainThreadMessage>
) {
    while let Some(mut buffer) = read_whole_packet(buffer).await {
        let context = ProcessContext { network_thread_tx, main_thread_tx, };
        match client.connection_state {
            Handshaking => parse_from_packets::<HandshakePacket>(&mut buffer, client, context).await,
            Status => parse_from_packets::<Status>(&mut buffer, client, context).await,
            Login => parse_from_packets::<Login>(&mut buffer, client, context).await,
            Play => {
                match Play::read(&mut buffer) {
                    Ok(packet) => {
                        if let Err(err) = packet.process(client, context).await {
                            eprintln!("error processing {err}");
                            continue
                        }
                        let _ = main_thread_tx.send(
                            MainThreadMessage::PacketReceived {
                                client_id: client.client_id, 
                                packet
                            }
                        );
                    }
                    Err(err) => {
                        eprintln!("Failed to parse packet from {err}")
                    }
                }
            }
        }
    }
}

async fn read_whole_packet(buf: &mut BytesMut) -> Option<BytesMut> {
    if buf.is_empty() {
        return None;
    }
    let (packet_len, varint_len) = peek_var_int(buf)?;

    let packet_len = packet_len as usize;
    if buf.len() < packet_len + varint_len {
        return None;
    }

    buf.advance(varint_len);
    Some(buf.split_to(packet_len))
}

async fn parse_from_packets<'a, P: PacketDeserializable + ProcessPacket>(
    buffer: &mut BytesMut,
    client: &mut Client,
    process_context: ProcessContext<'a>
) {
    match P::read(buffer) {
        Ok(packet) => {
            if let Err(e) = packet.process(client, process_context).await {
                eprintln!("error processing {e}");
            }
        }
        Err(e) => {
            eprintln!("Failed to parse packet from {e}")
        }
    }
}