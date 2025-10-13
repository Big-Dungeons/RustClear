use crate::network::client::handle_client;
use core::panic;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::network::internal_packets::{ClientHandlerMessage, MainThreadMessage, NetworkThreadMessage};
use crate::player::player::ClientId;

type Sender<T> = UnboundedSender<T>;
type Receiver<T> = UnboundedReceiver<T>;

type ClientMap = HashMap<ClientId, UnboundedSender<ClientHandlerMessage>>;

pub fn start_network(
    ip: &'static str,
    status: Arc<String> // should probably be mutable
) -> (Sender<NetworkThreadMessage>, Receiver<MainThreadMessage>) {
    let (network_tx, network_rx) = unbounded_channel::<NetworkThreadMessage>();
    let (main_tx, main_rx) = unbounded_channel::<MainThreadMessage>();
    tokio::spawn(run_network_thread(ip, status, network_rx, network_tx.clone(), main_tx));
    (network_tx, main_rx)
}

async fn run_network_thread(
    ip: &'static str,
    status: Arc<String>,
    mut network_rx: Receiver<NetworkThreadMessage>,
    network_tx: Sender<NetworkThreadMessage>,
    main_tx: Sender<MainThreadMessage>,
) {
    let listener = TcpListener::bind(ip).await.unwrap();
    println!("Network thread listening on {ip}");

    let mut clients: ClientMap = HashMap::new();
    let mut client_id_counter: ClientId = 1;

    loop {
        tokio::select! {
            // a client failing to connect here is recoverable and doesnt really do anything, so we can just ignore it.
            // we do need to continue on a failed connection though, otherwise it would need to wait for network_rx to receive
            // before attempting to get a new connection.
            result = listener.accept() => {
                let Ok((socket, _)) = result else { continue };
                let client_id: ClientId = client_id_counter;
                client_id_counter += 1;

                // do we need tx and rx for each client?
                //
                let (client_tx, client_rx) = unbounded_channel::<ClientHandlerMessage>();
                clients.insert(client_id, client_tx);
                tokio::spawn(handle_client(
                    client_id,
                    socket,
                    client_rx,
                    main_tx.clone(),
                    network_tx.clone(),
                    status.clone()
                ));
            }

            // this can never be none since this function owns a network_tx.
            Some(msg) = network_rx.recv() => {
                // we can just discard main thread -> network thread messages with a disconnected client_id
                // as the main thread either already has or will be be informed shortly of this issue
                match msg { 
                    NetworkThreadMessage::SendPackets { client_id, buffer } => {
                        if let Some(client_tx) = clients.get(&client_id) {
                            if let Err(e) = client_tx.send(ClientHandlerMessage::Send(buffer)) {
                                eprintln!("Client {} handler dropped its reciever: {}", client_id, e);
                                disconnect_client(client_id, &main_tx, &mut clients);
                            }
                        }
                    }
            
                    NetworkThreadMessage::DisconnectClient { client_id } => {
                        if let Some(client_tx) = clients.get(&client_id) {
                            if let Err(e) = client_tx.send(ClientHandlerMessage::CloseHandler) {
                                eprintln!("Client {} handler dropped its reciever: {}", client_id, e);
                                disconnect_client(client_id, &main_tx, &mut clients);
                            }
                        }
                    }
            
                    NetworkThreadMessage::ConnectionClosed { client_id } => {
                        disconnect_client(client_id, &main_tx, &mut clients)
                    }
                }
            }
        }
    }
}

fn disconnect_client(client_id: ClientId, main_tx: &UnboundedSender<MainThreadMessage>, clients: &mut ClientMap) {
    main_tx.send(MainThreadMessage::ClientDisconnected { client_id }).expect("Main thread should never drop its network reciever.");
    clients.remove(&client_id);
}