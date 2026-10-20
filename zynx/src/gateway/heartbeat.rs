use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use tokio::time::{Duration, Instant};

/// A heartbeat's ACK deadline is independent of server-requested heartbeats.
pub(crate) struct Heartbeat {
    interval: Duration,
    next_tick: Instant,
    ack_deadline: Option<Instant>,
}

impl Heartbeat {
    pub(crate) fn new(interval_ms: u64) -> Self {
        let interval = Duration::from_millis(interval_ms);
        let jitter = RandomState::new().hash_one(()) % interval_ms;
        Self {
            interval,
            next_tick: Instant::now() + Duration::from_millis(jitter),
            ack_deadline: None,
        }
    }

    pub(crate) fn deadline(&self) -> Instant {
        self.ack_deadline
            .map_or(self.next_tick, |ack| ack.min(self.next_tick))
    }

    pub(crate) fn tick(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        anyhow::ensure!(
            self.ack_deadline.is_none_or(|deadline| now < deadline),
            "gateway heartbeat acknowledgement timed out"
        );
        self.next_tick = now + self.interval;
        self.sent();
        Ok(())
    }

    pub(crate) fn sent(&mut self) {
        self.ack_deadline
            .get_or_insert_with(|| Instant::now() + self.interval);
    }

    pub(crate) fn acknowledge(&mut self) {
        self.ack_deadline = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn missing_ack_expires_even_with_additional_heartbeats() {
        let mut heartbeat = Heartbeat::new(1000);
        heartbeat.sent();
        tokio::time::advance(Duration::from_millis(900)).await;
        heartbeat.sent();
        tokio::time::advance(Duration::from_millis(100)).await;
        assert!(heartbeat.tick().is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn ack_allows_the_next_heartbeat() {
        let mut heartbeat = Heartbeat::new(1000);
        heartbeat.sent();
        heartbeat.acknowledge();
        tokio::time::advance(Duration::from_secs(1)).await;
        assert!(heartbeat.tick().is_ok());
    }
}
