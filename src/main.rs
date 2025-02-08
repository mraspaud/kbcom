use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use futures::StreamExt;
use tokio::sync::Mutex;

mod chat_backend;
mod dummy_backend;
mod config_loader; // Contains load_config_and_instantiate_backend
mod command_processor; // Contains process_command and run_command_socket

use chat_backend::{ChatBackend, BackendEvent};
use config_loader::load_config_and_instantiate_backend;
use command_processor::run_command_socket;

/// Streams events for a single backend instance.
async fn stream_events(backend: Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>) {
    // Send the initial channel list event.
    {
        let event = backend.lock().await.list_channels();
        println!("{}", serde_json::to_string(&event).unwrap());
    }

    let mut stream = backend.lock().await.get_messages();
    while let Some(event) = stream.next().await {
        println!("{}", serde_json::to_string(&event).unwrap());
    }
}

#[tokio::main]
async fn main() {
    // --- Read configuration file path from command-line arguments ---
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_file_path>", args[0]);
        return;
    }
    let config_path = &args[1];

    // --- Load configuration and instantiate all backend instances ---
    let backend_map: HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>> =
        load_config_and_instantiate_backend(config_path).await;
    // Wrap the map in an Arc<Mutex<>> so it can be shared across tasks.
    let backends = Arc::new(Mutex::new(backend_map));

    // --- Spawn a task to stream events for each backend ---
    {
        let backends_guard = backends.lock().await;
        for (service, backend_instance) in backends_guard.iter() {
            let service_clone = service.clone();
            let backend_clone = backend_instance.clone();
            tokio::spawn(async move {
                println!("Spawning event stream for service: {}", service_clone);
                stream_events(backend_clone).await;
            });
        }
    }

    // --- Run the Unix socket command processor ---
    let socket_path = "/tmp/chat_commands.sock";
    run_command_socket(socket_path, backends.clone()).await;
}
