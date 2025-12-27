use dioxus::prelude::*;

#[component]
pub fn HistoricalData() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Historical Data" }
            p { class: "muted", "Long-term trends and exports will appear here." }
        }
    }
}
