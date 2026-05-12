use dioxus::prelude::*;

use radio_app::state::AppState;

#[component]
pub fn ProgressBar() -> Element {
    let state = use_context::<AppState>();

    let progress = use_memo(move || match *state.current_track_start_time.read() {
        Some(start_time) => match *state.current_track_duration.read() {
            Some(duration) => {
                let elapsed = state.now.read().signed_duration_since(start_time);
                let progress = 100.0 * elapsed.as_seconds_f64() / duration.as_secs_f64();
                Some(progress)
            }
            None => None,
        },
        None => None,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/progress_bar.css") }
        match progress.read().as_ref() {
            Some(progress) => rsx! {
                div {
                    class: "progress-bar",
                    span { style: "width: {progress}%;" }
                }
            },
            None => rsx! {
            }
        }
    }
}
