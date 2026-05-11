use radio_core::net::protocol::{ClientMessage, ClientToServer, ServerEvent, ServerToClient};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinSet;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tokio_tungstenite::{WebSocketStream, accept_async};
use uuid::Uuid;

pub struct ServerInner {
    bind_address: String,
    clients: HashMap<Uuid, SplitSink<WebSocketStream<TcpStream>, Message>>,
}

#[derive(Clone)]
pub struct Server {
    inner: Arc<Mutex<ServerInner>>,
}

impl Server {
    pub fn new(bind_address: String) -> Self {
        let inner = Arc::new(Mutex::new(ServerInner {
            bind_address,
            clients: HashMap::new(),
        }));
        Server { inner }
    }

    pub async fn start_server(
        self: &mut Self,
        running: Arc<AtomicBool>,
        tx: UnboundedSender<ServerEvent>,
    ) {
        let listener = {
            let inner = self.inner.clone();
            let inner_guard = inner.lock().unwrap();

            TcpListener::bind(inner_guard.bind_address.clone())
                .await
                .expect("could not open port 8080")
        };
        let mut tasks = JoinSet::new();

        let inner = self.inner.clone();
        let tx = tx.clone();
        loop {
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    let inner = inner.clone();
                    let tx = tx.clone();
                    tasks.spawn(async move {
                        let Ok(ws) = accept_async(stream).await else {
                            return;
                        };
                        let client_id = Uuid::new_v4();
                        let (sink, mut source) = ws.split();
                        {
                            let mut inner_guard = inner.lock().unwrap();
                            inner_guard.clients.insert(client_id, sink);
                        }

                        tx.send(ServerEvent::ClientConnected(client_id)).unwrap();

                       while let Some(Ok(msg)) = source.next().await {
                            if let Message::Text(text) = msg {
                                match serde_json::from_str::<ClientToServer>(&text) {
                                    Ok(payload) => {
                                        tx.send(ServerEvent::Message(ClientMessage {
                                            client_id,
                                            payload
                                        })).unwrap();
                                    }
                                    Err(e) => eprintln!("invalid payload: {e}"),
                                }
                            }
                        }
                        {
                            let mut inner_guard = inner.lock().unwrap();
                            inner_guard.clients.remove(&client_id);
                        }
                        tx.send(ServerEvent::ClientDisconnected(client_id)).unwrap();
                        eprintln!("{addr} disconnected");
                    });
                }
                _ = wait_for_shutdown(running.clone()) => {
                    break;
                }
            }
        }

        // Wait for all active connections to finish
        while tasks.join_next().await.is_some() {}
        println!("Stopped server");
    }

    pub async fn send(self: &Self, client_id: Uuid, payload: ServerToClient) {
        let mut inner_guard = self.inner.lock().unwrap();
        let sink = inner_guard
            .clients
            .get_mut(&client_id)
            .expect("Unknown client");
        let json = json!(payload).to_string();
        dbg!(json.len());
        sink.send(Message::Text(Utf8Bytes::from(json)))
            .await
            .expect("Could not send OutgoingPayload");
    }

    pub async fn broadcast(self: &Self, payload: ServerToClient) {
        let mut inner_guard = self.inner.lock().unwrap();
        for sink in inner_guard.clients.values_mut() {
            sink.send(Message::Text(Utf8Bytes::from(json!(payload).to_string())))
                .await
                .expect("Could not send OutgoingPayload");
        }
    }
}

async fn wait_for_shutdown(running: Arc<AtomicBool>) {
    while running.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
