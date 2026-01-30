use dioxus::prelude::*;

#[component]
pub fn Header(title: String) -> Element {
    rsx! {
        header { class: "topbar",
            div { class: "topbar-title", "{title}" }
            div { class: "topbar-actions",
                div { class: "pill", "Realtime 2s" }
                button { class: "btn primary", "New action" }
            }
        }
    }
}
