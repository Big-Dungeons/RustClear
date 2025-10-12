use crate::player::player::{Player, PlayerExtension};

/// Used to identify packets sent to the client.
pub trait IdentifiedPacket {
    const PACKET_ID: i32;
}

/// Must be used on packets sent from a valid player to process them.
pub trait ProcessPacket {
    /// processes (play) packet sent by the player.
    ///
    /// this must be run on the main thread.
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>);
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

// since this doesn't need to be imported often (unlike client bound packets)
// it can use an enum just fine, (no annoying importing)
#[macro_export]
macro_rules! register_serverbound_packets {
    (
        $enum_name:ident;
        $( $packet_type:ident = $id:literal );* $(;)?
    ) => {
        pub enum $enum_name {
            Invalid(i32), // temporary, since not all packets are implemented, and without this it will error
            $( $packet_type($packet_type), )*
        }
        
        impl crate::network::packets::packet_deserialize::PacketDeserializable for $enum_name {
            fn read(buffer: &mut impl bytes::Buf) -> anyhow::Result<Self> {
                if let Some(packet_id) = crate::network::binary::var_int::read_var_int(buffer) {
                    match packet_id {
                        $(
                            $id => Ok($enum_name::$packet_type(
                                <$packet_type as crate::network::packets::packet_deserialize::PacketDeserializable>::read(buffer)?
                            )),
                        )*
                    _ => Ok($enum_name::Invalid(packet_id)),
                    }
                } else {
                    anyhow::bail!("failed to read var_int")
                }
            }
        }
        
        impl crate::network::packets::packet::ProcessPacket for $enum_name {
            fn process<P : crate::player::player::PlayerExtension>(&self, player: &mut  crate::player::player::Player<P>) {
                match self {
                    $(
                        $enum_name::$packet_type(inner) => {
                            <_ as ProcessPacket>::process(inner, player)
                        }
                    )*
                    $enum_name::Invalid(_) => unreachable!()
                }
            }
        }
    };
}

