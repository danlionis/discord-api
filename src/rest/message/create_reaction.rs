use http::{Method, Request};

use crate::{
    model::id::{ChannelId, MessageId},
    rest::Route,
};

/// Generate a send message request
pub fn create_reaction(channel_id: ChannelId, message_id: MessageId, emoji: String) -> Request<()> {
    let req = Request::builder()
        .uri(Route::OwnReaction {
            channel_id,
            message_id,
            emoji,
        })
        .method(Method::PUT)
        .body(())
        .unwrap();

    req
}
