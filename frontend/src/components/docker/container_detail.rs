use dioxus::prelude::*;

#[component]
pub fn ContainerDetail() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Container Detail" }
            p { class: "muted", "Select a container to inspect configuration, mounts, and live logs." }
        }
    }
}
