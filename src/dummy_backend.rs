use rand::Rng;
use crate::chat_backend::{Message, Channel, LoginError, PostError}; // adjust the path based on your project structure
use crate::chat_backend::ChatBackend;

pub struct DummyBackend;

impl ChatBackend for DummyBackend {
    // Implement the trait methods here.
    fn login(&self, username: &str, password: &str) -> Result<String, LoginError> {
        Ok("dummy_session_token".to_string())
    }
    fn list_channels(&self) -> Vec<Channel> {
        vec![
            Channel {
                id: "dummy_channel1".to_string(),
                name: "Dummy Channel1".to_string(),
            },
            Channel {
                id: "dummy_channel2".to_string(),
                name: "Dummy Channel2".to_string(),
            }
        ]
    }
    fn get_messages(&self, channel_id: &str) -> Option<Vec<Message>> {
        let mut rng = rand::rng();
        let msg1 = Message {
            id: rng.random::<u64>(),
            channel_id: channel_id.to_string(),
            author: "Dummy Author".to_string(),
            content: format!("Random message: {}", rng.random_range(0..100)),
        };
        let msg2 = Message {
            id: rng.random::<u64>(),
            channel_id: channel_id.to_string(),
            author: "Dummy Author".to_string(),
            content: format!("Random message: {}", rng.random_range(0..100)),
        };

        Some(vec![msg1, msg2])    // Implementation...
    }
    fn post_message(&self, channel_id: &str, author: &str, content: &str) -> Result<(), PostError> {
        Ok(())
    }
}
