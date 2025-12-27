use dioxus::prelude::*;

#[component]
pub fn TerminalPanel() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Terminal" }
            pre { style: "height: 140px; overflow: auto; background: #0a111b; padding: 10px; border-radius: 10px; border: 1px solid var(--border);",
                "$ devenv --help\n$ docker ps"
            }
        }
    }
}
