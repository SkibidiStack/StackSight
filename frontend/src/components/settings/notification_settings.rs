use dioxus::prelude::*;

#[component]
pub fn NotificationSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Notifications" }
            p { class: "muted", "Configure in-app and system alerts." }
            div { class: "chip", "Toast + system" }
        }
    }
}
