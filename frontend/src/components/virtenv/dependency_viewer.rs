use dioxus::prelude::*;

#[component]
pub fn DependencyViewer() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Dependency Graph" }
            p { class: "muted", "Visualize dependency relationships and conflicts." }
        }
    }
}
