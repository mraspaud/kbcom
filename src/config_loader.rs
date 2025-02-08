use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

// Assume these modules exist and provide the relevant types.
use crate::chat_backend;
use crate::chat_backend::ChatBackend;
use crate::dummy_backend;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(flatten)]
    services: HashMap<String, ServiceConfig>,
}

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    backend: String,
    // Options for Rocket.Chat; for other backends these can be omitted.
    #[serde(default)]
    server_url: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

/// Reads the configuration file at `config_path` and instantiates all backend
/// instances defined in the file. Each service is specified as its own table.
/// For example:
///
/// ```toml
/// [some_dummy_service]
/// backend = "dummy"
///
/// [some_rocketchat_service]
/// backend = "rocketchat"
/// server_url = "ws://chat.example.com/websocket"
/// username = "my_username"
/// password = "my_password"
/// ```
///
/// Returns a mapping from service names to the corresponding backend instances.
pub async fn load_config_and_instantiate_backend(
    config_path: &str,
) -> HashMap<String, Arc<Mutex<Box<dyn ChatBackend + Send + Sync>>>> {
    let config_str = fs::read_to_string(config_path)
        .expect("Failed to read configuration file");
    let config: Config =
        toml::from_str(&config_str).expect("Failed to parse configuration file");

    let mut backends = HashMap::new();
    for (service_name, service_config) in config.services.into_iter() {
        match service_config.backend.as_str() {
            "dummy" => {
                // Cast DummyBackend into a trait object.
                let backend: Box<dyn ChatBackend + Send + Sync> =
                    Box::new(dummy_backend::DummyBackend::new())
                        as Box<dyn ChatBackend + Send + Sync>;
                backends.insert(service_name, Arc::new(Mutex::new(backend)));
            }
            // "rocketchat" => {
            //     let server_url = service_config.server_url.clone()
            //         .expect("Missing server_url for rocketchat");
            //     let username = service_config.username.clone()
            //         .expect("Missing username for rocketchat");
            //     let password = service_config.password.clone()
            //         .expect("Missing password for rocketchat");
            //
            //     let mut rc_backend = chat_backend::rocket_backend::RocketChatBackend::new(&server_url);
            //     // Login asynchronously; _token is ignored here.
            //     let _token = rc_backend.login(&username, &password).await
            //         .expect("RocketChat login failed");
            //     let backend: Box<dyn ChatBackend + Send + Sync> =
            //         Box::new(rc_backend) as Box<dyn ChatBackend + Send + Sync>;
            //     backends.insert(service_name, Arc::new(Mutex::new(backend)));
            // }
            other => panic!("Unsupported backend: {}", other),
        }
    }
    backends
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::sync::Mutex;
    use crate::chat_backend::{ChatBackend, BackendEvent};

    #[tokio::test]
    async fn test_load_config_and_instantiate_backend() {
        // Create a temporary TOML config file with one service table for a dummy backend.
        let config_content = r#"
[some_dummy_service]
backend = "dummy"
        "#;

        let mut temp_file =
            NamedTempFile::new().expect("Failed to create temporary config file");
        write!(temp_file, "{}", config_content).expect("Failed to write config content");
        let config_path = temp_file.path().to_str().unwrap();

        // Load the backend(s) from the configuration file.
        let backends = load_config_and_instantiate_backend(config_path).await;
        // Verify that we have an entry for "some_dummy_service".
        assert!(backends.contains_key("some_dummy_service"),
            "Expected service 'some_dummy_service' to be loaded");

        // Retrieve the backend and check that list_channels returns a ChannelList event.
        let backend = backends.get("some_dummy_service").unwrap();
        let event = backend.lock().await.list_channels();
        match event {
            BackendEvent::ChannelList { ref channels } => {
                assert!(!channels.is_empty(),
                    "Dummy backend should return at least one channel in the ChannelList event");
            }
            _ => panic!("Expected a ChannelList event from the dummy backend"),
        }
    }
}
