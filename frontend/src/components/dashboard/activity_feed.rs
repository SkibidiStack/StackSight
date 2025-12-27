use dioxus::prelude::*;

#[component]
pub fn ActivityFeed() -> Element {
    let events = [
        ("Container api-server restarted", "12s ago"),
        ("Pulled image redis:7", "3m ago"),
        ("Created env py-data-lab", "6m ago"),
    ];

    rsx! {
        div { class: "panel",
            h2 { "Activity" }
            ul { style: "list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px;",
                for (message, time) in events.iter() {
                    li { class: "nav-link", 
                        span { "{message}" }
                        span { class: "muted", "{time}" }
                    }
                }
            }
        }
    }
}
