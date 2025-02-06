mod chat_backend; // Ensure this module declaration includes your trait definitions.
mod dummy_backend; // This includes your DummyBackend implementation.

// Import the trait and the backend implementation.
use std::thread;
use std::env;
use std::time::Duration;
use chat_backend::{ChatBackend, display_message};
use dummy_backend::DummyBackend;

fn main() {
    let args: Vec<String> = env::args().collect();
    let backend = DummyBackend;

    // List channels and pick the first one.
    let channels = backend.list_channels();
    if let Some(first_channel) = channels.first() {
        if args.contains(&"--live".to_string()) {
            loop {
                if let Some(messages) = backend.get_messages(&first_channel.id) {
                    // Print each message.
                    for message in messages {
                        println!("{}", display_message(&message));
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        } else {
            if let Some(messages) = backend.get_messages(&first_channel.id) {
                // Print each message.
                for message in messages {
                    println!("{}: {}", message.id, message.content);
                }
            } else {
                println!("No messages found in channel {}", first_channel.id);
            }
        }
    } else {
        println!("No channels available");
    }
}

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::prelude::*;

// A shared state for the active channel.
#[derive(Clone)]
struct BackendState {
    active_channel: Arc<Mutex<String>>,
}

impl BackendState {
    fn new(initial: &str) -> Self {
        Self {
            active_channel: Arc::new(Mutex::new(initial.to_string())),
        }
    }
    fn set_channel(&self, new_channel: &str) {
        let mut chan = self.active_channel.lock().unwrap();
        *chan = new_channel.to_string();
    }
    fn get_channel(&self) -> String {
        self.active_channel.lock().unwrap().clone()
    }
}

#[tokio::main]
async fn main() {
    // Initialize your backend state with a default channel.
    let state = BackendState::new("dummy_channel_1");

    // Spawn a task to listen for channel-switch commands.
    let state_for_socket = state.clone();
    tokio::spawn(async move {
        // Listen on a Unix domain socket (or TCP port).
        let listener = UnixListener::bind("/tmp/chat_backend.sock").unwrap();
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            let command = String::from_utf8_lossy(&buf[..n]);
            // A simple protocol: if the command starts with "switch ", update the channel.
            if command.starts_with("switch ") {
                let new_channel = command.trim_start_matches("switch ").trim();
                println!("Switching channel to {}", new_channel);
                state_for_socket.set_channel(new_channel);
            }
        }
    });

    // Main message loop: fetch and print messages based on the active channel.
    loop {
        let current_channel = state.get_channel();
        // (Replace this with your actual message fetching logic)
        println!("Message from {}: Random number {}", current_channel, rand::random::<u32>());
        thread::sleep(Duration::from_secs(1));
    }
}
