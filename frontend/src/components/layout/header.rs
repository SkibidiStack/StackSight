use dioxus::prelude::*;

#[component]
pub fn Header(title: String) -> Element {
    rsx! {
        header { class: "header",
            h1 { "{title}" }
            div { class: "action-bar",
                div { class: "badge", "Realtime 2s" }
                button { class: "btn primary", "New action" }
            }
        }
    }
}
