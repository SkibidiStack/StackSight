use crate::state::SetupConfig;
use dioxus::prelude::*;

#[component]
pub fn SummaryStep(config: Signal<SetupConfig>) -> Element {
    let cfg = config();

    rsx! {
        div { class: "wizard-step summary-step",
            h2 { "Setup Summary" }
            p { "Review your configuration before completing setup." }

            div { class: "summary-grid",
                div { class: "summary-card",
                    h3 { "🐳 Docker" }
                    if let Some(docker_path) = &cfg.docker_path {
                        div { class: "config-item",
                            label { "Path:" }
                            code { "{docker_path}" }
                            div { class: "status-badge success", "✓ Configured" }
                        }
                    } else {
                        div { class: "config-item",
                            div { class: "status-badge warning", "⚠ Not configured" }
                            p { "You can configure Docker later in Settings" }
                        }
                    }
                }

                div { class: "summary-card",
                    h3 { "📁 Virtual Environments" }
                    div { class: "config-item",
                        label { "Base Path:" }
                        code { "{cfg.virtualenv_base_path}" }
                        div { class: "status-badge success", "✓ Configured" }
                    }
                    p { class: "help-text",
                        "All virtual environments will be created here"
                    }
                }

                div { class: "summary-card",
                    h3 { "🛠️ Development Tools" }
                    if cfg.python_paths.is_empty() && cfg.node_paths.is_empty() &&
                       cfg.rust_path.is_none() && cfg.go_path.is_none() &&
                       cfg.java_path.is_none() && cfg.dotnet_path.is_none() {
                        div { class: "config-item",
                            div { class: "status-badge info", "None detected" }
                            p { "Install tools and they'll be detected automatically" }
                        }
                    } else {
                        ul { class: "tools-list",
                            if !cfg.python_paths.is_empty() {
                                li { "✓ Python ({cfg.python_paths.len()} version(s))" }
                            }
                            if !cfg.node_paths.is_empty() {
                                li { "✓ Node.js ({cfg.node_paths.len()} version(s))" }
                            }
                            if cfg.rust_path.is_some() {
                                li { "✓ Rust" }
                            }
                            if cfg.go_path.is_some() {
                                li { "✓ Go" }
                            }
                            if cfg.java_path.is_some() {
                                li { "✓ Java" }
                            }
                            if cfg.dotnet_path.is_some() {
                                li { "✓ .NET" }
                            }
                        }
                    }
                }
            }

            div { class: "next-steps",
                h3 { "What's Next?" }
                ul {
                    li { "✅ Your configuration will be saved automatically" }
                    li { "🚀 You'll be taken to the main dashboard" }
                    li { "📖 Check out the documentation for advanced features" }
                    li { "⚙️ You can modify these settings anytime in Preferences" }
                }
            }

            div { class: "ready-message",
                "🎉 You're all set! Click \"Complete Setup\" to start using StackSight."
            }
        }
    }
}
