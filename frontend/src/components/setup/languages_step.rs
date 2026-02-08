use dioxus::prelude::*;
use crate::state::{SetupConfig, PathConfig};

#[component]
pub fn LanguagesStep(config: Signal<SetupConfig>) -> Element {
    let mut detecting = use_signal(|| false);
    let mut detected_tools: Signal<Vec<(String, String, bool)>> = use_signal(Vec::new);

    let mut detect_languages = move |_| {
        detecting.set(true);
        spawn(async move {
            let mut tools = Vec::new();
            
            // Detect Python
            if let Ok(output) = tokio::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).to_string();
                    tools.push((String::from("Python"), version.trim().to_string(), true));
                }
            }

            // Detect Node.js
            if let Ok(output) = tokio::process::Command::new("node")
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).to_string();
                    tools.push((String::from("Node.js"), version.trim().to_string(), true));
                }
            }

            // Detect Rust
            if let Ok(output) = tokio::process::Command::new("rustc")
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).to_string();
                    tools.push((String::from("Rust"), version.trim().to_string(), true));
                }
            }

            // Detect Go
            if let Ok(output) = tokio::process::Command::new("go")
                .arg("version")
                .output()
                .await
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).to_string();
                    tools.push((String::from("Go"), version.trim().to_string(), true));
                }
            }

            // Detect Java
            if let Ok(output) = tokio::process::Command::new("java")
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).to_string();
                    tools.push((String::from("Java"), version.trim().to_string(), true));
                }
            }

            detected_tools.set(tools);
            detecting.set(false);
        });
    };

    // Auto-detect on mount
    use_effect(move || {
        detect_languages(());
    });

    rsx! {
        div { class: "wizard-step languages-step",
            h2 { "Language Tools Detection" }
            p { "We'll detect installed programming languages and tools." }

            div { class: "detection-panel",
                if detecting() {
                    div { class: "spinner", "Detecting installed tools..." }
                } else {
                    div { class: "detected-tools",
                        if detected_tools().is_empty() {
                            div { class: "no-tools",
                                "⚠️ No development tools detected yet."
                                p { "You can install them later and re-run the setup." }
                            }
                        } else {
                            h4 { "✓ Detected Tools:" }
                            div { class: "tools-grid",
                                for (name , version , _detected) in detected_tools().iter() {
                                    div { class: "tool-card detected",
                                        div { class: "tool-icon",
                                            match name.as_str() {
                                                "Python" => "🐍",
                                                "Node.js" => "🟢",
                                                "Rust" => "🦀",
                                                "Go" => "🐹",
                                                "Java" => "☕",
                                                _ => "📦",
                                            }
                                        }
                                        h4 { "{name}" }
                                        p { class: "version", "{version}" }
                                        div { class: "status-badge success", "✓ Detected" }
                                    }
                                }
                            }
                        }
                    }
                }

                button {
                    class: "btn btn-outline",
                    onclick: move |_| detect_languages(()),
                    disabled: detecting(),
                    "🔄 Re-scan Tools"
                }
            }

            div { class: "tools-help",
                h4 { "Missing a language?" }
                div { class: "install-guides",
                    details {
                        summary { "🐍 Python" }
                        ul {
                            li {
                                strong { "Windows/macOS: " }
                                a { href: "https://www.python.org/downloads/", "Download from python.org" }
                            }
                            li {
                                strong { "Linux: " }
                                code { "sudo apt install python3 python3-pip python3-venv" }
                            }
                        }
                    }

                    details {
                        summary { "🟢 Node.js" }
                        ul {
                            li {
                                strong { "All platforms: " }
                                a { href: "https://nodejs.org/", "Download from nodejs.org" }
                            }
                            li {
                                strong { "Or use nvm: " }
                                a { href: "https://github.com/nvm-sh/nvm", "Node Version Manager" }
                            }
                        }
                    }

                    details {
                        summary { "🦀 Rust" }
                        ul {
                            li {
                                strong { "All platforms: " }
                                code { "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" }
                            }
                        }
                    }

                    details {
                        summary { "🐹 Go" }
                        ul {
                            li {
                                strong { "Download: " }
                                a { href: "https://go.dev/dl/", "go.dev/dl" }
                            }
                        }
                    }

                    details {
                        summary { "☕ Java" }
                        ul {
                            li {
                                strong { "OpenJDK: " }
                                a { href: "https://adoptium.net/", "Adoptium (recommended)" }
                            }
                            li {
                                strong { "Or Oracle: " }
                                a { href: "https://www.oracle.com/java/technologies/downloads/", "Oracle JDK" }
                            }
                        }
                    }
                }

                p { class: "note",
                    "💡 You don't need all of these. Install only what you use for development."
                }
            }
        }
    }
}
