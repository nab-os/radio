use std::path::PathBuf;

use argh::FromArgs;

/// Radio backend
#[derive(FromArgs)]
pub struct Args {
    /// path to music library
    #[argh(option, default = "PathBuf::from(\"~/Music\")")]
    pub music_path: PathBuf,

    /// websocket bind address
    #[argh(option, default = "String::from(\"0.0.0.0:8080\")")]
    pub bind: String,
}

pub fn parse() -> Args {
    argh::from_env()
}
