use dioxus::prelude::*;

#[component]
pub fn FileBrowser() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "File Browser" }
            p { class: "muted", "Cross-platform picker for projects and mounts." }
        }
    }
}
