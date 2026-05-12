use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use uuid::Uuid;

use radio_core::{
    data::track::Track,
    net::{
        dto,
        protocol::{ClientToServer, ServerEvent, ServerToClient},
    },
    playlist::{Mode, Playlist},
};

use crate::{audio::player::Player, library::Library, net::server::Server};

pub struct MiddlewareInner {
    pub music_library: Arc<Library>,
    pub playlist: Playlist,
    pub current_player: Option<Player>,
    pub current_track: Option<(Uuid, Track, DateTime<Utc>)>,
    pub server: Server,
    running: Arc<AtomicBool>,
    server_handle: Option<JoinHandle<()>>,
    play_handle: Option<JoinHandle<()>>,
    processing_handle: Option<JoinHandle<()>>,
}

impl MiddlewareInner {
    fn hydrate(self: &mut Self) {
        let missing = 3 - self.playlist.next_up.len();
        for _ in 0..missing {
            self.music_library.pick_random().map(|id| {
                self.playlist.next_up.push_back(id);
            });
        }
        let missing = 3 - self.playlist.suggestions.len();
        for _ in 0..missing {
            self.music_library.pick_random().map(|id| {
                self.playlist.suggestions.push_back((id, 0));
            });
        }
    }

    pub fn next_track(self: &mut Self) {
        let next_track = {
            self.hydrate();
            let next_track_id = self.playlist.next_track_id();
            if self.playlist.mode == Mode::AUTO {
                self.playlist.suggestions.clear();
            }
            self.hydrate();

            let next_track: Option<(Uuid, Track, DateTime<Utc>)> =
                next_track_id.and_then(|track_id| {
                    self.music_library
                        .get_track(track_id)
                        .map(|track| (track_id, track, Utc::now()))
                });
            self.current_track = next_track.clone();
            next_track
        };

        if let Some((track_id, next_track, start_time)) = next_track {
            println!("Playing: {}", track_id);
            self.current_player = self
                .music_library
                .get_track(track_id)
                .map(|track| Player::new(track));

            self.server
                .broadcast(ServerToClient::CurrentTrackUpdate(Some(
                    dto::CurrentTrack {
                        start_time,
                        track_id,
                        duration: next_track.tech.duration,
                    },
                )));

            self.server
                .broadcast(ServerToClient::PlaylistUpdate(dto::IdPlaylist {
                    playlist: self.playlist.playlist.clone(),
                    next_up: self.playlist.next_up.clone(),
                    suggestions: self.playlist.suggestions.clone(),
                }));
        }
    }

    pub fn playlist_update(self: &mut Self, playlist_update: dto::IdPlaylist) {
        self.playlist.playlist = playlist_update.playlist.clone();
        self.playlist.next_up = playlist_update.next_up.clone();
        self.server
            .broadcast(ServerToClient::PlaylistUpdate(playlist_update));
    }

    pub fn mode_update(self: &mut Self, mode_update: dto::Mode) {
        self.server
            .broadcast(ServerToClient::ModeUpdate(mode_update));
    }

    pub fn on_connection(self: &mut Self, client_id: Uuid) {
        let library_infos = self
            .music_library
            .library
            .iter()
            .map(|(id, info)| dto::Track {
                id: *id,
                info: info.clone(),
            })
            .collect();

        self.server.send(
            client_id,
            ServerToClient::LibraryUpdate(dto::InfoLibrary {
                library: library_infos,
            }),
        );
        self.server.send(
            client_id,
            ServerToClient::CurrentTrackUpdate(self.current_track.as_ref().map(
                |(track_id, track, start_time)| dto::CurrentTrack {
                    start_time: *start_time,
                    track_id: *track_id,
                    duration: track.tech.duration,
                },
            )),
        );

        self.server
            .broadcast(ServerToClient::PlaylistUpdate(dto::IdPlaylist {
                playlist: self.playlist.playlist.clone(),
                next_up: self.playlist.next_up.clone(),
                suggestions: self.playlist.suggestions.clone(),
            }));
    }
}

#[derive(Clone)]
pub struct Middleware {
    inner: Arc<Mutex<MiddlewareInner>>,
}
impl Middleware {
    pub fn new(music_dir: PathBuf, bind: String) -> Self {
        let music_library = Arc::new(Library::new(music_dir));
        let playlist = Playlist::new();
        let current_player = None;
        let server = Server::new(bind);
        let running = Arc::new(AtomicBool::new(false));

        let inner = Arc::new(Mutex::new(MiddlewareInner {
            music_library,
            playlist,
            current_player,
            current_track: None,
            server,
            running,
            server_handle: None,
            play_handle: None,
            processing_handle: None,
        }));

        Middleware { inner: inner }
    }

    pub fn start(self: &mut Self) {
        let (server_handle, processing_handle) = self.start_server();
        let play_handle = self.play();

        let mut inner_guard = self.inner.lock().unwrap();
        inner_guard.server_handle = Some(server_handle);
        inner_guard.play_handle = Some(play_handle);
        inner_guard.processing_handle = Some(processing_handle);
        inner_guard.running.store(true, Ordering::Relaxed);
    }

    pub fn stop(self: &mut Self) {
        println!("Graceful shutdown");

        let (server_handle, play_handle) = {
            let mut inner_guard = self.inner.lock().unwrap();
            inner_guard.running.store(false, Ordering::Relaxed);
            (
                inner_guard.server_handle.take(),
                inner_guard.play_handle.take(),
            )
        };

        if let Some(server_handle) = server_handle {
            server_handle.join().unwrap();
        }

        if let Some(play_handle) = play_handle {
            play_handle.join().unwrap();
        }

        println!("Graceful shutdown finished");
    }

    fn start_server(self: &Self) -> (JoinHandle<()>, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

        let inner = self.inner.clone();
        let server_handle = thread::spawn(move || {
            let (mut server, running) = {
                let inner_guard = inner.lock().unwrap();
                (inner_guard.server.clone(), inner_guard.running.clone())
            };

            let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
            rt.block_on(async move {
                server.start_server(running, tx).await;
            });
        });

        let inner = self.inner.clone();
        let processing_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
            rt.block_on(async move {
                while let Some(server_event) = rx.recv().await {
                    let mut inner_guard = inner.lock().unwrap();
                    match server_event {
                        ServerEvent::ClientConnected(uuid) => inner_guard.on_connection(uuid),
                        ServerEvent::ClientDisconnected(_) => {}
                        ServerEvent::Message(client_message) => match client_message.payload {
                            ClientToServer::PlaylistUpdate(id_playlist_update) => {
                                inner_guard.playlist_update(id_playlist_update)
                            }
                            ClientToServer::ModeUpdate(mode_update) => {
                                inner_guard.mode_update(mode_update)
                            }
                        },
                    }
                }
            });
        });

        (server_handle, processing_handle)
    }

    fn play(self: &Self) -> JoinHandle<()> {
        let inner = self.inner.clone();
        thread::spawn(move || {
            let running = {
                let inner_guard = inner.lock().unwrap();
                inner_guard.running.clone()
            };
            while running.load(Ordering::Relaxed) {
                let inner = inner.clone();
                let play_handle = {
                    let mut inner_guard = inner.lock().unwrap();
                    inner_guard.next_track();
                    if let Some(player) = inner_guard.current_player.take() {
                        player.play(inner_guard.running.clone())
                    } else {
                        thread::spawn(|| {
                            thread::sleep(Duration::from_secs(1));
                        })
                    }
                };

                play_handle.join().expect("could not play");
            }
            println!("Stopped playing");
        })
    }
}
