use dioxus::prelude::*;

#[component]
pub fn VersionSelector() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Versions" }
            p { class: "muted", "Pin language versions and toolchains." }
        }
    }
}
