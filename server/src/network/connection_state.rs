use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Handshaking,
    Play,
    Status,
    Login,
}

impl ConnectionState {
    pub fn from_id(id: i32) -> Result<ConnectionState> {
        Ok(match id {
            -1 => ConnectionState::Handshaking,
            0 => ConnectionState::Play,
            1 => ConnectionState::Status,
            2 => ConnectionState::Login,
            _ => bail!("Invalid connection state id: {}", id),
        })
    }
}