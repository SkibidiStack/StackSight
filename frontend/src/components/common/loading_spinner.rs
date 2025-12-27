use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Loading" }
            p { class: "muted", "Shows progress for background work." }
        }
    }
}
