use dioxus::prelude::*;

#[component]
pub fn PerformanceMetrics() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Performance" }
            p { class: "muted", "Compare baselines and capture snapshots." }
        }
    }
}
