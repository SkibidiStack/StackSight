use crate::state::SetupConfig;
use dioxus::prelude::*;

#[component]
pub fn SetupWizard(on_complete: EventHandler<SetupConfig>) -> Element {
    let mut current_step = use_signal(|| 0);
    let config = use_signal(SetupConfig::default);

    let steps = vec![
        "Welcome",
        "Docker Setup",
        "Environment Paths",
        "Language Tools",
        "Summary",
    ];

    let can_go_next = match current_step() {
        0 => true, // Welcome
        1 => true, // Docker (optional)
        2 => !config().virtualenv_base_path.is_empty(),
        3 => true, // Languages (optional)
        4 => true, // Summary
        _ => false,
    };

    rsx! {
        div { class: "setup-wizard",
            // Header
            div { class: "wizard-header",
                div { class: "logo-container",
                    img {
                        src: "assets/icon.png",
                        alt: "StackSight Logo",
                        class: "wizard-logo"
                    }
                    h1 { "Welcome to StackSight" }
                    p { class: "subtitle", "Let's get your development environment configured" }
                }

                // Progress indicator
                div { class: "wizard-progress",
                    for (i , step_name) in steps.iter().enumerate() {
                        div {
                            class: if i == current_step() { "step active" }
                                   else if i < current_step() { "step completed" }
                                   else { "step" },
                            div { class: "step-number", "{i + 1}" }
                            div { class: "step-label", "{step_name}" }
                        }
                    }
                }
            }

            // Content
            div { class: "wizard-content",
                match current_step() {
                    0 => rsx! { super::welcome_step::WelcomeStep {} },
                    1 => rsx! { super::docker_step::DockerStep { config: config.clone() } },
                    2 => rsx! { super::paths_step::PathsStep { config: config.clone() } },
                    3 => rsx! { super::languages_step::LanguagesStep { config: config.clone() } },
                    4 => rsx! { super::summary_step::SummaryStep { config: config.clone() } },
                    _ => rsx! { div { "Invalid step" } },
                }
            }

            // Footer navigation
            div { class: "wizard-footer",
                button {
                    class: "btn btn-secondary",
                    disabled: current_step() == 0,
                    onclick: move |_| {
                        if current_step() > 0 {
                            current_step.set(current_step() - 1);
                        }
                    },
                    "← Back"
                }

                div { class: "step-indicator",
                    "Step {current_step() + 1} of {steps.len()}"
                }

                if current_step() < steps.len() - 1 {
                    button {
                        class: "btn btn-primary",
                        disabled: !can_go_next,
                        onclick: move |_| current_step.set(current_step() + 1),
                        "Next →"
                    }
                } else {
                    button {
                        class: "btn btn-success",
                        onclick: move |_| {
                            let mut final_config = config();
                            final_config.completed = true;

                            // Save configuration
                            if let Err(e) = final_config.save() {
                                tracing::error!("Failed to save setup config: {}", e);
                            }

                            on_complete.call(final_config);
                        },
                        "Complete Setup ✓"
                    }
                }
            }
        }

        style { {include_str!("wizard.css")} }
    }
}
