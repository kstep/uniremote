use crate::{ActionId, RemoteId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Update {
        remote_id: RemoteId,
        action: ActionId,
        args: serde_json::Value,
    },
    Error { 
        remote_id: RemoteId,
        message: String 
    },
}

impl ServerMessage {
    pub fn remote_id(&self) -> &RemoteId {
        match self {
            ServerMessage::Update { remote_id, .. } => remote_id,
            ServerMessage::Error { remote_id, .. } => remote_id,
        }
    }
}

impl Serialize for ServerMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        match self {
            ServerMessage::Update { action, args, .. } => {
                let mut state = serializer.serialize_struct("ServerMessage", 3)?;
                state.serialize_field("type", "update")?;
                state.serialize_field("action", action)?;
                state.serialize_field("args", args)?;
                state.end()
            }
            ServerMessage::Error { message, .. } => {
                let mut state = serializer.serialize_struct("ServerMessage", 2)?;
                state.serialize_field("type", "error")?;
                state.serialize_field("message", message)?;
                state.end()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "call")]
    CallAction(crate::CallActionRequest),
}
