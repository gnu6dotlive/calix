use crate::control::protocol::{
    ControlCommand, ControlRequest, ControlResponse, CONTROL_PROTOCOL_VERSION,
};
use crate::control::socket::ControlSocket;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};

pub struct ControlIpcMessage {
    pub command: ControlCommand,
    pub responder: oneshot::Sender<ControlResponse>,
}

pub struct ControlIpcServer;

impl ControlIpcServer {
    /// Starts the background Unix domain socket listener
    pub fn start(command_tx: mpsc::Sender<ControlIpcMessage>) -> Result<()> {
        let listener = ControlSocket::bind_listener()?;
        tokio::spawn(async move {
            Self::listen_loop(listener, command_tx).await;
        });
        Ok(())
    }

    async fn listen_loop(listener: UnixListener, command_tx: mpsc::Sender<ControlIpcMessage>) {
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let tx = command_tx.clone();
                    tokio::spawn(async move {
                        Self::handle_connection(stream, tx).await;
                    });
                }
                Err(e) => {
                    log::warn!("Control socket accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    }

    async fn handle_connection(stream: UnixStream, command_tx: mpsc::Sender<ControlIpcMessage>) {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        if buf_reader.read_line(&mut line).await.is_err() || line.trim().is_empty() {
            return;
        }

        let request: Result<ControlRequest, _> = serde_json::from_str(line.trim());
        let response = match request {
            Ok(req) => {
                if req.version != CONTROL_PROTOCOL_VERSION {
                    ControlResponse::error(format!(
                        "Protocol version mismatch: client is v{}, server is v{}",
                        req.version, CONTROL_PROTOCOL_VERSION
                    ))
                } else {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let msg = ControlIpcMessage {
                        command: req.command,
                        responder: resp_tx,
                    };
                    if command_tx.send(msg).await.is_err() {
                        ControlResponse::error("UI event loop is unavailable")
                    } else {
                        match resp_rx.await {
                            Ok(resp) => resp,
                            Err(_) => ControlResponse::error("UI failed to process command"),
                        }
                    }
                }
            }
            Err(e) => ControlResponse::error(format!("Malformed control request: {e}")),
        };

        if let Ok(mut resp_json) = serde_json::to_string(&response) {
            resp_json.push('\n');
            let _ = writer.write_all(resp_json.as_bytes()).await;
            let _ = writer.flush().await;
        }
    }
}
