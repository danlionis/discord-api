//! Wrapper utility module
//!
//! Wrapping a Type with a `ModelWrapper` allows it direct access to a `RestClient`

use crate::api::Api;
use std::ops::{Deref, DerefMut};

/// Utility to wrap a Model with access to a [`RestClient`]
///
/// by itself this type does not have any methods associated with it
#[derive(Debug)]
pub struct ApiWrapper<T> {
    inner: T,
    api: Api,
}

impl<T> ApiWrapper<T> {
    /// Wrap a given T with a ApiClient
    pub fn new(inner: T, api: Api) -> Self {
        ApiWrapper { inner, api }
    }

    /// Extracts the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the inner RestClient
    pub fn api(&self) -> &Api {
        &self.api
    }
}

impl<T> Deref for ApiWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ApiWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
