use dioxus::prelude::*;

#[component]
pub fn ImageDetail() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Image Detail" }
            p { class: "muted", "View layers, tags, and security scan results." }
        }
    }
}
