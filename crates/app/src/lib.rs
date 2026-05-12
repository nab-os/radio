pub mod net;
pub mod state;

use dioxus::prelude::*;
use futures_util::StreamExt;
use radio_core::net::protocol::ServerToClient;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use wasm_bindgen_futures::spawn_local;

use crate::net::client::Client;
use crate::state::AppState;

pub fn connect(mut state: AppState, url: String, running: Arc<AtomicBool>) {
    let (client, mut events) = Client::connect(url, running);

    state.client.set(Some(client));
    state.connected.set(true);

    spawn_local(async move {
        while let Some(event) = events.next().await {
            apply_event(&mut state, event);
        }
        state.connected.set(false);
        state.client.set(None);
    });
}

fn apply_event(state: &mut AppState, event: ServerToClient) {
    match event {
        ServerToClient::LibraryUpdate(u) => {
            tracing::info!("Library update");
            state.library.set(
                u.library
                    .iter()
                    .map(|dto| (dto.id, dto.info.clone()))
                    .collect(),
            );
        }
        ServerToClient::PlaylistUpdate(u) => {
            tracing::info!("Playlist update");
            state.playlist_ids.set(u.playlist);
            state.next_up_ids.set(u.next_up);
            state.suggestions.set(u.suggestions);
        }
        ServerToClient::ModeUpdate(mode) => {
            tracing::info!("Mode update");
            state.mode.set(mode.mode);
        }
        ServerToClient::CurrentTrackUpdate(current_track) => {
            tracing::info!("Current track update");
            match current_track {
                Some(current_track) => {
                    state
                        .current_track_start_time
                        .set(Some(current_track.start_time));
                    state.current_track_id.set(Some(current_track.track_id));
                    state
                        .current_track_duration
                        .set(Some(current_track.duration));
                }
                None => todo!(),
            }
        }
    }
}
