use dioxus::prelude::*;

#[component]
pub fn WelcomeStep() -> Element {
    rsx! {
        div { class: "wizard-step welcome-step",
            div { class: "welcome-content",
                h2 { "Welcome to StackSight DevEnv Manager!" }
                
                p { class: "lead",
                    "This setup wizard will help you configure StackSight for your system. "
                    "We'll detect your development tools and set up optimal paths."
                }

                div { class: "features-grid",
                    div { class: "feature-card",
                        div { class: "feature-icon", "🐳" }
                        h3 { "Docker Management" }
                        p { "Control containers, images, networks, and volumes with ease" }
                    }

                    div { class: "feature-card",
                        div { class: "feature-icon", "🐍" }
                        h3 { "Virtual Environments" }
                        p { "Create and manage Python, Node.js, Rust, and more" }
                    }

                    div { class: "feature-card",
                        div { class: "feature-icon", "📊" }
                        h3 { "System Monitoring" }
                        p { "Track CPU, memory, disk, and network usage in real-time" }
                    }

                    div { class: "feature-card",
                        div { class: "feature-icon", "⚙️" }
                        h3 { "Project Management" }
                        p { "Associate environments with projects and automate workflows" }
                    }
                }

                div { class: "setup-time",
                    "⏱️ This will take about 2-3 minutes"
                }

                div { class: "privacy-note",
                    "🔒 All configuration stays on your machine. No data is sent externally."
                }
            }
        }
    }
}
