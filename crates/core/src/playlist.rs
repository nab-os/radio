use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct Playlist {
    pub mode: Mode,
    pub playlist: VecDeque<Uuid>,
    pub next_up: VecDeque<Uuid>,
    pub suggestions: VecDeque<(Uuid, u32)>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    MANUAL,
    AUTO,
}

impl Playlist {
    pub fn new() -> Self {
        Playlist {
            playlist: VecDeque::new(),
            next_up: VecDeque::new(),
            suggestions: VecDeque::new(),
            mode: Mode::AUTO,
        }
    }

    pub fn next_track_id(self: &mut Self) -> Option<Uuid> {
        let next_track = match self.mode {
            Mode::MANUAL => {
                if !self.playlist.is_empty() {
                    self.playlist.pop_front()
                } else {
                    self.next_up.pop_front()
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
