use std::{collections::VecDeque, time::Duration};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{data::track, playlist};

#[derive(Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub info: track::Info,
}

#[derive(Serialize, Deserialize)]
pub struct IdPlaylist {
    pub playlist: VecDeque<Uuid>,
    pub next_up: VecDeque<Uuid>,
    pub suggestions: VecDeque<(Uuid, u32)>,
}

#[derive(Serialize, Deserialize)]
pub struct InfoLibrary {
    pub library: Vec<Track>,
}

#[derive(Serialize, Deserialize)]
pub struct Mode {
    pub mode: playlist::Mode,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentTrack {
    pub start_time: DateTime<Utc>,
    pub track_id: Uuid,
    pub duration: Duration,
}
