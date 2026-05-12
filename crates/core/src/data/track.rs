use std::{collections::VecDeque, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Info {
    pub path: PathBuf,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub cover: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Tech {
    pub channel_count: usize,
    pub sample_rate: u32,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub samples: VecDeque<f32>,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub info: Info,
    pub tech: Tech,
    pub data: Data,
}
