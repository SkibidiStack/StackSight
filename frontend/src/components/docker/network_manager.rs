use dioxus::prelude::*;

#[component]
pub fn NetworkManager() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Networks" }
            div { class: "muted", "Inspect and connect networks." }
            ul { style: "list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px;",
                li { class: "chip", "bridge" }
                li { class: "chip", "frontend" }
            }
        }
    }
}
