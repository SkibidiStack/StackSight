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
    --font-family: 'Space Grotesk', 'IBM Plex Sans', 'SF Pro Display', sans-serif;
    --bg: #0f1722;
    --panel: #131c2b;
    --panel-strong: #0a101b;
    --muted: #8ca3b8;
    --text: #e9f1ff;
    --accent: #6ce0c6;
    --accent-strong: #2dc7a2;
    --border: rgba(255, 255, 255, 0.08);
    --glow: 0 20px 60px rgba(44, 179, 152, 0.18);
}

* { box-sizing: border-box; }
html, body, #main { width: 100%; height: 100%; margin: 0; padding: 0; background: var(--bg); color: var(--text); font-family: var(--font-family); }

.app-shell { display: flex; width: 100%; height: 100%; }
.content-area { flex: 1; display: flex; flex-direction: column; background: radial-gradient(circle at 20% 20%, rgba(108, 224, 198, 0.06), transparent 30%), radial-gradient(circle at 80% 0%, rgba(45, 199, 162, 0.1), transparent 25%), var(--bg); }
.section-body { flex: 1; padding: 24px; overflow: auto; }
.section-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
.panel { background: linear-gradient(145deg, var(--panel), var(--panel-strong)); border: 1px solid var(--border); border-radius: 14px; padding: 16px; box-shadow: var(--glow); }
.panel h2 { margin: 0 0 8px; font-size: 18px; }
.muted { color: var(--muted); font-size: 14px; }
.chip { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: 10px; border: 1px solid var(--border); color: var(--text); background: rgba(255, 255, 255, 0.03); }
.nav-link { text-decoration: none; color: inherit; display: flex; align-items: center; justify-content: space-between; padding: 12px 14px; border-radius: 12px; border: 1px solid transparent; }
.nav-link:hover { border-color: var(--border); background: rgba(255, 255, 255, 0.04); }
.nav-active { background: rgba(108, 224, 198, 0.1); border-color: rgba(108, 224, 198, 0.4); }
.header { display: flex; align-items: center; justify-content: space-between; padding: 18px 22px; border-bottom: 1px solid var(--border); background: linear-gradient(135deg, rgba(19, 28, 43, 0.95), rgba(15, 23, 34, 0.95)); backdrop-filter: blur(6px); }
.header h1 { margin: 0; font-size: 22px; letter-spacing: 0.02em; }
.badge { padding: 6px 10px; border-radius: 999px; border: 1px solid var(--border); color: var(--muted); font-size: 13px; }
.grid-two { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 16px; }
.stat { display: flex; flex-direction: column; gap: 6px; }
.stat .label { font-size: 13px; color: var(--muted); }
.stat .value { font-size: 20px; font-weight: 600; }
.action-bar { display: flex; gap: 10px; }
.btn { border: 1px solid var(--border); border-radius: 10px; padding: 10px 14px; background: rgba(255, 255, 255, 0.04); color: var(--text); cursor: pointer; }
.btn.primary { background: linear-gradient(135deg, var(--accent), var(--accent-strong)); color: #0a111c; border: none; }
.btn.small { padding: 6px 10px; font-size: 13px; border-radius: 8px; }
.btn.ghost { background: rgba(255, 255, 255, 0.02); border-color: var(--border); }
.btn.danger { background: linear-gradient(135deg, #f79e9e, #e66b6b); color: #0a111c; border: none; }
.row-actions { display: flex; gap: 8px; justify-content: flex-end; }
.status-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.status-running { background: #2dc7a2; }
.status-stopped { background: #e66b6b; }
.status-unknown { background: #f5d06f; }
.pill { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: 10px; background: rgba(255, 255, 255, 0.04); border: 1px solid var(--border); font-size: 13px; }
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
