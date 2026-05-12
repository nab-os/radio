use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use chrono::Utc;
use dioxus::prelude::*;

use radio_app::{connect, state::AppState};

mod components;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut state = use_context_provider(AppState::new);

    let background_image = asset!("/assets/background.png");

    use_future(move || async move {
        loop {
            sleep(200).await;
            state.now.set(Utc::now());
        }
    });

    use_hook(|| {
        let running = Arc::new(AtomicBool::new(true));
        connect(state, "ws://localhost:8081".to_string(), running);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/style.css") }
        div { class: "app",
            background_image: "url({background_image})",
            components::Header {}
            main {
                components::NowPlaying {}
                components::UpNext {}
            }
            components::Footer {}
        }
    }
}

async fn sleep(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}
