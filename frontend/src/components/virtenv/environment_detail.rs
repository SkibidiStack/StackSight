use dioxus::prelude::*;

#[component]
pub fn EnvironmentDetail() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Environment Detail" }
            p { class: "muted", "Inspect interpreter, packages, and resource usage." }
        }
    }
}
