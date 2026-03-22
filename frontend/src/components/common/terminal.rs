use dioxus::prelude::*;
use std::collections::VecDeque;
use std::process::Command;

#[derive(Clone, Debug, PartialEq)]
pub struct TerminalLine {
    pub content: String,
    pub line_type: TerminalLineType,
    pub timestamp: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerminalLineType {
    Command,
    Output,
    Error,
    System,
}

#[derive(Clone, Debug)]
pub struct TerminalState {
    pub lines: VecDeque<TerminalLine>,
    pub current_input: String,
    pub working_directory: String,
    pub active_env: Option<String>,
    pub command_history: VecDeque<String>,
    pub history_index: Option<usize>,
}

impl TerminalState {
    pub fn activate_environment(&mut self, env_name: String) {
        self.active_env = Some(env_name.clone());
        self.lines.push_back(TerminalLine {
            content: format!("Environment '{}' activated", env_name),
            line_type: TerminalLineType::System,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
    }

    pub fn deactivate_environment(&mut self) {
        if let Some(env) = &self.active_env {
            self.lines.push_back(TerminalLine {
                content: format!("Environment '{}' deactivated", env),
                line_type: TerminalLineType::System,
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            });
            self.active_env = None;
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        let mut lines = VecDeque::new();
        lines.push_back(TerminalLine {
            content: "Integrated Terminal - DevEnv Manager".to_string(),
            line_type: TerminalLineType::System,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
        lines.push_back(TerminalLine {
            content: "You can run any system command here".to_string(),
            line_type: TerminalLineType::System,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
        lines.push_back(TerminalLine {
            content: "Use 'env activate <name>' to activate virtual environments".to_string(),
            line_type: TerminalLineType::System,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });

        Self {
            lines,
            current_input: String::new(),
            working_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            active_env: None,
            command_history: VecDeque::new(),
            history_index: None,
        }
    }
}

#[component]
pub fn Terminal() -> Element {
    let mut terminal_state = use_signal(|| TerminalState::default());
    let mut app_state = use_context::<Signal<crate::state::AppState>>();

    let scroll_to_bottom = move || {
        println!("[TERMINAL] scroll_to_bottom called!");
        spawn(async move {
            let mut eval = document::eval(
                r#"
                const terminal = document.getElementById('terminal-content');
                if (terminal) {
                    terminal.scrollTop = terminal.scrollHeight;
                    console.log('Scrolled terminal to bottom:', terminal.scrollTop);
                }
                "#,
            );
            let _ = eval.recv::<String>().await;
        });
    };

    // Check for pending commands from other components
    use_effect(move || {
        let app_snapshot = app_state.read();
        if let Some(command) = &app_snapshot.ui.terminal_pending_command {
            let command = command.clone();
            drop(app_snapshot);

            println!("[TERMINAL] Executing pending command: {}", command);

            // Execute the command
            let mut state = terminal_state.write();

            // Add command line to terminal
            state.lines.push_back(TerminalLine {
                content: format!("$ {}", command),
                line_type: TerminalLineType::Command,
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            });

            // Execute the command
            execute_command(&mut state, command);
            drop(state);

            // Clear the pending command
            let mut app_state_write = app_state.write();
            app_state_write.ui.terminal_pending_command = None;
            drop(app_state_write);

            // Scroll to bottom
            println!("[TERMINAL] About to call scroll_to_bottom from pending command");
            scroll_to_bottom();
        }
    });

    rsx! {
        div { class: "terminal-container",
            div { class: "terminal-header",
                div { class: "terminal-title", "Terminal" }
                div { class: "terminal-info",
                    if let Some(env) = &terminal_state.read().active_env {
                        span { class: "env-indicator active", "📦 {env}" }
                    }
                    span { class: "cwd", "📁 {terminal_state.read().working_directory}" }
                }
                button {
                    class: "terminal-close-btn",
                    onclick: move |_| {
                        let mut state = app_state.write();
                        state.ui.terminal_visible = false;
                    },
                    "×"
                }
            }

            div {
                class: "terminal-content",
                id: "terminal-content",
                style: "overflow-y: auto; max-height: 400px;",
                for line in &terminal_state.read().lines {
                    TerminalLineDisplay { line: line.clone() }
                }

                // Input line at the bottom
                div { class: "terminal-input-line",
                    span { class: "prompt",
                        if let Some(env) = &terminal_state.read().active_env {
                            "({env}) $ "
                        } else {
                            "$ "
                        }
                    }
                    input {
                        class: "terminal-input",
                        r#type: "text",
                        value: "{terminal_state.read().current_input}",
                        placeholder: "Enter command...",
                        autofocus: true,
                        oninput: move |evt| {
                            let mut state = terminal_state.write();
                            state.current_input = evt.value();
                        },
                        onkeydown: move |evt| {
                            let key = evt.code();
                            let mut state = terminal_state.write();

                            match key {
                                Code::Enter => {
                                    let command = state.current_input.trim().to_string();
                                    if !command.is_empty() {
                                        println!("[TERMINAL] Executing manual command: {}", command);

                                        // Add command to history
                                        state.command_history.push_back(command.clone());
                                        if state.command_history.len() > 100 {
                                            state.command_history.pop_front();
                                        }
                                        state.history_index = None;

                                        // Add command line
                                        state.lines.push_back(TerminalLine {
                                            content: format!("$ {}", command),
                                            line_type: TerminalLineType::Command,
                                            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                                        });

                                        // Execute command
                                        execute_command(&mut state, command);

                                        // Clear input
                                        state.current_input.clear();
                                        drop(state);

                                        // Scroll to bottom
                                        println!("[TERMINAL] About to call scroll_to_bottom from manual command");
                                        scroll_to_bottom();
                                    }
                                }
                                Code::ArrowUp => {
                                    if !state.command_history.is_empty() {
                                        let new_index = match state.history_index {
                                            None => state.command_history.len() - 1,
                                            Some(i) if i > 0 => i - 1,
                                            Some(i) => i,
                                        };
                                        state.history_index = Some(new_index);
                                        state.current_input = state.command_history[new_index].clone();
                                    }
                                }
                                Code::ArrowDown => {
                                    match state.history_index {
                                        Some(i) if i < state.command_history.len() - 1 => {
                                            let new_index = i + 1;
                                            state.history_index = Some(new_index);
                                            state.current_input = state.command_history[new_index].clone();
                                        }
                                        Some(_) => {
                                            state.history_index = None;
                                            state.current_input.clear();
                                        }
                                        None => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TerminalLineDisplay(line: TerminalLine) -> Element {
    let line_class = match line.line_type {
        TerminalLineType::Command => "terminal-line command",
        TerminalLineType::Output => "terminal-line output",
        TerminalLineType::Error => "terminal-line error",
        TerminalLineType::System => "terminal-line system",
    };

    rsx! {
        div { class: "{line_class}",
            span { class: "timestamp", "{line.timestamp}" }
            span { class: "content", "{line.content}" }
        }
    }
}

fn execute_command(state: &mut TerminalState, command: String) {
    // Handle compound commands with &&
    if command.contains(" && ") {
        let commands: Vec<&str> = command.split(" && ").collect();
        for cmd in commands {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                execute_single_command(state, cmd.to_string());
            }
        }
        return;
    }

    execute_single_command(state, command);
}

fn execute_single_command(state: &mut TerminalState, command: String) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => {
            let help_text = vec![
                "Available commands:",
                "  help           - Show this help message",
                "  clear          - Clear terminal",
                "  pwd            - Show current directory",
                "  cd <path>      - Change directory",
                "  env list       - List available environments",
                "  env activate <name> - Activate environment",
                "  env deactivate - Deactivate current environment",
                "  pip <args>     - Run pip in active environment",
                "  python <args>  - Run python in active environment",
                "  node <args>    - Run node in active environment",
                "  cargo <args>   - Run cargo in active environment",
            ];

            for line in help_text {
                state.lines.push_back(TerminalLine {
                    content: line.to_string(),
                    line_type: TerminalLineType::Output,
                    timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        "clear" => {
            state.lines.clear();
            state.lines.push_back(TerminalLine {
                content: "Terminal cleared".to_string(),
                line_type: TerminalLineType::System,
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            });
        }
        "pwd" => {
            state.lines.push_back(TerminalLine {
                content: state.working_directory.clone(),
                line_type: TerminalLineType::Output,
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            });
        }
        "cd" => {
            if parts.len() > 1 {
                let new_path = parts[1];

                // Determine the target path
                let target_path = if new_path.starts_with("~") {
                    // Expand tilde for home directory
                    if let Some(home) = std::env::var_os("HOME") {
                        new_path.replacen("~", &home.to_string_lossy(), 1)
                    } else {
                        new_path.to_string()
                    }
                } else if new_path.starts_with("/") {
                    // Absolute path
                    new_path.to_string()
                } else {
                    // Relative path - resolve against current working directory
                    let current = std::path::Path::new(&state.working_directory);
                    current.join(new_path).to_string_lossy().to_string()
                };

                // Canonicalize the path to handle .. and . components
                let path = std::path::Path::new(&target_path);
                match path.canonicalize() {
                    Ok(canonical_path) => {
                        let canonical_str = canonical_path.to_string_lossy().to_string();
                        if canonical_path.is_dir() {
                            state.working_directory = canonical_str.clone();
                            state.lines.push_back(TerminalLine {
                                content: format!("Changed directory to: {}", canonical_str),
                                line_type: TerminalLineType::Output,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                        } else {
                            state.lines.push_back(TerminalLine {
                                content: format!("cd: not a directory: {}", target_path),
                                line_type: TerminalLineType::Error,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                        }
                    }
                    Err(_) => {
                        state.lines.push_back(TerminalLine {
                            content: format!("cd: no such file or directory: {}", target_path),
                            line_type: TerminalLineType::Error,
                            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                        });
                    }
                }
            } else {
                // cd with no arguments goes to home directory
                if let Some(home) = std::env::var_os("HOME") {
                    let home_str = home.to_string_lossy().to_string();
                    state.working_directory = home_str.clone();
                    state.lines.push_back(TerminalLine {
                        content: format!("Changed directory to: {}", home_str),
                        line_type: TerminalLineType::Output,
                        timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                    });
                } else {
                    state.lines.push_back(TerminalLine {
                        content: "cd: HOME not set".to_string(),
                        line_type: TerminalLineType::Error,
                        timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
            }
            return;
        }
        "env" => {
            if parts.len() > 1 {
                match parts[1] {
                    "list" => {
                        state.lines.push_back(TerminalLine {
                            content: "TODO: List environments from backend".to_string(),
                            line_type: TerminalLineType::Output,
                            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                        });
                    }
                    "activate" => {
                        if parts.len() > 2 {
                            let env_name = parts[2];
                            state.active_env = Some(env_name.to_string());
                            state.lines.push_back(TerminalLine {
                                content: format!("Activated environment: {}", env_name),
                                line_type: TerminalLineType::System,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                        } else {
                            state.lines.push_back(TerminalLine {
                                content: "Usage: env activate <name>".to_string(),
                                line_type: TerminalLineType::Error,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                        }
                    }
                    "deactivate" => {
                        if let Some(env) = &state.active_env {
                            state.lines.push_back(TerminalLine {
                                content: format!("Deactivated environment: {}", env),
                                line_type: TerminalLineType::System,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                            state.active_env = None;
                        } else {
                            state.lines.push_back(TerminalLine {
                                content: "No active environment".to_string(),
                                line_type: TerminalLineType::Error,
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            });
                        }
                    }
                    _ => {
                        state.lines.push_back(TerminalLine {
                            content: "Unknown env command. Use: list, activate <name>, deactivate"
                                .to_string(),
                            line_type: TerminalLineType::Error,
                            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                        });
                    }
                }
            } else {
                state.lines.push_back(TerminalLine {
                    content: "Usage: env <list|activate|deactivate>".to_string(),
                    line_type: TerminalLineType::Error,
                    timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        "pip" | "python" | "node" | "cargo" => {
            if let Some(env) = &state.active_env {
                state.lines.push_back(TerminalLine {
                    content: format!("TODO: Execute '{}' in environment '{}'", command, env),
                    line_type: TerminalLineType::Output,
                    timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                });
            } else {
                state.lines.push_back(TerminalLine {
                    content: format!("No active environment. Use 'env activate <name>' first."),
                    line_type: TerminalLineType::Error,
                    timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        _ => {
            // Try to execute as system command
            execute_system_command(state, &command);
        }
    }
}

fn execute_system_command(state: &mut TerminalState, command: &str) {
    let working_dir = state.working_directory.clone();

    // Execute the command synchronously for now (in a real app, you'd want proper async handling)
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(&working_dir)
            .output()
    } else {
        Command::new("sh")
            .args(["-c", command])
            .current_dir(&working_dir)
            .output()
    };

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Add stdout if not empty
            if !stdout.is_empty() {
                for line in stdout.lines() {
                    state.lines.push_back(TerminalLine {
                        content: line.to_string(),
                        line_type: TerminalLineType::Output,
                        timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
            }

            // Add stderr if not empty
            if !stderr.is_empty() {
                for line in stderr.lines() {
                    state.lines.push_back(TerminalLine {
                        content: line.to_string(),
                        line_type: TerminalLineType::Error,
                        timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
            }

            // If no output, show command completed
            if stdout.is_empty() && stderr.is_empty() {
                state.lines.push_back(TerminalLine {
                    content: "Command completed".to_string(),
                    line_type: TerminalLineType::System,
                    timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        Err(e) => {
            state.lines.push_back(TerminalLine {
                content: format!("Error executing command: {}", e),
                line_type: TerminalLineType::Error,
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            });
        }
    }
}
