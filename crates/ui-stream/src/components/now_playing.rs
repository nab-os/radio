use dioxus::prelude::*;

use radio_app::state::AppState;

use crate::components::{ProgressBar, Track};

#[component]
pub fn NowPlaying() -> Element {
    let state = use_context::<AppState>();

    let track = use_memo(move || {
        let id = *state.current_track_id.read();
        id.and_then(|id| state.library.read().get(&id).cloned())
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/now-playing.css") }
        section { class: "now-playing",
            match track.read().as_ref() {
                Some(info) => rsx! {
                    Track { info: info.clone(), main: true }
                    ProgressBar {}
                },
                None => rsx! {
                    p { class: "now-playing--empty", "Waiting for next track…" }
                }
            }
        }
    }
}
