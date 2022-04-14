use crate::{model::id::UserId, rest::Route};
use http::{Method, Request};
use serde::Serialize;

/// Params for creating a DM channel
#[derive(Debug, Serialize)]
pub struct CreateDmParams {
    recipient_id: UserId,
}

impl CreateDmParams {
    /// Create new CreateDmParams
    pub fn new(recipient_id: UserId) -> Self {
        CreateDmParams { recipient_id }
    }
}

/// Generate a send message request
pub fn create_dm(params: CreateDmParams) -> Request<CreateDmParams> {
    let req = Request::builder()
        .uri(Route::CurrentUserChannels)
        .method(Method::POST)
        .body(params)
        .unwrap();

    req
}
