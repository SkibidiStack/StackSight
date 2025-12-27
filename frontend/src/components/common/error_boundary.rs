use dioxus::prelude::*;

#[component]
pub fn ErrorBoundary(children: Element) -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Error Boundary" }
            div { class: "muted", "Wrap components to prevent cascading failures." }
            div { {children} }
        }
    }
}
