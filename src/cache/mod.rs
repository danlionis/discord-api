//! cache events for later access

use crate::model::gateway::Event;
use crate::model::{id::MessageId, Message, User};
use lru::LruCache;
use std::fmt::Debug;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::Instant;

/// Discord Cache
#[derive(Debug)]
pub struct Cache {
    messages: RwLock<LruCache<MessageId, Message>>,
    connected_since: RwLock<Option<Instant>>,
    user: RwLock<Option<User>>,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            connected_since: Default::default(),
            messages: RwLock::new(LruCache::new(1024)),
            user: Default::default(),
        }
    }
}

impl Cache {
    /// Create a new cache
    pub fn new() -> Self {
        Cache::default()
    }

    /// Get a reference to all messages
    pub fn messages(&self) -> RwLockReadGuard<'_, LruCache<MessageId, Message>> {
        self.messages.read().unwrap()
    }

    /// Get a reference to the current user
    pub fn user(&self) -> RwLockReadGuard<'_, Option<User>> {
        self.user.read().unwrap()
    }

    /// Get the time since connected
    pub fn connected_since(&self) -> Option<Instant> {
        self.connected_since.read().unwrap().clone()
    }

    /// Update the cache with a new event
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Ready(ready) => {
                let mut connected_since = self.connected_since.write().unwrap();
                *connected_since = Some(Instant::now());

                let mut user = self.user.write().unwrap();
                *user = Some(ready.user.clone());
            }
            Event::MessageCreate(msg) => {
                if let Ok(mut messages) = self.messages.write() {
                    messages.put(msg.id, *msg.clone());
                }
            }
            Event::MessageUpdate(_msg) => {
                // update message
            }
            _ => {}
        }
    }
}
