use radio_core::data::track::{self, Track};
use std::{collections::HashMap, path::PathBuf};

use rand::seq::IndexedRandom;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::data::track::{load_track, load_track_info};

pub struct Library {
    pub library: HashMap<Uuid, track::Info>,
}

impl Library {
    pub fn new(path: PathBuf) -> Self {
        let library = list_track_infos(path);
        Library { library }
    }

    pub fn get_track_info(self: &Self, id: Uuid) -> Option<track::Info> {
        self.library.get(&id).cloned()
    }

    pub fn get_track(self: &Self, id: Uuid) -> Option<Track> {
        self.get_track_info(id).map(|infos| {
            load_track(infos.path)
                .join()
                .expect("Could not load track")
                .1
        })
    }

    pub fn pick_random(self: &Self) -> Option<Uuid> {
        let track_ids: Vec<&Uuid> = self.library.keys().collect();
        track_ids.choose(&mut rand::rng()).cloned().copied()
    }
}

fn list_track_infos(path: PathBuf) -> HashMap<Uuid, track::Info> {
    let paths: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            !e.file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    paths
        .par_iter()
        .map(|path| (Uuid::new_v4(), load_track_info(path.clone())))
        .collect()
}
