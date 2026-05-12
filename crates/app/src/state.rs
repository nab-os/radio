use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use radio_core::{data::track, playlist};
use uuid::Uuid;

use crate::net::client::Client;

#[derive(Clone, Copy)]
pub struct AppState {
    pub current_track_id: Signal<Option<Uuid>>,
    pub current_track_start_time: Signal<Option<DateTime<Utc>>>,
    pub current_track_duration: Signal<Option<Duration>>,
    pub now: Signal<DateTime<Utc>>,
    pub library: Signal<HashMap<Uuid, track::Info>>,
    pub playlist_ids: Signal<VecDeque<Uuid>>,
    pub next_up_ids: Signal<VecDeque<Uuid>>,
    pub suggestions: Signal<VecDeque<(Uuid, u32)>>,
    pub connected: Signal<bool>,
    pub client: Signal<Option<Client>>,
    pub mode: Signal<playlist::Mode>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_track_id: Signal::new(None),
            current_track_start_time: Signal::new(None),
            current_track_duration: Signal::new(None),
            now: Signal::new(Utc::now()),
            library: Signal::new(HashMap::new()),
            playlist_ids: Signal::new(VecDeque::new()),
            next_up_ids: Signal::new(VecDeque::new()),
            suggestions: Signal::new(VecDeque::new()),
            connected: Signal::new(false),
            client: Signal::new(None),
            mode: Signal::new(playlist::Mode::AUTO),
        }
    }
}
