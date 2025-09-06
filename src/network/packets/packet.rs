use crate::network::client::Client;
use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::player::player::{Player, PlayerExtension};
use tokio::sync::mpsc::UnboundedSender;

/// used for client bound packets, to identify them
pub trait IdentifiedPacket {
    const PACKET_ID: i32;
}

/// Implements IdentifiedPacket for all entries with the corresponding packet id.
#[macro_export]
macro_rules! register_packets {
    ($($packet:ty = $id:expr);* $(;)?) => {
        $(
            impl IdentifiedPacket for $packet {
                const PACKET_ID: i32 = $id;
            }
        )*
    };
}

pub struct ProcessContext<'a> {
    pub network_thread_tx: &'a UnboundedSender<NetworkThreadMessage>,
    pub main_thread_tx: &'a UnboundedSender<MainThreadMessage>,
}

pub trait ProcessPacket {
    async fn process<'a>(&self, _: &mut Client, _: ProcessContext<'a>) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// processes (play) packet sent by the player.
    /// 
    /// this must be run on the main thread.
    fn process_with_player<P : PlayerExtension>(&self, player: &mut Player<P>) {}
}

// since this doesn't need to be imported often (unlike client bound packets)
// it can use an enum just fine, (no annoying importing)
#[macro_export]
macro_rules! register_serverbound_packets {
    (
        $enum_name:ident;
        $( $packet_type:ident = $id:literal );* $(;)?
    ) => {
        pub enum $enum_name {
            $( $packet_type($packet_type), )*
        }
        
        impl crate::network::packets::packet_deserialize::PacketDeserializable for $enum_name {
            fn read(buffer: &mut bytes::BytesMut) -> anyhow::Result<Self> {
                if let Some(packet_id) = crate::network::binary::var_int::read_var_int(buffer) {
                    match packet_id {
                        $(
                            $id => Ok($enum_name::$packet_type(
                                <$packet_type as crate::network::packets::packet_deserialize::PacketDeserializable>::read(buffer)?
                            )),
                        )*
                    _ => anyhow::bail!(": invalid packet 0x{:02x}", packet_id),
                    }
                } else {
                    anyhow::bail!("failed to read var_int")
                }
            }
        }
        
        impl crate::network::packets::packet::ProcessPacket for $enum_name {
            async fn process<'a>(&self, client: &mut Client, ctx: crate::network::packets::packet::ProcessContext<'a>) -> anyhow::Result<()> {
                use crate::network::packets::packet::ProcessPacket;
                match self {
                    $(
                        $enum_name::$packet_type(inner) => {
                            <_ as ProcessPacket>::process(inner, client, ctx).await
                        }
                    )*
                }
            }
            
            fn process_with_player<P : crate::player::player::PlayerExtension>(&self, player: &mut  crate::player::player::Player<P>) {
                match self {
                    $(
                        $enum_name::$packet_type(inner) => {
                            <_ as ProcessPacket>::process_with_player(inner, player)
                        }
                    )*
                }
            }
        }
    };
}

