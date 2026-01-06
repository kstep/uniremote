use crate::{ActionId, RemoteId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "update")]
    Update {
        #[serde(skip_serializing)]
        remote_id: RemoteId,
        action: ActionId,
        args: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { 
        #[serde(skip_serializing)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "call")]
    CallAction(crate::CallActionRequest),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_serialization_skips_remote_id() {
        let msg = ServerMessage::Update {
            remote_id: RemoteId::from("test.remote"),
            action: ActionId::from("info"),
            args: serde_json::json!({"id": "info", "text": "foobar"}),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        // remote_id should not be in the serialized JSON
        assert!(!json.contains("remote_id"));
        assert!(json.contains(r#""type":"update""#));
        assert!(json.contains(r#""action":"info""#));
    }

    #[test]
    fn test_server_message_deserialization_with_remote_id() {
        let json = r#"{"type":"update","remote_id":"other.remote","action":"btn","args":{"id":"btn"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ServerMessage::Update { remote_id, action, .. } => {
                assert_eq!(&*remote_id, "other.remote");
                assert_eq!(&*action, "btn");
            }
            _ => panic!("Expected Update variant"),
        }
    }
}
