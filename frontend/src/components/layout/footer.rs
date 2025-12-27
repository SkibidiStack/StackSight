use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "muted", style: "padding: 12px 18px; border-top: 1px solid var(--border);", "StackSight DevEnv Manager" }
    }
}
