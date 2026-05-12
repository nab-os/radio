use dioxus::prelude::*;

use radio_app::state::AppState;

#[component]
pub fn Header() -> Element {
    let state = use_context::<AppState>();
    let connected = state.connected.read();
    // let uptime = state.uptime.read();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/header.css") }
        header { class: "header",
            h1 { "Radio Sasha" span { "v0.2.0" } }
            span {
                class: if *connected { "status status--connected" } else { "status status--disconnected" },
                if *connected {
                    "● live "
                    span {
                        class: "status__uptime",
                        "for 0h00m00s"
                    }
                } else {
                    "○ offline"
                }
            }
        }
    }
}
