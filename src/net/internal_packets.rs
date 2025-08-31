use crate::net::protocol::play::serverbound::Play;
use crate::server::player::player::{ClientId, GameProfile};
use bytes::Bytes;

// too many comments because theres 4 different client disconnect related messages and theyre all needed and do different things...

pub enum NetworkThreadMessage {
    SendPackets {
        client_id: ClientId,
        buffer: Bytes,
    },
    
    /// received when the client's handler is closed.
    /// sends a client disconnected message to the main thread
    ConnectionClosed {
        client_id: ClientId,
    },

    /// Disconnects the client from the server.
    /// This sends a close handler message to the client's handler.
    /// It should be sent after the vanilla disconnect packet is sent.
    DisconnectClient {
        client_id: ClientId,
    },
}

pub enum ClientHandlerMessage {
    Send(Bytes),
    /// Closes the handler for this client. This then sends a connection closed message to the network thread.
    CloseHandler,
}

pub enum MainThreadMessage {
    PacketReceived {
        client_id: ClientId,
        packet: Play,
    },

    NewPlayer {
        client_id: ClientId,
        profile: GameProfile,
    },

    ClientDisconnected {
        client_id: ClientId,
    },

    Abort {
        reason: String,
    },
}