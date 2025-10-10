use crate::network::client::Client;
use crate::network::connection_state::ConnectionState;
use crate::network::internal_packets::MainThreadMessage;
use crate::network::packets::packet::{ProcessContext, ProcessPacket};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::login::clientbound::LoginSuccess;
use crate::player::player::{GameProfile, GameProfileProperty};
use crate::register_serverbound_packets;
use crate::types::sized_string::SizedString;
use blocks::packet_deserializable;
use fstr::FString;
use std::collections::HashMap;
use uuid::Uuid;

register_serverbound_packets! {
    Login;
    LoginStart = 0x00;
    // EncryptionResponse = 0x01,
}

packet_deserializable! {
    pub struct LoginStart {
        pub username: SizedString<16>
    }
}

impl ProcessPacket for LoginStart {
    async fn process<'a>(&self, client: &mut Client, context: ProcessContext<'a>) -> anyhow::Result<()> {
        println!("player {} attempted to join", self.username);
        let mut packet_buffer = PacketBuffer::new();
        let uuid_str = "d74cb748-b23b-4a99-b41e-b85f73d41999";
        
        pub const FLAME_OF_WAR: &str = "ewogICJ0aW1lc3RhbXAiIDogMTc1OTQzODI1MzM2OCwKICAicHJvZmlsZUlkIiA6ICI4YTdhZDkyMzc3MjI0ZjIyOGMwNDI4Y2I1YmQ5NzJkYSIsCiAgInByb2ZpbGVOYW1lIiA6ICJGbGFtZU9mV2FyIiwKICAic2lnbmF0dXJlUmVxdWlyZWQiIDogdHJ1ZSwKICAidGV4dHVyZXMiIDogewogICAgIlNLSU4iIDogewogICAgICAidXJsIiA6ICJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlL2JiNDg4Njc1YjMxYTQyZTc5MDI0ZGUzOGY1YmQ3ODZhMzlmNzVhMmE2ZGJhMDk0NDc5MmQ0NDNjNjA1ZDE4ZjkiCiAgICB9CiAgfQp9";
        pub const FLAME_OF_WAR_SIG: &str = "UvRQflcS0w4KTJSN+fpqYxVBTwo6wb66JMp6seThrmSGwUmbPfs8WEK2TPBIcipG0kBjWWdDMUpXFZ5YMBshnb7kHh588oPeL0gja/m9yHGEgtfucyqudL3m4sq3iZnJbdO3yKnF/00WqelBI5fZ3zc9SDyAjLUL4QHIXPm4U/z3UH1ZnVjGc5bZbV7qXILw7pF00al8ks1kpOUeds8zjSpVMRMTF9WQww89jNjbpvzcKP97KOOBXPJB1cuTUi3DEe3/9omZhcfgDyZDDJkmF3hTVZx1ijKtknlKRJqFcUEmsL1XUgRxqLSYNt1D1XCjEJeWAyT5YDVtvuj3Oa/zEeWQa9WVSXaUTGpVpQBRJrTJmtLH4O4hDMz4j7M2T0lsbOg7sIqvWVRvmKptKlLWKSWk8tlYXrx+Ef4YN5iva8/xhnKZmfe/JmT8uIKtNiv8Zcrj1WXasJ4wz0JCEQBOJDJXnEU548Sk1nxAcmX/W8jHkMnXArE3LKkLdxD7e++Hw60pv3GcyvTou5Mlrmgo6rHk188Li4CU826i+z0OuodRtdY+vsQIoFWLnnHu4HdqKA3IevcV7+Gl3FDzbzPXiSbUmSAV4drpLELTTPMnhhvMK85zS8138LTuScBiFRKVaSuXZJS7UIJ6VtjYK+iEuVblN9BJihP2NiuubCeL484=";

        let game_profile = GameProfile {
            uuid: Uuid::parse_str(uuid_str)?,
            username: FString::new(&*self.username),
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

        client.game_profile = Some(game_profile.clone());

        packet_buffer.write_packet(&LoginSuccess {
            uuid: FString::new_static(uuid_str),
            name: game_profile.username.clone(),
        });
        
        context.network_thread_tx.send(packet_buffer.get_packet_message(client.client_id))?;
        context.main_thread_tx.send(MainThreadMessage::NewPlayer {
            client_id: client.client_id,
            profile: game_profile
        })?;
        client.connection_state = ConnectionState::Play;
        Ok(())
    }
}