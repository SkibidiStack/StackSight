use dioxus::prelude::*;

#[component]
pub fn VolumeManager() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Volumes" }
            div { class: "muted", "Persistent data volumes." }
            div { class: "chip", "postgres-data" }
        }
    }
}
