use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures_channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, StreamExt, select};
use radio_core::net::protocol::{ClientToServer, ServerToClient};
use tokio_tungstenite_wasm::Message;
use tokio_tungstenite_wasm::connect;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct Client {
    outbound: UnboundedSender<ClientToServer>,
}

impl Client {
    pub fn connect(
        url: String,
        running: Arc<AtomicBool>,
    ) -> (Self, UnboundedReceiver<ServerToClient>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded::<ClientToServer>();
        let (incoming_tx, incoming_rx) = mpsc::unbounded::<ServerToClient>();

        spawn_local(async move { run_connection(url, running, outgoing_rx, incoming_tx).await });

        (
            Client {
                outbound: outgoing_tx,
            },
            incoming_rx,
        )
    }
}

async fn run_connection(
    url: String,
    running: Arc<AtomicBool>,
    mut outgoing_rx: UnboundedReceiver<ClientToServer>,
    incoming_tx: UnboundedSender<ServerToClient>,
) {
    let ws = connect(url).await.expect("can't connect websocket");
    let (mut sink, source) = ws.split();

    let mut source = source.fuse();

    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        select! {
            outgoing = outgoing_rx.next() => match outgoing {
                Some(payload) => {
                    let json = match serde_json::to_string(&payload) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("serialize error: {e}");
                            continue;
                        }
                    };
                    if sink.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                None => break,
            },
            incoming = source.next() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    match serde_json::from_str::<ServerToClient>(&text) {
                        Ok(event) => {
                            if incoming_tx.unbounded_send(event).is_err() {
                                break;
                            }
                        }
                        Err(e) => tracing::warn!("invalid payload: {e}"),
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    tracing::warn!("ws error: {e}");
                    break;
                }
                None => break,
            },
        }
    }
}
