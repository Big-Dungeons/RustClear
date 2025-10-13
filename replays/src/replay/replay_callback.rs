use crate::replay_packet::ReplayPacket;

pub trait ReplayCallback: Send + Sync {
    fn callback(&mut self, packet: ReplayPacket) -> impl std::future::Future<Output = ()> + Send;
}