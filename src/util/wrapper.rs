//! Wrapper utility module
//!
//! Wrapping a Type with a `ModelWrapper` allows it direct access to a `RestClient`

use crate::rest::RestClient;
use std::ops::{Deref, DerefMut};

/// Utility to wrap a Model with access to a [`RestClient`]
///
/// by itself this type does not have any methods associated with it
#[derive(Debug)]
pub struct RestWrapper<T> {
    inner: T,
    rest_client: RestClient,
}

impl<T> RestWrapper<T> {
    /// Wrap a given T with a RestClient
    pub fn new(inner: T, rest_client: RestClient) -> Self {
        RestWrapper { inner, rest_client }
    }

    /// Extracts the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the inner RestClient
    pub fn rest_client(&self) -> &RestClient {
        &self.rest_client
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
