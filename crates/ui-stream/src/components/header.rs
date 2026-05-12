use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;

use radio_app::state::AppState;

#[component]
pub fn Header() -> Element {
    let state = use_context::<AppState>();
    let connected = state.connected.read();
    let server_uptime = use_memo(move || -> Option<String> {
        (*state.server_start_time.read()).map(|server_start_time| {
            let now = *state.now.read();
            let uptime = now.signed_duration_since(server_start_time);

            let uptime_secs = uptime.as_seconds_f64();
            let uptime_mins = uptime_secs / 60.0;

            let secs = (uptime_secs % 60.0) as u32;
            let mins = (uptime_mins % 60.0) as u32;
            let hours = (uptime_mins / 60.0) as u32;
            let uptime_str = format!("{hours:00}h {mins:00}m {secs:00}s");
            uptime_str.to_string()
        })
    });

    let time = use_memo(move || -> String {
        let now = *state.now.read();
        let converted: DateTime<Local> = DateTime::from(now);
        let time_str = converted.format("%H:%M");
        time_str.to_string()
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/header.css") }
        header { class: "header",
            h1 { "Radio Sasha" span { "v0.2.0" } }
            h2 { "{time}" }
            span {
                class: if *connected { "status status--connected" } else { "status status--disconnected" },
                if *connected {
                    "● live ",
                    match server_uptime.as_ref() {
                        Some(uptime) => rsx! {
                            span {
                                class: "status__uptime",
                                "for {uptime}"
                            }
                        },
                        None => rsx! {}
                    }
                } else {
                    "○ offline"
                }
            }
        }
    }
}
