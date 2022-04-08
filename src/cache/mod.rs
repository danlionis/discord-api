//! cache events for later access

use crate::model::gateway::Event;
use crate::model::{id::MessageId, Message, User};
use lru::LruCache;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Instant;

#[derive(Debug)]
pub struct Cache {
    inner: Arc<Inner>,
}

impl Cache {
    pub fn new() -> Self {
        CacheBuilder::new().build()
    }

    pub fn messages(&self) -> RwLockReadGuard<'_, LruCache<MessageId, Message>> {
        self.inner.messages.read().unwrap()
    }

    pub fn user(&self) -> RwLockReadGuard<'_, Option<User>> {
        self.inner.user.read().unwrap()
    }

    pub fn connected_since(&self) -> Option<Instant> {
        self.inner.connected_since.read().unwrap().clone()
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Cache {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(Debug)]
pub struct Inner {
    messages: RwLock<LruCache<MessageId, Message>>,
    connected_since: RwLock<Option<Instant>>,
    user: RwLock<Option<User>>,
}

impl Cache {
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Ready(ready) => {
                let mut connected_since = self.inner.connected_since.write().unwrap();
                *connected_since = Some(Instant::now());

                let mut user = self.inner.user.write().unwrap();
                *user = Some(ready.user.clone());
            }
            Event::MessageCreate(msg) => {
                if let Ok(mut messages) = self.inner.messages.write() {
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

pub struct CacheBuilder {
    message_cap: usize,
}

impl CacheBuilder {
    fn new() -> Self {
        CacheBuilder { message_cap: 1024 }
    }

    fn build(self) -> Cache {
        Cache {
            inner: Arc::new(Inner {
                connected_since: Default::default(),
                messages: RwLock::new(LruCache::new(self.message_cap)),
                user: Default::default(),
            }),
        }
    }

    fn message_cap(mut self, cap: usize) -> CacheBuilder {
        self.message_cap = cap;
        self
    }
}
