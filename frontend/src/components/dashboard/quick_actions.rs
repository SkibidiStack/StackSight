use dioxus::prelude::*;

#[component]
pub fn QuickActions() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Quick Actions" }
            div { class: "action-bar",
                button { class: "btn primary", "+ New container" }
                button { class: "btn", "+ New environment" }
                button { class: "btn", "Open project" }
            }
        }
    }
}
