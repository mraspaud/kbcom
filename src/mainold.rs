use std::sync::{Arc,Mutex};
use tokio::time::{sleep, Duration};
use tokio::net::UnixListener;
use tokio::io::AsyncReadExt;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

mod chat_backend;
mod dummy_backend;
mod config;
use chat_backend::{ChatBackend, MyBackendState, BackendEvent, Channel};
use dummy_backend::DummyBackend;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use futures::StreamExt;

impl MyBackendState {
    pub fn new() -> Self {
        // Create dummy channels for demonstration.
        Self {
            channels: vec![
                Channel {
                    id: "dummy_channel_1".to_string(),
                    name: "Dummy Channel 1".to_string(),
                },
                Channel {
                    id: "dummy_channel_2".to_string(),
                    name: "Dummy Channel oj".to_string(),
                },
                Channel {
                    id: "dummy_channel_3".to_string(),
                    name: "Dummy Channel 3".to_string(),
                },
            ],
        }
    }
}

/// This asynchronous function streams events to stdout.
/// It first sends an event with the full channel list, then enters
/// a loop where it sends a new message event every second.

async fn stream_events<B>(backend: Arc<Mutex<B>>)
where
    B: ChatBackend + Send + 'static,
{
    // Send the initial channel list event.
    {
        //let state = state.lock().unwrap();
        let event = backend.lock().unwrap().list_channels();
        // Print the serialized JSON event.
        println!("{}", serde_json::to_string(&event).unwrap());
    }

    let stream = backend.lock().unwrap().get_messages();
    // Merge the streams into a single stream that yields messages from both sources.
    let mut combined_stream = futures::stream::select_all(vec![
        Box::pin(stream),
        // Box::pin(stream2),
    ]);

    // Process messages as they arrive from any of the dummy backends.
    while let Some(message) = combined_stream.next().await {
        println!("{}", serde_json::to_string(&message).unwrap());
    }
}

// Define a guard that will remove the socket file on drop.
struct UnixSocketGuard {
    path: String,
}

impl UnixSocketGuard {
    fn new(path: impl Into<String>) -> Self {
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

#[tokio::main]
async fn main() {
    // Initialize your backend state, connections, etc.
    let state = Arc::new(Mutex::new(MyBackendState::new()));
    // Create a shared backend instance.
    let backend = Arc::new(Mutex::new(DummyBackend::new()));
    // Start the task that streams events (channel list and messages).
    tokio::spawn(stream_events(backend.clone()));
    // Create the guard—this will ensure the file is removed when the program exits.
    let socket_path = "/tmp/chat_commands.sock";
    // Remove the socket file if it exists from previous runs.
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Failed to remove existing socket file");
    }
    let _socket_guard = UnixSocketGuard::new(socket_path);

     // Listen for incoming commands on a Unix socket.
    let listener = UnixListener::bind(socket_path).unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let backend = backend.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let cmd_str = String::from_utf8_lossy(&buf[..n]);
                    if let Ok(json_val) = serde_json::from_str::<Value>(&cmd_str) {
                        if let Some(cmd) = json_val.get("command").and_then(|c| c.as_str()) {
                            match cmd {
                                "post_message" => {
                                    let channel_id = json_val.get("channel_id").and_then(|v| v.as_str()).unwrap_or("");
                                    let body = json_val.get("body").and_then(|v| v.as_str()).unwrap_or("");
                                    backend.lock().unwrap().post_message(channel_id, body);
                                    // Process post_message; for example:
                                    // eprintln!("Posting message in {:?}: {:?}", channel_id, body);
                                    // Use your state to actually post the message.
                                }
                                "leave_channel" => {
                                    let channel_id = json_val.get("channel_id").and_then(|v| v.as_str());
                                    eprintln!("Leaving channel {:?}", channel_id);
                                    // Process the leave command.
                                }
                                _ => {
                                    println!("Unknown command: {}", cmd);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

