use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::anyhow;
use flume::{Receiver, SendError, Sender};
use tokio::{sync::Mutex, task::JoinHandle};
use uniremote_core::{CallActionRequest, ServerMessage};
use uniremote_lua::LuaState;

mod subscription;
pub use subscription::Subscription;

const CHANNEL_BUFFER_SIZE: usize = 100;
const MAX_SEND_RETRIES: usize = 10;

struct LuaWorkerInner {
    started: AtomicBool,
    inbox: Receiver<CallActionRequest>,
    outbox: Receiver<ServerMessage>,
    state: Arc<LuaState>,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl Drop for LuaWorkerInner {
    fn drop(&mut self) {
        if let Some(task) = self.task.get_mut().take() {
            task.abort();
        }
    }
}

#[derive(Clone)]
pub struct LuaWorker {
    inner: Arc<LuaWorkerInner>,
    sender: Sender<CallActionRequest>,
}

impl LuaWorker {
    pub fn new(state: LuaState) -> Self {
        let (sender, inbox) = flume::bounded(CHANNEL_BUFFER_SIZE);
        let (outbox_tx, outbox) = flume::bounded(CHANNEL_BUFFER_SIZE);
        state.add_state(outbox_tx.clone());

        Self {
            inner: Arc::new(LuaWorkerInner {
                inbox,
                outbox,
                started: AtomicBool::new(false),
                state: Arc::new(state),
                task: Mutex::new(None),
            }),
            sender,
        }
    }

    async fn start(&self) {
        if self.inner.started.swap(true, Ordering::SeqCst) {
            return;
        }

        let inbox = self.inner.inbox.clone();
        let state = self.inner.state.clone();
        let task = tokio::spawn(async move {
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
        self.inner.task.lock().await.replace(task);
    }

    pub fn subscribe(&self) -> Subscription {
        Subscription::new(self.inner.outbox.clone(), self.inner.state.clone())
    }

    pub async fn send(&self, mut request: CallActionRequest) -> anyhow::Result<()> {
        self.start().await;

        for _ in 0..MAX_SEND_RETRIES {
            request = match self.sender.send_async(request).await {
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
