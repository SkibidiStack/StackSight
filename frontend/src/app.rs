use crate::router::AppRouter;
use crate::state::{AppState, Command, Event, DockerfileEditor, LogsModal, SystemSnapshot, Theme, Toast, ToastType};
use dioxus::prelude::*;
use dioxus_signals::Signal;
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::FutureExt;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const LIGHT_THEME: &str = r#"
:root {
    --font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --bg: #f7f8fa;
    --surface: #ffffff;
    --panel: #ffffff;
    --panel-hover: #f7f8fa;
    --muted: #6c757d;
    --text: #212529;
    --accent: #0d6efd;
    --accent-hover: #0b5ed7;
    --border: #dee2e6;
    --border-light: #e9ecef;
    --success: #198754;
    --danger: #dc3545;
    --warning: #ffc107;
}
"#;

const DARK_THEME: &str = r#"
:root {
    --font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --bg: #1a1d23;
    --surface: #23262d;
    --panel: #23262d;
    --panel-hover: #2c2f38;
    --muted: #8b92a0;
    --text: #e4e6eb;
    --accent: #4dabf7;
    --accent-hover: #339af0;
    --border: #3a3d47;
    --border-light: #2f323a;
    --success: #51cf66;
    --danger: #ff6b6b;
    --warning: #ffd43b;
}
"#;

const BASE_STYLE: &str = r#"

* { box-sizing: border-box; margin: 0; padding: 0; }
html, body, #main { width: 100%; height: 100%; background: var(--bg); color: var(--text); font-family: var(--font-family); font-size: 14px; }

.app-shell { display: flex; width: 100%; height: 100%; }
.content-area { flex: 1; display: flex; flex-direction: column; background: var(--bg); overflow: hidden; }
.section-body { flex: 1; padding: 0; overflow: auto; }

/* Sidebar */
.sidebar {
    width: 240px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
}

.sidebar-brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px;
    border-bottom: 1px solid var(--border);
    font-size: 16px;
    font-weight: 600;
}

.sidebar-brand > span {
    flex: 1;
}

.brand-icon {
    font-size: 20px;
    color: var(--accent);
}

.sidebar-nav {
    display: flex;
    flex-direction: column;
    padding: 8px;
}

.sidebar-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 6px;
    text-decoration: none;
    color: var(--text);
    font-size: 14px;
    transition: all 0.15s;
}

.sidebar-item:hover {
    background: var(--panel-hover);
}

.sidebar-item-active {
    background: var(--panel-hover);
    color: var(--accent);
    font-weight: 500;
}

.sidebar-icon {
    font-size: 18px;
    width: 20px;
    text-align: center;
}

/* Docker View */
.docker-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
}

.view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
}

.view-title {
    display: flex;
    align-items: center;
    gap: 12px;
}

.view-title h1 {
    font-size: 24px;
    font-weight: 600;
}

.count-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 8px;
    border-radius: 12px;
    background: var(--panel-hover);
    color: var(--muted);
    font-size: 13px;
    font-weight: 500;
}

.view-actions {
    display: flex;
    align-items: center;
    gap: 12px;
}

.search-input {
    width: 300px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 14px;
}

.search-input:focus {
    outline: none;
    border-color: var(--accent);
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 16px;
    font-size: 12px;
    font-weight: 500;
}

.status-badge.status-running {
    color: var(--success);
    background: rgba(25, 135, 84, 0.1);
}

.status-badge.status-stopped {
    color: var(--muted);
    background: rgba(108, 117, 125, 0.1);
}

.status-badge.status-warning {
    color: var(--warning);
    background: rgba(255, 193, 7, 0.1);
}

.status-badge.status-error {
    color: var(--danger);
    background: rgba(220, 53, 69, 0.1);
}

/* Docker Table */
.docker-table {
    width: 100%;
    border-collapse: collapse;
    background: var(--surface);
}

.docker-table thead {
    position: sticky;
    top: 0;
    background: var(--surface);
    z-index: 10;
}

.docker-table th {
    padding: 12px 16px;
    text-align: left;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
}

.docker-table td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-light);
    font-size: 13px;
}

.docker-table tbody tr {
    transition: background 0.1s;
}

.docker-table tbody tr:hover {
    background: var(--panel-hover);
}

.col-checkbox {
    width: 48px;
    text-align: center;
}

.col-name {
    min-width: 200px;
}

.cell-main {
    font-weight: 500;
    margin-bottom: 4px;
}

.cell-sub {
    color: var(--muted);
    font-size: 12px;
}

.col-image {
    color: var(--muted);
}

.col-status {
    width: 140px;
}

.col-ports {
    color: var(--muted);
}

.col-actions {
    width: 180px;
}

.action-buttons {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.15s;
}

.docker-table tbody tr:hover .action-buttons {
    opacity: 1;
}

.action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
}

.action-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

.action-btn.action-primary {
    color: var(--success);
}

.action-btn.action-primary:hover {
    background: var(--success);
    color: white;
    border-color: var(--success);
}

.action-btn.action-danger {
    color: var(--danger);
}

.action-btn.action-danger:hover {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
}

/* Empty State */
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 24px;
    text-align: center;
}

.empty-icon {
    font-size: 64px;
    color: var(--muted);
    opacity: 0.3;
    margin-bottom: 16px;
}

.empty-state h3 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 8px;
}

.empty-state p {
    color: var(--muted);
    font-size: 14px;
}

/* Alert */
.alert {
    margin: 16px 24px;
    padding: 12px 16px;
    border-radius: 6px;
    font-size: 13px;
}

.alert-error {
    background: rgba(220, 53, 69, 0.1);
    color: var(--danger);
    border: 1px solid rgba(220, 53, 69, 0.2);
}

/* Legacy Compatibility */
.topbar { display: flex; align-items: center; justify-content: space-between; padding: 14px 20px; border-bottom: 1px solid var(--border); background: var(--surface); }
.topbar-title { font-size: 18px; font-weight: 600; }
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 16px; }
.panel h2 { margin: 0 0 12px; font-size: 16px; font-weight: 600; }
.muted { color: var(--muted); }
.btn { border: 1px solid var(--border); border-radius: 6px; padding: 8px 16px; background: var(--surface); color: var(--text); cursor: pointer; font-size: 14px; }
.btn:hover { background: var(--panel-hover); }
.btn.primary { background: var(--accent); color: white; border-color: var(--accent); }
.btn.primary:hover { background: var(--accent-hover); }
.input { width: 100%; padding: 8px 12px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface); color: var(--text); font-size: 14px; }
.input:focus { outline: none; border-color: var(--accent); }
textarea.input {
    resize: vertical;
    font-family: var(--font-family);
    min-height: 60px;
}
select.input {
    appearance: auto;
}

/* Modal */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal {
    background: var(--surface);
    border-radius: 8px;
    padding: 24px;
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
}

.modal h2 {
    margin: 0 0 20px;
    font-size: 20px;
    font-weight: 600;
}

.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
}

.modal-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 24px;
}

/* Logs Modal */
.logs-modal {
    max-width: 900px;
    max-height: 80vh;
}

.logs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
}

.logs-content {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 16px;
    max-height: 60vh;
    overflow: auto;
}

.logs-pre {
    margin: 0;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text);
    white-space: pre-wrap;
    word-wrap: break-word;
}

/* Dockerfile Editor Modal */
.dockerfile-editor-modal {
    max-width: 900px;
    width: 90%;
}

.dockerfile-path {
    margin-bottom: 16px;
    padding: 8px 12px;
    background: var(--panel-hover);
    border-radius: 6px;
    font-size: 13px;
    color: var(--muted);
}

.dockerfile-textarea {
    min-height: 400px;
    font-family: 'Courier New', monospace;
    font-size: 13px;
    line-height: 1.5;
}

/* Theme Toggle */
.theme-toggle-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    color: var(--text);
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    flex-shrink: 0;
}

.theme-toggle-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

/* Engine Manager */
.engine-info-panel {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
    margin-bottom: 24px;
}

.info-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
}

.info-card h3 {
    margin: 0 0 16px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-light);
}

.info-row:last-child {
    border-bottom: none;
}

.info-label {
    font-size: 14px;
    color: var(--muted);
}

.info-value {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
}

.error-text {
    color: #ef4444;
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
}

.status-running {
    background: #10b98120;
    color: #10b981;
}

.status-stopped {
    background: #ef444420;
    color: #ef4444;
}

.logs-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
}

.logs-panel h3 {
    margin: 0 0 16px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.logs-container {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 16px;
    max-height: 500px;
    overflow-y: auto;
}

.logs-content {
    margin: 0;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.6;
    color: var(--text);
    white-space: pre-wrap;
    word-wrap: break-word;
}

.logs-empty {
    text-align: center;
    padding: 40px 20px;
    color: var(--muted);
    font-size: 14px;
}

/* Toast Notifications */
.toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    z-index: 2000;
    pointer-events: none;
}

.toast {
    min-width: 320px;
    max-width: 420px;
    padding: 16px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    display: flex;
    align-items: flex-start;
    gap: 12px;
    pointer-events: auto;
    animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

.toast-icon {
    font-size: 20px;
    flex-shrink: 0;
}

.toast-content {
    flex: 1;
    font-size: 14px;
    line-height: 1.4;
}

.toast.toast-success {
    border-color: var(--success);
}

.toast.toast-success .toast-icon {
    color: var(--success);
}

.toast.toast-error {
    border-color: var(--danger);
}

.toast.toast-error .toast-icon {
    color: var(--danger);
}

.toast.toast-info {
    border-color: var(--accent);
}

.toast.toast-info .toast-icon {
    color: var(--accent);
}
"#;

#[component]
pub fn AppRoot() -> Element {
    let app_state: Signal<AppState> = use_signal(AppState::default);
    use_context_provider(|| app_state);

    let bridge = {
        let app_state = app_state.clone();
        use_coroutine(move |rx: UnboundedReceiver<BridgeAction>| async move {
            let mut app_state = app_state;
            let mut pending: VecDeque<Command> = VecDeque::new();
            let mut rx = rx.fuse();
            let addr = "ws://127.0.0.1:8765";

            'outer: loop {
                let connect = connect_async(addr).fuse();
                futures::pin_mut!(connect);

                let (mut sink, mut stream) = loop {
                    futures::select! {
                        conn = connect => {
                            match conn {
                                Ok((ws, _)) => {
                                    {
                                        let mut state = app_state.write();
                                        state.docker.connected = true;
                                        state.docker.last_error = None;
                                    }
                                    let (sink, stream) = ws.split();
                                    break (sink, stream.fuse());
                                }
                                Err(err) => {
                                    {
                                        let mut state = app_state.write();
                                        state.docker.connected = false;
                                        state.docker.last_error = Some(err.to_string());
                                    }
                                    tokio::time::sleep(Duration::from_secs(2)).await;
                                    continue 'outer;
                                }
                            }
                        }
                        action = rx.next() => {
                            match action {
                                Some(BridgeAction::SendCommand(cmd)) => pending.push_back(cmd),
                                None => return,
                            }
                        }
                    }
                };

                pending.push_back(Command::DockerList);
                pending.push_back(Command::DockerListImages);
                pending.push_back(Command::DockerListNetworks);
                pending.push_back(Command::DockerListVolumes);

                while let Some(cmd) = pending.pop_front() {
                    if let Err(err) = send_command(&mut sink, cmd.clone()).await {
                        pending.push_front(cmd);
                        let mut state = app_state.write();
                        state.docker.connected = false;
                        state.docker.last_error = Some(err);
                        break;
                    }
                }

                loop {
                    futures::select! {
                        inbound = stream.next() => {
                            match inbound {
                                Some(Ok(Message::Text(text))) => {
                                    if let Err(err) = handle_event(app_state.clone(), &text) {
                                        let mut state = app_state.write();
                                        state.docker.last_error = Some(err);
                                    }
                                }
                                Some(Ok(Message::Close(_))) | None => {
                                    let mut state = app_state.write();
                                    state.docker.connected = false;
                                    break;
                                }
                                Some(Err(err)) => {
                                    let mut state = app_state.write();
                                    state.docker.connected = false;
                                    state.docker.last_error = Some(err.to_string());
                                    break;
                                }
                                _ => {}
                            }
                        }
                        action = rx.next() => {
                            match action {
                                Some(BridgeAction::SendCommand(cmd)) => {
                                    if let Err(err) = send_command(&mut sink, cmd.clone()).await {
                                        pending.push_back(cmd);
                                        let mut state = app_state.write();
                                        state.docker.connected = false;
                                        state.docker.last_error = Some(err);
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
    };

    let bridge_handle = BackendBridge { tx: bridge };
    use_context_provider(|| bridge_handle);

    // Auto-dismiss toasts after 5 seconds
    use_effect(move || {
        let mut app_state = app_state;
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                let mut state = app_state.write();
                state.ui.toasts.retain(|toast| {
                    now - toast.timestamp < 5000 // Keep toasts for 5 seconds
                });
            }
        });
    });

    let theme = app_state.read().user.theme.clone();
    let theme_css = match theme {
        Theme::Light => LIGHT_THEME,
        Theme::Dark => DARK_THEME,
    };
    
    let full_style = format!("{}{}", theme_css, BASE_STYLE);

    rsx! {
        style { {full_style} }
        AppRouter {}
        ToastContainer {}
        LogsModalComponent {}
        DockerfileEditorModal {}
        BuildConfirmationModal {}
    }
}

#[component]
fn ToastContainer() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let toasts = app_state.read().ui.toasts.clone();

    rsx! {
        div { class: "toast-container",
            for toast in toasts.iter() {
                ToastItem { key: "{toast.id}", toast: toast.clone() }
            }
        }
    }
}

#[component]
fn ToastItem(toast: crate::state::Toast) -> Element {
    let icon = match toast.toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
        ToastType::Info => "ℹ",
    };
    
    let class_name = match toast.toast_type {
        ToastType::Success => "toast toast-success",
        ToastType::Error => "toast toast-error",
        ToastType::Info => "toast toast-info",
    };

    rsx! {
        div { class: "{class_name}",
            div { class: "toast-icon", "{icon}" }
            div { class: "toast-content", "{toast.message}" }
        }
    }
}

#[derive(Clone)]
pub struct BackendBridge {
    tx: Coroutine<BridgeAction>,
}

impl BackendBridge {
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(BridgeAction::SendCommand(cmd));
    }
}

#[derive(Clone)]
enum BridgeAction {
    SendCommand(Command),
}

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

async fn send_command(
    sink: &mut WsSink,
    cmd: Command,
) -> Result<(), String> {
    let payload = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
    sink.send(Message::Text(payload)).await.map_err(|e| e.to_string())
}

fn handle_event(mut app_state: Signal<AppState>, payload: &str) -> Result<(), String> {
    match serde_json::from_str::<Event>(payload) {
        Ok(Event::DockerContainers(containers)) => {
            app_state.write().docker.containers = containers;
            Ok(())
        }
        Ok(Event::DockerStatus { connected, error }) => {
            let mut state = app_state.write();
            state.docker.connected = connected;
            state.docker.last_error = error;
            Ok(())
        }
        Ok(Event::DockerStats {
            containers,
            cpu_percent_avg,
            memory_used,
            memory_limit,
            net_rx,
            net_tx,
        }) => {
            let mut state = app_state.write();
            state.docker.stats.containers = containers;
            state.docker.stats.cpu_percent_avg = cpu_percent_avg;
            state.docker.stats.memory_used = memory_used;
            state.docker.stats.memory_limit = memory_limit;
            state.docker.stats.net_rx = net_rx;
            state.docker.stats.net_tx = net_tx;
            Ok(())
        }
        Ok(Event::DockerImages(images)) => {
            app_state.write().docker.images = images;
            Ok(())
        }
        Ok(Event::DockerNetworks(networks)) => {
            app_state.write().docker.networks = networks;
            Ok(())
        }
        Ok(Event::DockerVolumes(volumes)) => {
            app_state.write().docker.volumes = volumes;
            Ok(())
        }
        Ok(Event::DockerAction { action, ok, message }) => {
            let mut state = app_state.write();
            state.docker.action.in_progress = false;
            state.docker.action.last_action = Some(action.clone());
            state.docker.action.last_ok = Some(ok);
            state.docker.action.message = message.clone();
            
            // Add toast notification
            let toast_type = if ok { ToastType::Success } else { ToastType::Error };
            let toast_message = if ok {
                format!("{} completed successfully", action)
            } else {
                message.clone().unwrap_or_else(|| format!("{} failed", action))
            };
            
            let toast_id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            state.ui.toasts.push(Toast {
                id: toast_id,
                message: toast_message,
                toast_type,
                timestamp: toast_id,
            });
            
            // Keep only last 5 toasts
            if state.ui.toasts.len() > 5 {
                state.ui.toasts.remove(0);
            }
            
            Ok(())
        }
        Ok(Event::DockerLogs { container_id, logs }) => {
            let mut state = app_state.write();
            state.ui.logs_modal = Some(LogsModal { container_id, logs });
            Ok(())
        }
        Ok(Event::DockerfileGenerated { path, dockerfile }) => {
            let mut state = app_state.write();
            state.ui.dockerfile_editor = Some(DockerfileEditor { path, dockerfile });
            Ok(())
        }
        Ok(Event::DockerfileSaved { path }) => {
            let mut state = app_state.write();
            state.ui.build_confirmation = Some(path);
            Ok(())
        }
        Ok(Event::DockerEngineLogs { logs }) => {
            let mut state = app_state.write();
            state.ui.engine_logs = Some(logs);
            Ok(())
        }
        Ok(Event::VirtualEnvSummary { total, active }) => {
            let mut state = app_state.write();
            state.virtenv.environments = total;
            state.virtenv.active = active;
            Ok(())
        }
        Ok(Event::SystemSnapshot(snapshot)) => {
            apply_system_snapshot(app_state, snapshot);
            Ok(())
        }
        Err(err) => Err(format!("event parse failed: {err}")),
    }
}

fn apply_system_snapshot(mut app_state: Signal<AppState>, snapshot: SystemSnapshot) {
    let mut state = app_state.write();
    state.system.cpu_usage = snapshot.cpu_usage;
    if snapshot.memory_total > 0 {
        state.system.memory_usage = (snapshot.memory_used as f32 / snapshot.memory_total as f32) * 100.0;
    } else {
        state.system.memory_usage = 0.0;
    }
}

#[component]
pub fn ThemeToggle() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let theme = app_state.read().user.theme.clone();
    
    let toggle_theme = move |_| {
        let mut state = app_state.write();
        state.user.theme = match state.user.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    };
    
    let icon = match theme {
        Theme::Light => "🌙",
        Theme::Dark => "☀️",
    };
    
    rsx! {
        button {
            class: "theme-toggle-btn",
            onclick: toggle_theme,
            title: "Toggle theme",
            "{icon}"
        }
    }
}

#[component]
fn LogsModalComponent() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let logs_modal = app_state.read().ui.logs_modal.clone();
    
    if let Some(modal) = logs_modal {
        let on_close = move |_| {
            app_state.write().ui.logs_modal = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal logs-modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Container Logs" }
                    div { class: "logs-header",
                        div { class: "cell-sub", "{modal.container_id[..12].to_string()}" }
                        button { class: "btn", onclick: on_close, "Close" }
                    }
                    div { class: "logs-content",
                        pre { class: "logs-pre", "{modal.logs}" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

#[component]
fn DockerfileEditorModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let editor = app_state.read().ui.dockerfile_editor.clone();
    
    if let Some(editor_data) = editor {
        let mut dockerfile_content = use_signal(|| editor_data.dockerfile.clone());
        let path_clone = editor_data.path.clone();
        
        let on_close = move |_| {
            app_state.write().ui.dockerfile_editor = None;
        };
        
        let on_save = move |_| {
            let content = dockerfile_content.read().clone();
            let path = path_clone.clone();
            
            bridge.send(Command::DockerSaveDockerfile {
                path,
                dockerfile: content,
            });
            
            app_state.write().ui.dockerfile_editor = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal dockerfile-editor-modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Edit Dockerfile" }
                    div { class: "dockerfile-path",
                        "Project: {editor_data.path}"
                    }
                    div { class: "form-group",
                        textarea {
                            class: "input dockerfile-textarea",
                            rows: "20",
                            value: "{dockerfile_content}",
                            oninput: move |e| dockerfile_content.set(e.value().clone()),
                            style: "font-family: 'Courier New', monospace; white-space: pre;"
                        }
                    }
                    div { class: "modal-actions",
                        button { class: "btn", onclick: on_close, "Cancel" }
                        button { class: "btn primary", onclick: on_save, "Save Dockerfile" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

#[component]
fn BuildConfirmationModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let build_path = app_state.read().ui.build_confirmation.clone();
    
    if let Some(path) = build_path {
        let path_clone = path.clone();
        
        let on_close = move |_| {
            app_state.write().ui.build_confirmation = None;
        };
        
        let on_build = move |_| {
            let tag = format!("{}:latest", path_clone.split('/').last().unwrap_or("app"));
            bridge.send(Command::DockerBuildImage {
                context_path: path_clone.clone(),
                tag: Some(tag),
            });
            app_state.write().ui.build_confirmation = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Dockerfile Saved" }
                    p { "Dockerfile has been saved to:" }
                    div { class: "dockerfile-path",
                        "{path}/Dockerfile"
                    }
                    p { style: "margin-top: 16px;", "Would you like to build this as a Docker image?" }
                    div { class: "modal-actions",
                        button { class: "btn", onclick: on_close, "Not Now" }
                        button { class: "btn primary", onclick: on_build, "Build Image" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
