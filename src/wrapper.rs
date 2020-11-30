use crate::rest::RestClient;
use std::ops::{Deref, DerefMut};

/// Utility to wrap a Model with access to a [`RestClient`]
///
/// by itself this type does not have any methods associated with it
#[derive(Debug)]
pub struct ModelWrapper<T> {
    inner: T,
    rest_client: RestClient,
}

impl<T> ModelWrapper<T> {
    pub fn new(inner: T, rest_client: RestClient) -> Self {
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

#[macro_export]
macro_rules! wrap_model {
    ($v:vis $name:ident, $model:ident) => {
        /// A Wrapper that adds additional methods to the [`$model`] type
        $v type $name = $crate::wrapper::ModelWrapper<$model>;
    };
}

struct Ts {
    yeet: u64,
}
