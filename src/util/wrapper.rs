//! Wrapper utility module
//!
//! Wrapping a Type with a `ModelWrapper` allows it direct access to a `RestClient`

use crate::rest::Client;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// Utility to wrap a Model with access to a [`RestClient`]
///
/// by itself this type does not have any methods associated with it
#[derive(Debug)]
pub struct RestWrapper<T> {
    inner: T,
    rest: Arc<Client>,
}

impl<T> RestWrapper<T> {
    /// Wrap a given T with a ApiClient
    pub fn new(inner: T, api: Arc<Client>) -> Self {
        RestWrapper { inner, rest: api }
    }

    /// Extracts the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the inner RestClient
    pub fn api(&self) -> &Client {
        &self.rest
    }
}

impl<T> Deref for RestWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for RestWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
