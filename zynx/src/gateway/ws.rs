use futures_util::{SinkExt, StreamExt};
use rustls::{ClientConfig, RootCertStore};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Duration, Instant, sleep_until, timeout};
use tokio_tungstenite::{
    Connector, WebSocketStream, connect_async_tls_with_config,
    tungstenite::{Message, protocol::CloseFrame},
};

use crate::gateway::{DisconnectReason, Gateway, heartbeat::Heartbeat};
use crate::models::gateway::{
    GatewayDispatch, GatewayDispatchEvents, GatewayIntents, GatewayOpcodes,
};

const DISCORD_GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const EVENT_CAPACITY: usize = 256;

#[derive(thiserror::Error, Debug)]
pub enum ResumeError {
    #[error("invalid session (resumable={resumable})")]
    InvalidSession { resumable: bool },
    #[error("gateway closed with fatal code {code}; check token, intents, and shard configuration")]
    Fatal { code: u16 },
    #[error(transparent)]
    Transport(#[from] anyhow::Error),
}

impl ResumeError {
    fn recovery(&self) -> DisconnectReason {
        match self {
            Self::InvalidSession { resumable: false } => DisconnectReason::Identify,
            Self::Fatal { code } => DisconnectReason::Fatal(*code),
            _ => DisconnectReason::Resume,
        }
    }
}

#[derive(Deserialize)]
struct Frame {
    op: u8,
    s: Option<i64>,
    t: Option<String>,
    d: Option<Box<RawValue>>,
}

#[derive(Deserialize)]
struct SessionInfo {
    session_id: String,
    resume_gateway_url: String,
}

type ReadySender = oneshot::Sender<Result<SessionInfo, ResumeError>>;

pub async fn connect(token: &str, intents: GatewayIntents) -> anyhow::Result<Gateway> {
    let identify = serde_json::json!({
        "op": 2,
        "d": {
            "token": token,
            "intents": intents.bits(),
            "properties": { "os": std::env::consts::OS, "browser": "zynx", "device": "zynx" }
        }
    });
    let (socket, heartbeat_ms) = open(DISCORD_GATEWAY_URL, identify).await?;
    let (ready_tx, ready_rx) = oneshot::channel();
    let mut gateway = spawn_session(socket, heartbeat_ms, None, Some(ready_tx));
    // The owned Gateway aborts the task if this wait is cancelled or fails.
    let info = ready_rx.await??;
    gateway.session_id = info.session_id;
    gateway.resume_gateway_url = info.resume_gateway_url;
    Ok(gateway)
}

/// Start resuming a session. Replay events are available immediately; RESUMED
/// confirms completion. A rejected session closes the stream with a recovery reason.
/// Returning before RESUMED lets consumers drain replays through a bounded queue.
pub async fn resume(
    token: &str,
    session_id: &str,
    resume_gateway_url: &str,
    last_seq: Option<i64>,
) -> Result<Gateway, ResumeError> {
    let payload = serde_json::json!({
        "op": 6,
        "d": { "token": token, "session_id": session_id, "seq": last_seq }
    });
    let (socket, heartbeat_ms) = open(&normalize_gateway_url(resume_gateway_url)?, payload).await?;
    let mut gateway = spawn_session(socket, heartbeat_ms, last_seq, None);
    gateway.session_id = session_id.to_owned();
    gateway.resume_gateway_url = resume_gateway_url.to_owned();
    Ok(gateway)
}

async fn open(
    url: &str,
    auth: serde_json::Value,
) -> Result<
    (
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        u64,
    ),
    ResumeError,
> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config =
        ClientConfig::builder_with_provider(rustls::crypto::ring::default_provider().into())
            .with_safe_default_protocol_versions()
            .map_err(anyhow::Error::from)?
            .with_root_certificates(roots)
            .with_no_client_auth();
    let (mut socket, _) = timeout(
        HANDSHAKE_TIMEOUT,
        connect_async_tls_with_config(url, None, true, Some(Connector::Rustls(Arc::new(config)))),
    )
    .await
    .map_err(anyhow::Error::from)?
    .map_err(anyhow::Error::from)?;
    let heartbeat_ms = timeout(HANDSHAKE_TIMEOUT, async {
        loop {
            match socket.next().await {
                Some(Ok(Message::Text(text))) => {
                    let frame: Frame = serde_json::from_str(&text).map_err(anyhow::Error::from)?;
                    if frame.op == 10 {
                        #[derive(Deserialize)]
                        struct Hello {
                            heartbeat_interval: u64,
                        }
                        let raw = frame
                            .d
                            .ok_or_else(|| anyhow::anyhow!("HELLO without data"))?;
                        let hello: Hello =
                            serde_json::from_str(raw.get()).map_err(anyhow::Error::from)?;
                        if hello.heartbeat_interval == 0 {
                            return Err(anyhow::anyhow!(
                                "HELLO heartbeat interval must be positive"
                            )
                            .into());
                        }
                        return Ok::<_, ResumeError>(hello.heartbeat_interval);
                    }
                    if frame.op == 9 {
                        return Err(invalid_session(&frame));
                    }
                    if frame.op == 7 {
                        return Err(anyhow::anyhow!("reconnect before HELLO").into());
                    }
                }
                Some(Ok(Message::Close(frame))) => return Err(close_error(frame)),
                Some(Ok(Message::Ping(data))) => send(&mut socket, Message::Pong(data)).await?,
                Some(Ok(_)) => {}
                Some(Err(err)) => return Err(anyhow::Error::from(err).into()),
                None => return Err(anyhow::anyhow!("gateway closed before HELLO").into()),
            }
        }
    })
    .await
    .map_err(anyhow::Error::from)??;
    send(&mut socket, Message::Text(auth.to_string().into())).await?;
    Ok((socket, heartbeat_ms))
}

fn spawn_session<S>(
    socket: WebSocketStream<S>,
    heartbeat_ms: u64,
    last_seq: Option<i64>,
    mut ready: Option<ReadySender>,
) -> Gateway
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (writer_tx, writer_rx) = mpsc::channel(32);
    let (events_tx, events_rx) = mpsc::channel(EVENT_CAPACITY);
    let (last_seq_tx, last_seq_rx) = watch::channel(last_seq);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (disconnect_tx, disconnect_rx) = watch::channel(DisconnectReason::Resume);
    let task = tokio::spawn(async move {
        let mut session = Session {
            socket,
            heartbeat: Heartbeat::new(heartbeat_ms),
            writer_rx,
            events_tx,
            last_seq_tx,
            shutdown_rx,
        };
        let result = session.run(&mut ready).await;
        if let Err(err) = result {
            disconnect_tx.send_replace(err.recovery());
            crate::log!("WARN", "gateway disconnected: {err}");
            if let Some(tx) = ready.take() {
                let _ = tx.send(Err(err));
            }
        }
        // Dropping the socket and sender closes the stream. On recovery we do not
        // send a normal close frame, which would invalidate the resumable session.
    });
    Gateway {
        session_id: String::new(),
        resume_gateway_url: String::new(),
        writer_tx,
        events_rx,
        last_seq_rx,
        shutdown_tx,
        disconnect_rx,
        task,
    }
}

struct Session<S> {
    socket: WebSocketStream<S>,
    heartbeat: Heartbeat,
    writer_rx: mpsc::Receiver<Message>,
    events_tx: mpsc::Sender<GatewayDispatch<Box<RawValue>>>,
    last_seq_tx: watch::Sender<Option<i64>>,
    shutdown_rx: watch::Receiver<bool>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Session<S> {
    async fn run(&mut self, ready: &mut Option<ReadySender>) -> Result<(), ResumeError> {
        let expected = if ready.is_some() { "READY" } else { "RESUMED" };
        let handshake_deadline = Instant::now() + HANDSHAKE_TIMEOUT;
        let mut authenticated = false;
        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    let _ = timeout(WRITE_TIMEOUT, self.socket.close(None)).await;
                    return Ok(());
                }
                _ = sleep_until(handshake_deadline), if !authenticated => {
                    return Err(anyhow::anyhow!("timed out waiting for {expected}").into());
                }
                _ = sleep_until(self.heartbeat.deadline()) => {
                    self.heartbeat.tick()?;
                    self.send_heartbeat().await?;
                }
                outgoing = self.writer_rx.recv() => {
                    let Some(message) = outgoing else { return Ok(()); };
                    send(&mut self.socket, message).await?;
                }
                incoming = self.socket.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            let frame: Frame = serde_json::from_str(&text).map_err(anyhow::Error::from)?;
                            match frame.op {
                                0 => {
                                    let name = frame.t.as_deref().ok_or_else(|| anyhow::anyhow!("dispatch without event name"))?;
                                    let seq = frame.s.ok_or_else(|| anyhow::anyhow!("dispatch without sequence"))?;
                                    let raw = frame.d.ok_or_else(|| anyhow::anyhow!("dispatch without data"))?;
                                    let info = if !authenticated && name == "READY" && expected == "READY" {
                                        let info: SessionInfo = serde_json::from_str(raw.get()).map_err(anyhow::Error::from)?;
                                        if info.session_id.is_empty() { return Err(anyhow::anyhow!("empty session id").into()); }
                                        Some(info)
                                    } else { None };
                                    if let Ok(kind) = serde_json::from_value::<GatewayDispatchEvents>(serde_json::Value::String(name.to_owned())) {
                                        // Never wait on consumers in the socket loop: that would starve ACKs.
                                        // Leave seq unchanged on overflow so RESUME replays this event.
                                        self.events_tx.try_send(GatewayDispatch {
                                            op: GatewayOpcodes::Dispatch, t: kind, s: seq, d: raw,
                                        }).map_err(|_| anyhow::anyhow!("event queue full or closed; reconnecting from last queued sequence"))?;
                                    }
                                    self.last_seq_tx.send_replace(Some(seq));
                                    if name == expected {
                                        authenticated = true;
                                        if let (Some(info), Some(tx)) = (info, ready.take()) {
                                            let _ = tx.send(Ok(info));
                                        }
                                    }
                                }
                                1 => {
                                    self.heartbeat.sent();
                                    self.send_heartbeat().await?;
                                }
                                7 => return Err(anyhow::anyhow!("server requested reconnect").into()),
                                9 => return Err(invalid_session(&frame)),
                                11 => self.heartbeat.acknowledge(),
                                _ => {}
                            }
                        }
                        Some(Ok(Message::Close(frame))) => return Err(close_error(frame)),
                        Some(Ok(Message::Ping(data))) => send(&mut self.socket, Message::Pong(data)).await?,
                        Some(Ok(_)) => {}
                        Some(Err(err)) => return Err(anyhow::Error::from(err).into()),
                        None => return Err(anyhow::anyhow!("gateway stream ended").into()),
                    }
                }
            }
        }
    }

    async fn send_heartbeat(&mut self) -> Result<(), ResumeError> {
        let seq = *self.last_seq_tx.borrow();
        send(
            &mut self.socket,
            Message::Text(serde_json::json!({"op": 1, "d": seq}).to_string().into()),
        )
        .await
    }
}

async fn send<S: AsyncRead + AsyncWrite + Unpin>(
    socket: &mut WebSocketStream<S>,
    message: Message,
) -> Result<(), ResumeError> {
    timeout(WRITE_TIMEOUT, socket.send(message))
        .await
        .map_err(anyhow::Error::from)?
        .map_err(anyhow::Error::from)?;
    Ok(())
}

fn invalid_session(frame: &Frame) -> ResumeError {
    ResumeError::InvalidSession {
        resumable: frame.d.as_ref().is_some_and(|raw| raw.get() == "true"),
    }
}

fn close_error(frame: Option<CloseFrame>) -> ResumeError {
    match frame.map(|frame| u16::from(frame.code)) {
        Some(code @ (4004 | 4010..=4014)) => ResumeError::Fatal { code },
        Some(1000 | 1001 | 4007 | 4009) => ResumeError::InvalidSession { resumable: false },
        code => anyhow::anyhow!("gateway closed: {code:?}").into(),
    }
}

fn normalize_gateway_url(base: &str) -> anyhow::Result<String> {
    let mut url = reqwest::Url::parse(base)?;
    anyhow::ensure!(url.scheme() == "wss", "resume URL must use wss");
    url.set_query(Some("v=10&encoding=json"));
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{DuplexStream, duplex};
    use tokio_tungstenite::tungstenite::protocol::{Role, frame::coding::CloseCode};

    async fn pair(ready: Option<ReadySender>) -> (Gateway, WebSocketStream<DuplexStream>) {
        let (client, server) = duplex(128 * 1024);
        let client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        let server = WebSocketStream::from_raw_socket(server, Role::Server, None).await;
        (spawn_session(client, 1000, Some(10), ready), server)
    }

    async fn emit(server: &mut WebSocketStream<DuplexStream>, value: serde_json::Value) {
        server
            .send(Message::Text(value.to_string().into()))
            .await
            .unwrap();
    }

    async fn wait_closed(gateway: &mut Gateway) {
        timeout(Duration::from_secs(3), async {
            while gateway.events_rx.recv().await.is_some() {}
        })
        .await
        .expect("gateway must close the event stream");
    }

    #[tokio::test]
    async fn reconnect_and_invalid_session_close_the_stream_with_correct_recovery() {
        for (payload, expected) in [
            (serde_json::json!({"op": 7}), DisconnectReason::Resume),
            (
                serde_json::json!({"op": 9, "d": true}),
                DisconnectReason::Resume,
            ),
            (
                serde_json::json!({"op": 9, "d": false}),
                DisconnectReason::Identify,
            ),
        ] {
            let (mut gateway, mut server) = pair(None).await;
            emit(&mut server, payload).await;
            wait_closed(&mut gateway).await;
            assert_eq!(*gateway.disconnect_rx.borrow(), expected);
        }
    }

    #[tokio::test]
    async fn close_codes_distinguish_fatal_and_expired_sessions() {
        for (code, expected) in [
            (4004, DisconnectReason::Fatal(4004)),
            (4014, DisconnectReason::Fatal(4014)),
            (4007, DisconnectReason::Identify),
            (4009, DisconnectReason::Identify),
            (4000, DisconnectReason::Resume),
        ] {
            let (mut gateway, mut server) = pair(None).await;
            server
                .send(Message::Close(Some(CloseFrame {
                    code: CloseCode::from(code),
                    reason: "test".into(),
                })))
                .await
                .unwrap();
            wait_closed(&mut gateway).await;
            assert_eq!(*gateway.disconnect_rx.borrow(), expected);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn missing_heartbeat_ack_disconnects_a_live_socket() {
        let (mut gateway, mut server) = pair(None).await;
        emit(
            &mut server,
            serde_json::json!({"op": 0, "t": "RESUMED", "s": 11, "d": {}}),
        )
        .await;
        gateway.events_rx.recv().await.unwrap();
        // Keep the peer alive without acknowledging any heartbeats.
        wait_closed(&mut gateway).await;
        assert_eq!(*gateway.disconnect_rx.borrow(), DisconnectReason::Resume);
    }

    #[tokio::test]
    async fn requested_heartbeat_uses_last_queued_sequence() {
        let (mut gateway, mut server) = pair(None).await;
        emit(
            &mut server,
            serde_json::json!({"op": 0, "t": "RESUMED", "s": 42, "d": {}}),
        )
        .await;
        gateway.events_rx.recv().await.unwrap();
        emit(&mut server, serde_json::json!({"op": 1})).await;
        timeout(Duration::from_secs(2), async {
            loop {
                if let Message::Text(text) = server.next().await.unwrap().unwrap() {
                    let frame: serde_json::Value = serde_json::from_str(&text).unwrap();
                    if frame["op"] == 1 && frame["d"] == 42 {
                        break;
                    }
                }
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn queue_overflow_preserves_resume_sequence() {
        let (mut gateway, mut server) = pair(None).await;
        for seq in 11..=11 + EVENT_CAPACITY as i64 {
            emit(
                &mut server,
                serde_json::json!({"op": 0, "t": "MESSAGE_CREATE", "s": seq, "d": {}}),
            )
            .await;
        }
        // Await task termination without draining the deliberately full queue.
        timeout(Duration::from_secs(2), &mut gateway.task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            *gateway.last_seq_rx.borrow(),
            Some(10 + EVENT_CAPACITY as i64)
        );
        assert_eq!(gateway.events_rx.len(), EVENT_CAPACITY);
        assert_eq!(*gateway.disconnect_rx.borrow(), DisconnectReason::Resume);
    }

    #[tokio::test]
    async fn malformed_ready_returns_an_error_without_panicking() {
        let (tx, rx) = oneshot::channel();
        let (mut gateway, mut server) = pair(Some(tx)).await;
        emit(
            &mut server,
            serde_json::json!({"op": 0, "t": "READY", "s": 1, "d": {}}),
        )
        .await;
        assert!(rx.await.unwrap().is_err());
        wait_closed(&mut gateway).await;
    }

    #[tokio::test]
    async fn dropping_gateway_stops_the_socket_task() {
        let (gateway, mut server) = pair(None).await;
        drop(gateway);
        timeout(Duration::from_secs(2), async {
            while let Some(Ok(_)) = server.next().await {}
        })
        .await
        .expect("dropping a session must release its socket");
    }

    #[test]
    fn resume_url_keeps_host_and_forces_supported_protocol() {
        assert_eq!(
            normalize_gateway_url("wss://gateway.example/?v=9&encoding=etf").unwrap(),
            "wss://gateway.example/?v=10&encoding=json"
        );
        assert!(normalize_gateway_url("http://gateway.example").is_err());
    }

    #[tokio::test]
    async fn ready_returns_session_metadata_and_queues_the_event() {
        let (tx, rx) = oneshot::channel();
        let (mut gateway, mut server) = pair(Some(tx)).await;
        emit(
            &mut server,
            serde_json::json!({
                "op": 0, "t": "READY", "s": 1,
                "d": {"session_id": "session", "resume_gateway_url": "wss://gateway.example"}
            }),
        )
        .await;
        let info = rx.await.unwrap().unwrap();
        assert_eq!(info.session_id, "session");
        assert_eq!(info.resume_gateway_url, "wss://gateway.example");
        let event = gateway.events_rx.recv().await.unwrap();
        assert_eq!(event.t, GatewayDispatchEvents::Ready);
        assert_eq!(*gateway.last_seq_rx.borrow(), Some(1));
    }

    #[tokio::test]
    async fn replay_larger_than_queue_can_be_consumed_before_resumed() {
        let (mut gateway, mut server) = pair(None).await;
        for seq in 11..=11 + EVENT_CAPACITY as i64 {
            emit(
                &mut server,
                serde_json::json!({
                    "op": 0, "t": "MESSAGE_CREATE", "s": seq, "d": {}
                }),
            )
            .await;
            assert_eq!(gateway.events_rx.recv().await.unwrap().s, seq);
        }
        emit(
            &mut server,
            serde_json::json!({
                "op": 0, "t": "RESUMED", "s": 12 + EVENT_CAPACITY, "d": {}
            }),
        )
        .await;
        assert_eq!(
            gateway.events_rx.recv().await.unwrap().t,
            GatewayDispatchEvents::Resumed
        );
        assert!(!gateway.task.is_finished());
    }

    #[tokio::test(start_paused = true)]
    async fn acked_connection_stays_alive_across_heartbeat_intervals() {
        let (mut gateway, mut server) = pair(None).await;
        emit(
            &mut server,
            serde_json::json!({"op": 0, "t": "RESUMED", "s": 11, "d": {}}),
        )
        .await;
        gateway.events_rx.recv().await.unwrap();
        for _ in 0..3 {
            let Message::Text(text) = server.next().await.unwrap().unwrap() else {
                panic!("expected heartbeat");
            };
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&text).unwrap()["op"],
                1
            );
            emit(&mut server, serde_json::json!({"op": 11})).await;
        }
        emit(
            &mut server,
            serde_json::json!({"op": 0, "t": "MESSAGE_CREATE", "s": 12, "d": {}}),
        )
        .await;
        assert_eq!(gateway.events_rx.recv().await.unwrap().s, 12);
    }

    #[tokio::test(start_paused = true)]
    async fn missing_ready_times_out_even_when_heartbeats_are_acknowledged() {
        let (tx, rx) = oneshot::channel();
        let (mut gateway, mut server) = pair(Some(tx)).await;
        let peer = tokio::spawn(async move {
            while let Some(Ok(Message::Text(_))) = server.next().await {
                if server
                    .send(Message::Text(r#"{"op":11}"#.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        let error = rx.await.unwrap().err().expect("missing READY must fail");
        assert!(error.to_string().contains("timed out waiting for READY"));
        wait_closed(&mut gateway).await;
        peer.await.unwrap();
    }
}
