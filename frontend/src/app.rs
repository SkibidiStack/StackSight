use crate::router::AppRouter;
use crate::state::{AppState, Command, Event, SystemSnapshot};
use dioxus::prelude::*;
use dioxus_signals::Signal;
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::FutureExt;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const BASE_STYLE: &str = r#"
:root {
    --font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --bg: #0d1117;
    --surface: #0f141c;
    --panel: #121926;
    --panel-strong: #0f1624;
    --muted: #8b97ab;
    --text: #e6edf7;
    --accent: #4d8cf5;
    --accent-strong: #2c6df2;
    --border: rgba(255, 255, 255, 0.08);
    --card: #141c2b;
    --card-border: rgba(255, 255, 255, 0.06);
}

* { box-sizing: border-box; }
html, body, #main { width: 100%; height: 100%; margin: 0; padding: 0; background: var(--bg); color: var(--text); font-family: var(--font-family); }

.app-shell { display: flex; width: 100%; height: 100%; }
.content-area { flex: 1; display: flex; flex-direction: column; background: var(--bg); }
.section-body { flex: 1; padding: 20px 24px; overflow: auto; }
.section-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
.panel { background: var(--card); border: 1px solid var(--card-border); border-radius: 12px; padding: 16px; }
.panel h2 { margin: 0 0 8px; font-size: 16px; }
.muted { color: var(--muted); font-size: 14px; }
.nav-link { text-decoration: none; color: inherit; display: flex; align-items: center; justify-content: space-between; padding: 12px 14px; border-radius: 10px; border: 1px solid transparent; }
.nav-link:hover { border-color: var(--border); background: rgba(255, 255, 255, 0.04); }
.nav-active { background: rgba(77, 140, 245, 0.12); border-color: rgba(77, 140, 245, 0.4); }
.topbar { display: flex; align-items: center; justify-content: space-between; padding: 14px 20px; border-bottom: 1px solid var(--border); background: var(--surface); }
.topbar-title { font-size: 18px; font-weight: 600; }
.topbar-actions { display: flex; gap: 10px; align-items: center; }
.badge { padding: 6px 10px; border-radius: 999px; border: 1px solid var(--border); color: var(--muted); font-size: 13px; }
.pill { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: 999px; border: 1px solid var(--border); font-size: 12px; color: var(--muted); }
.grid-two { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 16px; }
.stat { display: flex; flex-direction: column; gap: 6px; }
.stat .label { font-size: 13px; color: var(--muted); }
.stat .value { font-size: 20px; font-weight: 600; }
.action-bar { display: flex; gap: 10px; }
.btn { border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; background: rgba(255, 255, 255, 0.04); color: var(--text); cursor: pointer; }
.btn.primary { background: linear-gradient(135deg, var(--accent), var(--accent-strong)); color: #0b1020; border: none; }
.btn.small { padding: 5px 8px; font-size: 12px; border-radius: 6px; }
.btn.ghost { background: transparent; border-color: var(--border); }
.btn.danger { background: linear-gradient(135deg, #f27d7d, #e46363); color: #0b1020; border: none; }
.input {
    width: 100%;
    padding: 8px 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    font-size: 13px;
}
.input:focus {
    outline: none;
    border-color: rgba(77, 140, 245, 0.6);
    box-shadow: 0 0 0 2px rgba(77, 140, 245, 0.2);
}
select.input {
    appearance: none;
}
.row-actions { display: flex; gap: 8px; justify-content: flex-end; }
.status-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.status-running { background: #2dc7a2; }
.status-stopped { background: #e66b6b; }
.status-unknown { background: #f5d06f; }
.pill { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: 999px; background: rgba(255, 255, 255, 0.04); border: 1px solid var(--border); font-size: 12px; }

.sidebar {
    width: 260px;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px 14px;
    background: var(--surface);
    border-right: 1px solid var(--border);
}

.sidebar-brand {
    font-size: 14px;
    font-weight: 700;
    color: var(--text);
    padding: 6px 10px 12px;
}

.nav-item {
    text-decoration: none;
    color: inherit;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid transparent;
}

.nav-item:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: var(--border);
}

.nav-item.nav-active {
    background: rgba(77, 140, 245, 0.16);
    border-color: rgba(77, 140, 245, 0.4);
}

.nav-title {
    font-size: 14px;
    font-weight: 600;
}

.nav-subtitle {
    font-size: 12px;
    color: var(--muted);
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

    rsx! {
        style { {BASE_STYLE} }
        AppRouter {}
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
            if ok {
                state.docker.last_error = None;
            } else {
                let msg = message.unwrap_or_else(|| format!("{action} failed"));
                state.docker.last_error = Some(msg);
            }
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
