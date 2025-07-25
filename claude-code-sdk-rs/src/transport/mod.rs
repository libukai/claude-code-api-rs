//! Transport layer abstractions
//!
//! This module defines the Transport trait and its implementations for
//! communicating with the Claude CLI.

use crate::{
    errors::Result,
    types::{ControlRequest, ControlResponse, Message},
};
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

pub mod subprocess;

pub use subprocess::SubprocessTransport;

/// Input message structure for sending to Claude
#[derive(Debug, Clone, serde::Serialize)]
pub struct InputMessage {
    /// Message type (always "user")
    #[serde(rename = "type")]
    pub r#type: String,
    /// Message content
    pub message: serde_json::Value,
    /// Parent tool use ID (for tool results)
    pub parent_tool_use_id: Option<String>,
    /// Session ID
    pub session_id: String,
}

impl InputMessage {
    /// Create a new user message
    pub fn user(content: String, session_id: String) -> Self {
        Self {
            r#type: "user".to_string(),
            message: serde_json::json!({
                "role": "user",
                "content": content
            }),
            parent_tool_use_id: None,
            session_id,
        }
    }

    /// Create a tool result message
    pub fn tool_result(
        tool_use_id: String,
        content: String,
        session_id: String,
        is_error: bool,
    ) -> Self {
        Self {
            r#type: "user".to_string(),
            message: serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": tool_use_id,
                    "content": content,
                    "is_error": is_error
                }]
            }),
            parent_tool_use_id: Some(tool_use_id),
            session_id,
        }
    }
}

/// Transport trait for communicating with Claude CLI
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the Claude CLI
    async fn connect(&mut self) -> Result<()>;

    /// Send a message to Claude
    async fn send_message(&mut self, message: InputMessage) -> Result<()>;

    /// Receive messages from Claude as a stream
    fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>>;

    /// Send a control request (e.g., interrupt)
    async fn send_control_request(&mut self, request: ControlRequest) -> Result<()>;

    /// Receive control responses
    async fn receive_control_response(&mut self) -> Result<Option<ControlResponse>>;

    /// Check if the transport is connected
    #[allow(dead_code)]
    fn is_connected(&self) -> bool;

    /// Disconnect from the Claude CLI
    async fn disconnect(&mut self) -> Result<()>;
}

/// Transport state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    /// Not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected and ready
    Connected,
    /// Disconnecting
    Disconnecting,
    /// Error state
    #[allow(dead_code)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_message_user() {
        let msg = InputMessage::user("Hello".to_string(), "session-123".to_string());
        assert_eq!(msg.r#type, "user");
        assert_eq!(msg.session_id, "session-123");
        assert!(msg.parent_tool_use_id.is_none());

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"user""#));
        assert!(json.contains(r#""content":"Hello""#));
    }

    #[test]
    fn test_input_message_tool_result() {
        let msg = InputMessage::tool_result(
            "tool-123".to_string(),
            "Result".to_string(),
            "session-456".to_string(),
            false,
        );
        assert_eq!(msg.r#type, "user");
        assert_eq!(msg.parent_tool_use_id, Some("tool-123".to_string()));

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""tool_use_id":"tool-123""#));
        assert!(json.contains(r#""is_error":false"#));
    }
}