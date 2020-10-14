use crate::rest::RestClient;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Utility to wrap a Model with access to a [`RestClient`]
#[derive(Debug)]
pub struct ModelWrapper<T> {
    inner: T,
    rest_client: Arc<RestClient>,
}

impl<T> ModelWrapper<T> {
    pub fn new(inner: T, rest_client: Arc<RestClient>) -> Self
    where
        T: Wrap,
    {
        ModelWrapper { inner, rest_client }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn rest_client(&self) -> &RestClient {
        &self.rest_client
    }
}

impl<T> Deref for ModelWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ModelWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait Wrap: Sized {
    fn wrap(self, rest_client: Arc<RestClient>) -> ModelWrapper<Self> {
        ModelWrapper::new(self, rest_client)
    }
}
