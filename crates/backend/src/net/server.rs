use radio_core::net::protocol::{ClientMessage, ClientToServer, ServerEvent, ServerToClient};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::JoinSet;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use uuid::Uuid;

pub struct ServerInner {
    bind_address: String,
    clients: HashMap<Uuid, UnboundedSender<Message>>,
}

#[derive(Clone)]
pub struct Server {
    inner: Arc<Mutex<ServerInner>>,
}

impl Server {
    pub fn new(bind_address: String) -> Self {
        Server {
            inner: Arc::new(Mutex::new(ServerInner {
                bind_address,
                clients: HashMap::new(),
            })),
        }
    }

    pub async fn start_server(
        &mut self,
        running: Arc<AtomicBool>,
        tx: UnboundedSender<ServerEvent>,
    ) {
        let bind_address = self.inner.lock().unwrap().bind_address.clone();
        let listener = TcpListener::bind(&bind_address)
            .await
            .expect("could not open port");

        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    let inner = self.inner.clone();
                    let tx = tx.clone();
                    tasks.spawn(async move {
                        handle_connection(stream, addr, inner, tx).await;
                    });
                }
                _ = wait_for_shutdown(running.clone()) => {
                    break;
                }
            }
        }

        // Wait for all active connections to finish
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        println!("Stopped server");
    }

    pub fn send(&self, client_id: Uuid, payload: ServerToClient) -> bool {
        let json = match serde_json::to_string(&payload) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("serialize error: {e}");
                return false;
            }
        };
        let msg = Message::Text(Utf8Bytes::from(json));

        let inner_guard = self.inner.lock().unwrap();
        match inner_guard.clients.get(&client_id) {
            Some(sender) => sender.send(msg).is_ok(),
            None => false,
        }
    }

    pub fn broadcast(&self, payload: ServerToClient) {
        let json = match serde_json::to_string(&payload) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("serialize error: {e}");
                return;
            }
        };

        let inner_guard = self.inner.lock().unwrap();
        for sender in inner_guard.clients.values() {
            let _ = sender.send(Message::Text(Utf8Bytes::from(json.clone())));
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    inner: Arc<Mutex<ServerInner>>,
    tx: UnboundedSender<ServerEvent>,
) {
    let Ok(ws) = accept_async(stream).await else {
        eprintln!("{addr} handshake failed");
        return;
    };

    let client_id = Uuid::new_v4();
    let (mut sink, mut source) = ws.split();

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();

    {
        let mut inner_guard = inner.lock().unwrap();
        inner_guard.clients.insert(client_id, out_tx);
    }

    let _ = tx.send(ServerEvent::ClientConnected(client_id));

    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = source.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientToServer>(&text) {
                Ok(payload) => {
                    let _ = tx.send(ServerEvent::Message(ClientMessage { client_id, payload }));
                }
                Err(e) => eprintln!("invalid payload: {e}"),
            }
        }
    }

    {
        let mut inner_guard = inner.lock().unwrap();
        inner_guard.clients.remove(&client_id);
    }
    writer.abort();
    let _ = tx.send(ServerEvent::ClientDisconnected(client_id));
    eprintln!("{addr} disconnected");
}

async fn wait_for_shutdown(running: Arc<AtomicBool>) {
    while running.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
