use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::anyhow;
use flume::{Receiver, SendError, Sender};
use uniremote_core::{CallActionRequest, ServerMessage};
use uniremote_lua::LuaState;

const CHANNEL_BUFFER_SIZE: usize = 100;
const MAX_SEND_RETRIES: usize = 10;

/// A subscription to the outbox that tracks focus/blur events
pub struct Subscription {
    receiver: Receiver<ServerMessage>,
    state: Arc<LuaState>,
}

impl Subscription {
    /// Create a new subscription and trigger focus event if this is the first subscription
    fn new(receiver: Receiver<ServerMessage>, state: Arc<LuaState>) -> Self {
        // Check if this is the first subscription
        // After cloning, receiver_count will be at least 2 (original + this clone)
        // If it's exactly 2, we're creating the first subscription
        if receiver.receiver_count() == 2
            && let Err(error) = state.trigger_event("focus")
        {
            tracing::warn!("failed to trigger focus event: {error}");
        }

        Self { receiver, state }
    }

    /// Receive a message from the subscription
    pub async fn recv_async(&self) -> Result<ServerMessage, flume::RecvError> {
        self.receiver.recv_async().await
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Check if this is the last subscription being dropped
        // receiver_count() still includes this receiver (drop hasn't completed yet)
        // So if count is 2, it means: 1 master in worker + 1 this subscription
        // After this drop completes, only the master will remain
        if self.receiver.receiver_count() == 2
            && let Err(error) = self.state.trigger_event("blur")
        {
            tracing::warn!("failed to trigger blur event: {error}");
        }
    }
}

#[derive(Clone)]
pub struct LuaWorker {
    started: Arc<AtomicBool>,
    inbox: Receiver<CallActionRequest>,
    inbox_tx: Sender<CallActionRequest>,
    outbox: Receiver<ServerMessage>,
    state: Arc<LuaState>,
}

impl LuaWorker {
    pub fn new(state: LuaState) -> Self {
        let (inbox_tx, inbox) = flume::bounded(CHANNEL_BUFFER_SIZE);
        let (outbox_tx, outbox) = flume::bounded(CHANNEL_BUFFER_SIZE);
        state.add_state(outbox_tx.clone());

        Self {
            inbox,
            inbox_tx,
            outbox,
            started: Arc::new(AtomicBool::new(false)),
            state: Arc::new(state),
        }
    }

    fn start(&self) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }

        let inbox = self.inbox.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            if let Err(error) = state.trigger_event("create") {
                tracing::error!("failed to run create event handler: {error}");
            }

            while let Ok(CallActionRequest { action, args }) = inbox.recv_async().await {
                if let Err(error) = state.call_action(action, args) {
                    tracing::error!("failed to handle action request: {error:#}");
                }
            }

            if let Err(error) = state.trigger_event("destroy") {
                tracing::error!("failed to run destroy event handler: {error}");
            }
        });
    }

    pub fn subscribe(&self) -> Subscription {
        Subscription::new(self.outbox.clone(), self.state.clone())
    }

    pub async fn send(&self, mut request: CallActionRequest) -> anyhow::Result<()> {
        self.start();

        for _ in 0..MAX_SEND_RETRIES {
            request = match self.inbox_tx.send_async(request).await {
                Ok(_) => return Ok(()),
                Err(SendError(request)) => {
                    tracing::warn!("worker is not ready, retrying send");
                    request
                }
            };
        }

        tracing::error!("failed to send action request to worker after {MAX_SEND_RETRIES} retries");
        Err(anyhow!("failed to send action request to worker"))
    }
}
