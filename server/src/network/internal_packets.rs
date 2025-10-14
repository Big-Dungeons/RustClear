use crate::network::protocol::play::serverbound::Play;
use crate::player::player::{ClientId, GameProfile};
use crate::types::status::StatusUpdate;
use bytes::Bytes;

pub enum NetworkThreadMessage {
    UpdateStatus(StatusUpdate),
    
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
    /// the main thread should wait for a ClientDisconnected response to handle actually removing the player.
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

    /// sent to the main thread when a client is removed for any reason, even reasons caused by the main thread.
    ClientDisconnected {
        client_id: ClientId,
    },
}