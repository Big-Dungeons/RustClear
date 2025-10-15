use crate::network::client::{ClientHandler, ClientKey};
use crate::network::connection_state::ConnectionState;
use crate::types::status::Status;
use core::panic;
use slotmap::SlotMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};

type Sender<T> = UnboundedSender<T>;
type Receiver<T> = UnboundedReceiver<T>;


pub fn start_network(
    ip: &'static str,
    status: Status,
) -> (Sender<NetworkThreadMessage>, Receiver<MainThreadMessage>) {
    let (network_tx, network_rx) = unbounded_channel::<NetworkThreadMessage>();
    let (main_tx, main_rx) = unbounded_channel::<MainThreadMessage>();
    tokio::spawn(run_network_thread(ip, status, network_rx, network_tx.clone(), main_tx));
    (network_tx, main_rx)
}

async fn run_network_thread(
    ip: &'static str,
    mut status: Status,
    mut network_rx: Receiver<NetworkThreadMessage>,
    network_tx: Sender<NetworkThreadMessage>,
    main_tx: Sender<MainThreadMessage>,
) {
    let listener = TcpListener::bind(ip).await.unwrap();
    println!("Network thread listening on {ip}");

    // slotmap is faster than a hashmap and works just as well for us here
    let mut clients: SlotMap<ClientKey, ClientHandler> = SlotMap::with_key();
    
    loop {
        tokio::select! {
            // a client failing to connect here is recoverable and doesnt really do anything, so we can just ignore it.
            // we do need to continue on a failed connection though, otherwise it would need to wait for network_rx to receive
            // before attempting to get a new connection.
            result = listener.accept() => {
                let Ok((socket, _)) = result else { continue };
                
                clients.insert_with_key(|key| {
                    ClientHandler::spawn(key, socket, network_tx.clone(), main_tx.clone(), status.get())
                });
            }

            // this can never be none since this function owns a network_tx.
            Some(msg) = network_rx.recv() => {
                // we can just discard main thread -> network thread messages with a disconnected client_id
                // as the main thread either already has or will be be informed shortly of this issue
                match msg { 
                    NetworkThreadMessage::UpdateStatus(update) => status.set(update),
                    NetworkThreadMessage::SendPackets { client_id, buffer } => {
                        if let Some(handler) = clients.get(client_id) {
                            if let Err(e) = handler.send(buffer) {
                                eprintln!("Client {:?} handler dropped its reciever {}", client_id, e);
                                clients.remove(client_id);
                                main_tx.send(MainThreadMessage::ClientDisconnected { client_id }).expect("Main thread should never drop its network reciever.");
                            }
                        }
                    }
            
                    NetworkThreadMessage::DisconnectClient { client_id } => {
                        if clients.remove(client_id).is_some() {
                            main_tx.send(MainThreadMessage::ClientDisconnected { client_id }).expect("Main thread should never drop its network reciever.");
                        }
                    }
            
                    NetworkThreadMessage::ConnectionClosed { client_id, connection_state } => {
                        // we probably shouldnt tell the main thread a client it never added got disconnected? 
                        if clients.remove(client_id).is_some() && connection_state == ConnectionState::Play {
                            main_tx.send(MainThreadMessage::ClientDisconnected { client_id }).expect("Main thread should never drop its network reciever.");
                        }
                    }
                }
            }
        }
    }
}