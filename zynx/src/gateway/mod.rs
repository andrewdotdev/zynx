pub mod heartbeat;
pub mod ws;

use serde::Deserialize;
use serde_json::value::RawValue;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;

use crate::models::gateway::GatewayDispatchEvents;

/// Recovery decision available after the event stream closes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Resume,
    Identify,
    Fatal(u16),
}

/// Active Gateway session. Dropping it cancels its socket task.
pub struct Gateway {
    pub session_id: String,
    pub resume_gateway_url: String,
    pub writer_tx: mpsc::Sender<Message>,
    /// Up to 256 dispatches, in wire order, with lazily decoded payloads.
    pub events_rx: mpsc::Receiver<crate::models::gateway::GatewayDispatch<Box<RawValue>>>,
    /// Sequence of the last dispatch successfully queued (or an unknown event skipped).
    pub last_seq_rx: watch::Receiver<Option<i64>>,
    pub shutdown_tx: watch::Sender<bool>,
    pub disconnect_rx: watch::Receiver<DisconnectReason>,
    pub(crate) task: JoinHandle<()>,
}

impl Gateway {
    /// Close the connection and wait for the socket task to finish.
    pub async fn close(&mut self) {
        let _ = self.shutdown_tx.send(true);
        let _ = (&mut self.task).await;
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub trait EventPayload: for<'de> Deserialize<'de> + Send + 'static {
    const EVENT: GatewayDispatchEvents;
}

impl EventPayload for crate::models::gateway::GatewayMessageCreateDispatchData {
    const EVENT: GatewayDispatchEvents = GatewayDispatchEvents::MessageCreate;
}
