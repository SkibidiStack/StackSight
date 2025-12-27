use dioxus::prelude::*;

#[component]
pub fn ProcessMonitor() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Processes" }
            ul { style: "list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 6px;",
                li { class: "nav-link", span { "tauri-backend" } span { class: "muted", "42 MB" } }
                li { class: "nav-link", span { "docker" } span { class: "muted", "1.2 GB" } }
            }
        }
    }
}
