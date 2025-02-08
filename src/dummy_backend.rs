use rand::Rng;
use crate::chat_backend::{Message, Channel, LoginError, PostError, BackendEvent}; // adjust the path based on your project structure
use crate::chat_backend::ChatBackend;
use async_stream::stream;
use futures::Stream;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use std::pin::Pin;
use async_trait::async_trait;

pub struct DummyBackend {
    posted_messages: Arc<Mutex<Vec<BackendEvent>>>,
}

impl DummyBackend {
    pub fn new() -> Self {
        DummyBackend {
            posted_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl ChatBackend for DummyBackend {
    // Implement the trait methods here.
    async fn login(&self, username: &str, password: &str) -> Result<String, LoginError> {
        Ok("dummy_session_token".to_string())
    }
    fn list_channels(&self) -> BackendEvent {
        BackendEvent::ChannelList {
            channels: vec![
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
    }

    fn get_messages(&self) -> Pin<Box<dyn Stream<Item = BackendEvent> + Send>> {
        let extra_messages = self.posted_messages.clone();
        let s = stream! {
            let mut message_id = 1u64;
            loop {
                // First, yield any messages that were posted (and clear the table)
                // Extract posted messages from the table without holding the lock across an await.
                let posted_msgs = {
                    let mut table = extra_messages.lock().unwrap();
                    std::mem::take(&mut *table)
                };
                // Yield each posted message.
                for msg in posted_msgs {
                    yield msg;
                }
                let msg1 = BackendEvent::Message {
                    message_id: message_id,
                    channel_id: "dummy_channel1".to_string(),
                    author: "Dummy Author".to_string(),
                    body: format!("Random message: {}", message_id),
                };
                yield msg1;
                message_id += 1;
                let msg2 = BackendEvent::Message {
                    message_id: message_id,
                    channel_id: "dummy_channel2".to_string(),
                    author: "Another Dummy Author".to_string(),
                    body: format!("Random message: {}", message_id),
                };
                yield msg2;
                message_id += 1;
                sleep(Duration::from_millis(500)).await;
            }
        };

        Box::pin(s)
    }

    async fn post_message(&self, channel_id: &str, content: &str) -> Result<(), PostError> {
        let message = BackendEvent::Message {
            message_id: 0u64,
            channel_id: channel_id.to_string(),
            author: "Good old me".to_string(),
            body: content.to_string(),
        };
        let mut table = self.posted_messages.lock().unwrap();
        println!("pushing message");
        table.push(message);
        Ok(())
    }
}
