use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct Playlist {
    pub mode: Mode,
    pub playlist: Vec<Uuid>,
    pub next_up: Vec<Uuid>,
    pub suggestions: Vec<(Uuid, u32)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Mode {
    MANUAL,
    AUTO,
}

impl Playlist {
    pub fn new() -> Self {
        Playlist {
            playlist: vec![],
            next_up: vec![],
            suggestions: vec![],
            mode: Mode::AUTO,
        }
    }

    pub fn next_track(self: &mut Self) -> Option<Uuid> {
        let next_track = match self.mode {
            Mode::MANUAL => {
                if !self.playlist.is_empty() {
                    self.playlist.pop()
                } else {
                    self.next_up.pop()
                }
            }
            Mode::AUTO => self
                .suggestions
                .iter()
                .max_by_key(|(_, votes)| *votes)
                .map(|id| id.0)
                .clone(),
        };
        self.suggestions.clear();
        next_track
    }

    pub fn clear_next(self: &mut Self) {
        self.next_up.clear();
    }
}
