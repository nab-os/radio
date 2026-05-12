use app::net::client::Client;
use dioxus::prelude::*;
use radio_core::{data::track, net::protocol::ClientToServer};

#[derive(Clone, Copy)]
pub struct AppState {
    pub current_track: Signal<Option<track::Info>>,
    pub library: Signal<Vec<track::Info>>,
    pub playlist: Signal<Vec<track::Info>>,
    pub next_up: Signal<Vec<track::Info>>,
    pub connected: Signal<bool>,
    pub client: Signal<Option<Client>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_track: Signal::new(None),
            library: Signal::new(Vec::new()),
            playlist: Signal::new(Vec::new()),
            next_up: Signal::new(Vec::new()),
            connected: Signal::new(false),
            client: Signal::new(None),
        }
    }

    /// Convenience: send a message to the server, if connected.
    pub fn send(&self, msg: ClientToServer) {
        if let Some(tx) = self.outbound.read().as_ref() {
            let _ = tx.unbounded_send(msg);
        }
    }
}
