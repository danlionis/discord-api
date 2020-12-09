use crate::model::gateway::Event;
use crate::model::{id::MessageId, Message};
use dashmap::DashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct Cache {
    inner: Arc<Inner>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            inner: Default::default(),
        }
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Cache {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(Default, Debug)]
pub struct Inner {
    messages: DashMap<MessageId, Message>,
    connected_since: RwLock<Option<Instant>>,
}

impl Cache {
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Ready(_usr) => {
                let mut connected_since = self.inner.connected_since.write().unwrap();
                *connected_since = Some(Instant::now());
            }
            Event::MessageCreate(msg) => {
                self.inner.messages.insert(msg.id, msg.clone());
            }
            Event::MessageUpdate(_msg) => {
                // update message
            }
            _ => {}
        }
    }
}
