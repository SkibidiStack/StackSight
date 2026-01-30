use dioxus::prelude::*;

#[component]
pub fn ActivityFeed() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Activity" }
            div { class: "muted", "No activity recorded yet." }
        }
    }
}
