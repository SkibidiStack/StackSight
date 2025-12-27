use dioxus::prelude::*;

#[component]
pub fn LogsViewer() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Logs" }
            pre { style: "height: 180px; overflow: auto; background: #0a111b; padding: 12px; border-radius: 10px; border: 1px solid var(--border);",
                "[12:01] api-server: listening on :8080\n[12:00] worker: job complete"
            }
        }
    }
}
