use dioxus::prelude::*;

#[component]
pub fn NotificationToast() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Notifications" }
            p { class: "muted", "Toast stack for async operations." }
        }
    }
}
