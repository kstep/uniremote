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
            while let Ok(CallActionRequest { action, args }) = inbox.recv_async().await {
                if let Err(error) = state.call_action(action, args) {
                    tracing::error!("failed to handle action request: {error:#}");
                }
            }
        });
    }

    pub fn add_state<T: Send + 'static>(&self, state: T) {
        self.state.add_state(state);
    }

    pub fn subscribe(&self) -> Receiver<ServerMessage> {
        self.outbox.clone()
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
