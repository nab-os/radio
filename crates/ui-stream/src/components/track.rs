use base64::{engine::general_purpose::STANDARD, Engine};
use dioxus::prelude::*;
use radio_core::data::track;

#[component]
pub fn Track(info: track::Info, main: bool) -> Element {
    let cover_src = use_memo(use_reactive(&info.cover, |cover| {
        cover.as_ref().map(|bytes| {
            let mime = detect_mime(bytes);
            let b64 = STANDARD.encode(bytes);
            format!("data:{mime};base64,{b64}")
        })
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/track.css") }
        div { class: if main {"track track--main"} else {"track"},
            match cover_src.read().as_ref() {
                Some(src) => rsx! {
                    img { class: "track__cover", src: "{src}" }
                },
                None => rsx! {
                    div { class: "track__cover track__cover--missing" }
                }
            }
            div {
                h2 { class: "track__title", "{info.title}" }
                p {
                    span { class: "track__artist", "{info.artist}" }
                    " − "
                    span { class: "track__album", "{info.album}" }
                }
            }
        }
    }
}

fn detect_mime(bytes: &[u8]) -> &'static str {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => "image/jpeg",
        [0x89, 0x50, 0x4E, 0x47, ..] => "image/png",
        [0x47, 0x49, 0x46, ..] => "image/gif",
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50, ..] => "image/webp",
        _ => "application/octet-stream",
    }
}
