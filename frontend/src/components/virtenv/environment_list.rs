use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn EnvironmentList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let total = snapshot.virtenv.environments;
    let active = snapshot.virtenv.active;
    drop(snapshot);

    let summary = if total == 0 {
        "No environments detected".to_string()
    } else if active == 0 {
        format!("{total} environments")
    } else {
        format!("{active} active / {total} total")
    };

    rsx! {
        div { class: "panel",
            h2 { "Environments" }
            div { class: "muted", "{summary}" }
        }
    }
}
