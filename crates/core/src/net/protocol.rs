use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::net::dto::{CurrentTrack, IdPlaylist, InfoLibrary, Mode};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientToServer {
    ModeUpdate(Mode),
    PlaylistUpdate(IdPlaylist),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerToClient {
    CurrentTrackUpdate(CurrentTrack),
    LibraryUpdate(InfoLibrary),
    ModeUpdate(Mode),
    PlaylistUpdate(IdPlaylist),
}

pub struct ClientMessage {
    pub client_id: Uuid,
    pub payload: ClientToServer,
}

pub enum ServerEvent {
    ClientConnected(Uuid),
    ClientDisconnected(Uuid),
    Message(ClientMessage),
}
