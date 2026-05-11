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
    net::{
        dto,
        protocol::{ClientToServer, ServerEvent, ServerToClient},
    },
    playlist::Playlist,
};

use crate::{audio::player::Player, library::Library, net::Server};

pub struct MiddlewareInner {
    pub music_library: Arc<Library>,
    pub playlist: Playlist,
    pub current_player: Option<Player>,
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
                self.playlist.next_up.push(id);
            });
        }
        let missing = 3 - self.playlist.suggestions.len();
        for _ in 0..missing {
            self.music_library.pick_random().map(|id| {
                self.playlist.suggestions.push((id, 0));
            });
        }
    }

    pub async fn next_track(self: &mut Self) {
        self.hydrate();
        let next_track = self.playlist.next_track();
        self.hydrate();

        if let Some(next_track) = next_track {
            self.current_player = self
                .music_library
                .get_track(next_track)
                .map(|track| Player::new(track));

            self.server
                .broadcast(ServerToClient::CurrentTrackUpdate(dto::CurrentTrack {
                    start_time: Utc::now(),
                    track_id: next_track,
                }))
                .await;
        }
    }

    pub async fn playlist_update(self: &mut Self, playlist_update: dto::IdPlaylist) {
        self.playlist.playlist = playlist_update.playlist.clone();
        self.playlist.next_up = playlist_update.next_up.clone();
        self.server
            .broadcast(ServerToClient::PlaylistUpdate(playlist_update))
            .await;
    }

    pub async fn mode_update(self: &mut Self, mode_update: dto::Mode) {
        self.server
            .broadcast(ServerToClient::ModeUpdate(mode_update))
            .await;
    }

    pub async fn on_connection(self: &mut Self, client_id: Uuid) {
        let library_infos = self
            .music_library
            .library
            .iter()
            .map(|(id, info)| dto::Track {
                id: *id,
                info: info.clone(),
            })
            .collect();

        self.server
            .send(
                client_id,
                ServerToClient::LibraryUpdate(dto::InfoLibrary {
                    library: library_infos,
                }),
            )
            .await;
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
                        ServerEvent::ClientConnected(uuid) => inner_guard.on_connection(uuid).await,
                        ServerEvent::ClientDisconnected(_) => {}
                        ServerEvent::Message(client_message) => match client_message.payload {
                            ClientToServer::PlaylistUpdate(id_playlist_update) => {
                                inner_guard.playlist_update(id_playlist_update).await
                            }
                            ClientToServer::ModeUpdate(mode_update) => {
                                inner_guard.mode_update(mode_update).await
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
