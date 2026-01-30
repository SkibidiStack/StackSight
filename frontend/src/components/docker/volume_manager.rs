use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn VolumeManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let volumes = snapshot.docker.volumes.clone();
    drop(snapshot);

    rsx! {
        div { class: "panel",
            h2 { "Volumes" }
            div { class: "muted", "Persistent data volumes." }
            if volumes.is_empty() {
                div { class: "muted", "No volumes found." }
            } else {
                ul { style: "list-style: none; padding: 0; margin: 12px 0 0; display: flex; flex-direction: column; gap: 8px;",
                    for volume in volumes.iter() {
                        li { class: "nav-link",
                            span { "{volume.name}" }
                            span { class: "muted", "{volume.driver}" }
                        }
                    }
                }
            }
        }
    }
}
