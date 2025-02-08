use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

use crate::chat_backend::{ChatBackend, BackendEvent};

/// A guard that removes the Unix socket file when dropped.
pub struct UnixSocketGuard {
    pub path: String,
}

impl UnixSocketGuard {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Drop for UnixSocketGuard {
    fn drop(&mut self) {
        if Path::new(&self.path).exists() {
            match fs::remove_file(&self.path) {
                Ok(_) => println!("Socket file {} removed.", self.path),
                Err(e) => eprintln!("Failed to remove socket file {}: {}", self.path, e),
            }
        }
    }
}

/// Processes a single command received over the Unix socket.
/// Expects the command JSON to contain a "service" field to determine which backend to use.
pub async fn process_command(
    mut socket: UnixStream,
    backends: Arc<Mutex<HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>>>>,
) {
    let mut buf = vec![0; 1024];
    if let Ok(n) = socket.read(&mut buf).await {
        if n > 0 {
            let cmd_str = String::from_utf8_lossy(&buf[..n]);
            if let Ok(json_val) = serde_json::from_str::<Value>(&cmd_str) {
                if let Some(cmd) = json_val.get("command").and_then(|c| c.as_str()) {
                    // Expect a "service" field to know which backend to use.
                    let service = json_val
                        .get("service")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");
                    let backends_guard = backends.lock().await;
                    if let Some(backend_instance) = backends_guard.get(service) {
                        match cmd {
                            "post_message" => {
                                let channel_id = json_val
                                    .get("channel_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let body = json_val
                                    .get("body")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if let Err(e) = backend_instance
                                    .lock()
                                    .await
                                    .post_message(channel_id, body)
                                    .await
                                {
                                    eprintln!("Failed to post message: {:?}", e);
                                }
                            }
                            "leave_channel" => {
                                let channel_id = json_val
                                    .get("channel_id")
                                    .and_then(|v| v.as_str());
                                eprintln!(
                                    "Service {} leaving channel {:?}",
                                    service, channel_id
                                );
                                // Add additional handling here if needed.
                            }
                            _ => {
                                println!("Unknown command: {}", cmd);
                            }
                        }
                    } else {
                        eprintln!("Service '{}' not found", service);
                    }
                }
            }
        }
    }
}

/// Creates a Unix socket at `socket_path`, and enters a loop accepting connections
/// and processing commands using `process_command`.
pub async fn run_command_socket(
    socket_path: &str,
    backends: Arc<Mutex<HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>>>>,
) {
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Failed to remove existing socket file");
    }
    let _socket_guard = UnixSocketGuard::new(socket_path);
    let listener = UnixListener::bind(socket_path).expect("Failed to bind to Unix socket");

    loop {
        let (socket, _) = listener.accept().await.expect("Failed to accept connection");
        let backends_clone = backends.clone();
        tokio::spawn(async move {
            process_command(socket, backends_clone).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::net::UnixStream;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::Mutex;
    use serde_json::json;
    use futures::stream::Stream;
    use std::pin::Pin;

    // Assume that ChatBackend, BackendEvent, LoginError, and PostError are defined in your crate.
    use crate::chat_backend::{ChatBackend, BackendEvent};

    // Define a simple test backend that records calls to post_message.
    struct TestBackend {
        pub posted_messages: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl TestBackend {
        fn new() -> Self {
            Self { posted_messages: Arc::new(Mutex::new(vec![])) }
        }
    }

    #[async_trait::async_trait]
    impl ChatBackend for TestBackend {
        async fn login(&self, _username: &str, _password: &str) -> Result<String, crate::chat_backend::LoginError> {
            Ok("dummy_token".to_string())
        }

        fn list_channels(&self) -> BackendEvent {
            // For testing, return an empty channel list.
            BackendEvent::ChannelList { channels: vec![] }
        }

        fn get_messages(&self) -> Pin<Box<dyn Stream<Item = BackendEvent> + Send>> {
            // Return an empty stream.
            Box::pin(futures::stream::empty())
        }

        async fn post_message(&self, channel_id: &str, content: &str) -> Result<(), crate::chat_backend::PostError> {
            let mut msgs = self.posted_messages.lock().await;
            msgs.push((channel_id.to_string(), content.to_string()));
            Ok(())
        }
    }

    // Test for process_command with a valid post_message command.
    #[tokio::test]
    async fn test_process_command_post_message() {
        // Create a test backend and a HashMap mapping service "test_service" to it.
        let test_backend = TestBackend::new();
        let posted_messages = test_backend.posted_messages.clone();

        let mut services: HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>> = HashMap::new();
        services.insert(
            "test_service".to_string(),
            Arc::new(Mutex::new(Box::new(test_backend) as Box<dyn ChatBackend + Send + Sync>))
        );
        let backends = Arc::new(Mutex::new(services));

        // Create a pair of connected UnixStream sockets.
        let (mut client, server) = UnixStream::pair().unwrap();

        // Prepare a JSON command for "post_message".
        let command = json!({
            "command": "post_message",
            "service": "test_service",
            "channel_id": "channel123",
            "body": "Hello, test!"
        });
        let command_str = command.to_string();

        // Write the command to the client side and shutdown writing.
        client.write_all(command_str.as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();

        // Process the command on the server side.
        process_command(server, backends.clone()).await;

        // Check that the test backend recorded the post_message call.
        let msgs = posted_messages.lock().await;
        assert_eq!(msgs.len(), 1, "Expected one posted message");
        assert_eq!(msgs[0].0, "channel123");
        assert_eq!(msgs[0].1, "Hello, test!");
    }

    // Test for process_command with an unknown service.
    #[tokio::test]
    async fn test_process_command_unknown_service() {
        // Create an empty backend mapping.
        let services: HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>> = HashMap::new();
        let backends = Arc::new(Mutex::new(services));

        // Create a UnixStream pair.
        let (mut client, server) = UnixStream::pair().unwrap();

        // Prepare a JSON command with a nonexistent service.
        let command = json!({
            "command": "post_message",
            "service": "nonexistent_service",
            "channel_id": "channel123",
            "body": "Hello, test!"
        });
        let command_str = command.to_string();
        client.write_all(command_str.as_bytes()).await.unwrap();
        client.shutdown().await.unwrap();

        // Call process_command. It should print an error but not panic.
        process_command(server, backends.clone()).await;
        // Nothing to assert—just ensure no panic occurs.
    }
}
