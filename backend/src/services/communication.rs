use crate::core::event_bus::EventBus;
use anyhow::Result;
use futures_util::{stream::StreamExt, SinkExt};
use std::net::SocketAddr;
use tokio::{net::TcpListener, select as tokio_select, sync::{broadcast, mpsc}};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{info, warn};

pub struct CommunicationService {
    bus: EventBus,
    addr: SocketAddr,
    cmd_tx: mpsc::Sender<crate::models::commands::Command>,
}

impl CommunicationService {
    pub async fn new(bus: EventBus, cmd_tx: mpsc::Sender<crate::models::commands::Command>) -> Result<Self> {
        let addr: SocketAddr = "127.0.0.1:8765".parse().unwrap();
        Ok(Self { bus, addr, cmd_tx })
    }
}

#[async_trait::async_trait]
impl crate::services::Service for CommunicationService {
    async fn start(&mut self) -> Result<()> {
        info!("communication service start");
        Ok(())
    }

    async fn run(self) -> Result<()> {
        let (tx, _) = broadcast::channel::<String>(128);

        // Forward internal events to WebSocket broadcast channel.
        let bus = self.bus.clone();
        let tx_events = tx.clone();
        tokio::spawn(async move {
            let mut rx = bus.subscribe();
            while let Ok(event) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = tx_events.send(json);
                }
            }
        });

        let listener = TcpListener::bind(self.addr).await?;
        info!(addr = %self.addr, "comm websocket listening");

        loop {
            let (stream, peer) = listener.accept().await?;
            let tx_clients = tx.clone();
            let cmd_tx = self.cmd_tx.clone();
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws) => {
                        info!(%peer, "ws connected");
                        if let Err(err) = handle_socket(ws, tx_clients.subscribe(), cmd_tx).await {
                            warn!(%peer, error = ?err, "ws handler exited");
                        }
                    }
                    Err(err) => warn!(%peer, error = ?err, "ws upgrade failed"),
                }
            });
        }
    }
}

async fn handle_socket(
    stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut rx: broadcast::Receiver<String>,
    cmd_tx: mpsc::Sender<crate::models::commands::Command>,
) -> Result<()> {
    let (mut sink, mut source) = stream.split();

    loop {
        tokio_select! {
            msg = rx.recv() => {
                if let Ok(payload) = msg {
                    sink.send(Message::Text(payload)).await?;
                } else {
                    break;
                }
            }
            inbound = source.next() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        info!(target = "comm", "cmd_in: {text}");
                        match serde_json::from_str::<crate::models::commands::Command>(&text) {
                            Ok(cmd) => {
                                if let Err(err) = cmd_tx.send(cmd).await {
                                    warn!(error = ?err, "command send failed");
                                }
                            }
                            Err(err) => warn!(error = ?err, "bad command json"),
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(err)) => {
                        warn!(error = ?err, "ws read error");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
