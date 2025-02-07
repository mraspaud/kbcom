use serde::{Serialize, Deserialize};
use futures::Stream;
use std::sync::{Arc, Mutex};
use std::pin::Pin;

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    ConnectionError(String),
    // Add other variants as needed
}

// Optionally, implement Display for better error messages:
use std::fmt;

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginError::InvalidCredentials => write!(f, "Invalid credentials provided"),
            LoginError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

// And implement the Error trait so it integrates with other error handling utilities:
impl std::error::Error for LoginError {}

#[derive(Debug)]
pub enum PostError {
    ChannelNotFound,
    PermissionDenied,
    ConnectionError(String),
    // Add other variants as needed.
}

impl fmt::Display for PostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostError::ChannelNotFound => write!(f, "Channel not found"),
            PostError::PermissionDenied => write!(f, "Permission denied"),
            PostError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for PostError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    // You can add other fields such as description, members, etc.
}


/// Backend state containing the list of channels.
/// In a more complete implementation, you might also store
/// persistent connections or other state here.
#[derive(Debug)]
pub struct MyBackendState {
    pub channels: Vec<Channel>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub channel_id: String,
    pub author: String,
    pub content: String,
    // Optionally, add other fields like a timestamp.
}

pub fn display_message(message: &Message) -> String {
    format!(
        "{{\"message_id\": {}, \"channel_id\": \"{}\", \"author\": \"{}\", \"body\": \"{}\"}}",
        message.id, message.channel_id, message.author, message.content
    )
}

/// Events that the backend sends to frontends, serialized as JSON.
/// The `#[serde(tag = "event")]` attribute means that each variant
/// will include an `"event"` field in the JSON output.
#[derive(Debug, Serialize)]
#[serde(tag = "event")]
pub enum BackendEvent {
    #[serde(rename = "channel_list")]
    ChannelList { channels: Vec<Channel> },
    #[serde(rename = "message")]
    Message { channel_id: String, message_id: u64, body: String, author: String },
}

pub trait ChatBackend {
    fn login(&self, username: &str, password: &str) -> Result<String, LoginError>;
    fn list_channels(&self) -> BackendEvent;
    fn get_messages(&self) -> Pin<Box<dyn Stream<Item = BackendEvent> + Send>>;
    fn post_message(&self, channel_id: &str, content: &str) -> Result<(), PostError>;
}


