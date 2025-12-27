use dioxus::prelude::*;

#[component]
pub fn EnvironmentList() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Environments" }
            ul { style: "list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px;",
                li { class: "nav-link", span { "py-data-lab" } span { class: "muted", "python 3.11" } }
                li { class: "nav-link", span { "node-services" } span { class: "muted", "node 20" } }
                li { class: "nav-link", span { "rust-tools" } span { class: "muted", "stable" } }
            }
        }
    }
}
