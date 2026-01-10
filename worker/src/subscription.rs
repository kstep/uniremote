use std::sync::Arc;

use flume::Receiver;
use uniremote_core::ServerMessage;
use uniremote_lua::LuaState;

/// A subscription to the outbox that tracks focus/blur events
pub struct Subscription {
    receiver: Receiver<ServerMessage>,
    state: Arc<LuaState>,
}

impl Subscription {
    /// Create a new subscription and trigger focus event if this is the first
    /// subscription.
    pub(crate) fn new(receiver: Receiver<ServerMessage>, state: Arc<LuaState>) -> Self {
        // Check if this is the first subscription.
        // After cloning, receiver_count will be at least 2 (original + this clone).
        // If it's exactly 2, we're creating the first subscription.
        tracing::info!(
            "new subscription, receiver count {}",
            receiver.receiver_count()
        );
        if receiver.receiver_count() == 2
            && let Err(error) = state.trigger_event("focus")
        {
            tracing::warn!("failed to trigger focus event: {error}");
        }

        Self { receiver, state }
    }

    /// Receive a message from the subscription
    pub async fn recv(&self) -> Result<ServerMessage, flume::RecvError> {
        self.receiver.recv_async().await
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Check if this is the last subscription being dropped
        // receiver_count() still includes this receiver (drop hasn't completed yet)
        // So if count is 2, it means: 1 master in worker + 1 this subscription
        // After this drop completes, only the master will remain
        tracing::info!(
            "subscription drop, receiver count {}",
            self.receiver.receiver_count()
        );
        if self.receiver.receiver_count() == 2
            && let Err(error) = self.state.trigger_event("blur")
        {
            tracing::warn!("failed to trigger blur event: {error}");
        }
    }
}
