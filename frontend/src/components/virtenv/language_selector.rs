use dioxus::prelude::*;

#[component]
pub fn LanguageSelector() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Languages" }
            div { class: "muted", "Pick language and toolchain." }
            div { class: "action-bar",
                button { class: "btn", "Python 3.11" }
                button { class: "btn", "Node 20" }
                button { class: "btn", "Rust stable" }
            }
        }
    }
}
