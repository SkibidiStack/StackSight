use dioxus::prelude::*;

#[component]
pub fn FormComponents() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Form Controls" }
            p { class: "muted", "Inputs, dropdowns, toggles, and sliders for flows." }
        }
    }
}
