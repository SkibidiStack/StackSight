use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn NetworkManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let networks = snapshot.docker.networks.clone();
    drop(snapshot);

    rsx! {
        div { class: "panel",
            h2 { "Networks" }
            div { class: "muted", "Inspect and connect networks." }
            if networks.is_empty() {
                div { class: "muted", "No networks found." }
            } else {
                ul { style: "list-style: none; padding: 0; margin: 12px 0 0; display: flex; flex-direction: column; gap: 8px;",
                    for network in networks.iter() {
                        li { class: "nav-link",
                            span { "{network.name}" }
                            span { class: "muted", "{network.driver}" }
                        }
                    }
                }
            }
        }
    }
}
