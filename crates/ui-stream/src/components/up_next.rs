use dioxus::prelude::*;

use radio_app::state::AppState;
use radio_core::{data::track, playlist::Mode};

use crate::components::Track;

#[component]
pub fn UpNext() -> Element {
    let state = use_context::<AppState>();
    let mode = state.mode.read();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/up-next.css") }
        section { class: "up-next",
            match *mode {
                Mode::MANUAL => rsx! { ManualQueue {} },
                Mode::AUTO => rsx! { AutoVoting {} },
            }
        }
    }
}

#[component]
fn ManualQueue() -> Element {
    let state = use_context::<AppState>();
    let track_infos = use_memo(move || {
        let library = state.library.read();
        state
            .next_up_ids
            .read()
            .iter()
            .filter_map(|id| library.get(id).cloned())
            .collect::<Vec<track::Info>>()
    });

    rsx! {
        h3 { "Up next" }
        ul {
            if track_infos.read().is_empty() {
                li { class: "up-next__element up-next__element--empty", "Queue is empty" }
            } else {
                for (i, info) in track_infos.read().iter().enumerate() {
                    li { key: "{i}",
                        class: "up-next__element",
                        span { class: "position", "{i + 1}" }
                        Track { info: info.clone(), main: false }
                    }
                }
            }
        }
    }
}

#[component]
fn AutoVoting() -> Element {
    let state = use_context::<AppState>();

    let suggestions = use_memo(move || {
        let library = state.library.read();
        let mut resolved: Vec<(track::Info, u32)> = state
            .suggestions
            .read()
            .iter()
            .filter_map(|(id, votes)| library.get(id).cloned().map(|info| (info, *votes)))
            .collect();
        resolved.sort_by(|a, b| b.1.cmp(&a.1));
        resolved
    });

    let leader_votes = suggestions.read().first().map(|(_, v)| *v).unwrap_or(0);

    rsx! {
        h3 { "Vote for the next track" }
        ul {
            if suggestions.read().is_empty() {
                li { class: "up-next__element up-next__element--empty", "Generating suggestions…" }
            } else {
                for (i, (info, votes)) in suggestions.read().iter().enumerate() {
                    li {
                        key: "{i}",
                        class: if *votes == leader_votes && leader_votes > 0 { "up-next__element up-next__element--leading" } else { "up-next__element" },
                        Track { info: info.clone(), main: false }
                        span { class: "up-next__element__votes", "{votes}" }
                    }
                }
            }
        }
    }
}
