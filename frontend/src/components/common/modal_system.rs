use dioxus::prelude::*;

#[component]
pub fn ModalSystem() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Modal" }
            p { class: "muted", "Re-usable dialogs for confirmations and forms." }
        }
    }
}
