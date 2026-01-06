use serde::{Deserialize, Serialize};

use crate::ActionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "update")]
    Update {
        action: ActionId,
        args: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
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
    fn test_server_message_serialization() {
        let msg = ServerMessage::Update {
            action: ActionId::from("info"),
            args: serde_json::json!({"id": "info", "text": "foobar"}),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"update""#));
        assert!(json.contains(r#""action":"info""#));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"update","action":"btn","args":{"id":"btn"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        match msg {
            ServerMessage::Update { action, .. } => {
                assert_eq!(&*action, "btn");
            }
            _ => panic!("Expected Update variant"),
        }
    }
}
