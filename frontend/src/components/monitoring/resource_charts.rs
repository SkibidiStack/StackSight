use dioxus::prelude::*;

#[component]
pub fn ResourceCharts() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Resource Charts" }
            p { class: "muted", "Canvas-based charts will render here with zoom and pan." }
        }
    }
}
