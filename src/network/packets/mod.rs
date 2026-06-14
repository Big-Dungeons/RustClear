pub mod packet_serializable;
pub mod packet_deserializable;

use crate::network::client::ClientId;
use crate::network::packets::packet_serializable::PacketSerializable;
use crate::network::protocol::var_int::{var_int_size, write_var_int};
use crate::network::NetworkMessage;
use bevy::prelude::{Deref, Entity, Message};
use bytes::BytesMut;
use std::fmt::Debug;

pub trait IdentifiedPacket {
    const PACKET_ID: i32;
}

#[derive(Message, Deref)]
pub struct PacketEvent<P> {
    pub client: Entity,
    #[deref]
    pub packet: P
}

impl<P> PacketEvent<P> {
    pub fn client(&self) -> Entity {
        self.client
    }
}

#[macro_export]
macro_rules! register_serverbound_packets {
    (
        $enum_name:ident;
        $( $packet_type:ident ),* $(,)?
    ) => {
        paste::paste! {
            #[derive(bevy::ecs::system::SystemParam)]
            pub struct [< $enum_name Writers >]<'w> {
                $( pub [< $packet_type:snake >]: MessageWriter<'w, PacketEvent<$packet_type>>, )*
            }

            impl<'w> [< $enum_name Writers >]<'w> {
                pub fn dispatch(&mut self, entity: Entity, buffer: &mut Bytes) -> anyhow::Result<()> {
                    let Some(packet_id) = read_var_int(buffer) else {
                        anyhow::bail!("failed to read var_int")
                    };
                    match packet_id.0 {
                        $(
                            <$packet_type as IdentifiedPacket>::PACKET_ID => {
                                let packet = <$packet_type as PacketDeserializable>::read(buffer)?;
                                self.[< $packet_type:snake >].write(PacketEvent { client: entity, packet });
                            }
                        )*
                        _ => {
                            eprintln!("invalid packet: 0x{:02x}", packet_id.0);
                        }
                    }
                    Ok(())
                }
            }

            pub fn [< register_ $enum_name:snake >](app: &mut App) {
                $( app.add_message::<PacketEvent<$packet_type>>(); )*
            }
        }
    };
}

pub trait BytesMutExt {
    fn write_packet<P: IdentifiedPacket + PacketSerializable + Debug>(&mut self, packet: &P);
    fn get_packet_message(&mut self, client_id: ClientId) -> NetworkMessage;
}

impl BytesMutExt for BytesMut {
    fn write_packet<P: IdentifiedPacket + PacketSerializable + Debug>(&mut self, packet: &P) {
        // println!("packet written {:?}", packet);
        let write_size = (var_int_size(P::PACKET_ID) + packet.write_size()) as i32;
        self.reserve(write_size as usize + var_int_size(write_size));

        write_var_int(self, write_size);
        write_var_int(self, P::PACKET_ID);
        packet.write(self);
    }

    fn get_packet_message(&mut self, client_id: ClientId) -> NetworkMessage {
        NetworkMessage::SendPackets { client_id, buffer: self.split().freeze() }
    }
}