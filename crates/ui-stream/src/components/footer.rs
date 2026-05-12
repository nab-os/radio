use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/footer.css") }
        footer { class: "footer",
            h3 { "Tips: from Twitch, you can click on a music on the right to vote for it" }
        }
    }
}
