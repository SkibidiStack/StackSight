use dioxus::prelude::*;

#[component]
pub fn SearchInput() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Search" }
            input { r#type: "text", placeholder: "Search containers, images, envs" }
        }
    }
}
