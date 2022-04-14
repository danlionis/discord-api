use http::{Method, Request};
use serde::Serialize;

use crate::model::id::StickerId;
use crate::model::Attachment;
use crate::model::{id::ChannelId, Embed, MessageReference};

use crate::rest::Route;

/// Params to for the create message endpoint
///
/// <https://discord.com/developers/docs/resources/channel#create-message>
#[derive(Debug, Default, Serialize)]
#[allow(missing_docs)]
pub struct CreateMessageParams {
    pub content: String,
    pub tts: bool,
    pub embeds: Vec<Embed>,
    // allowed_mentions: AllowedMentions, // TODO
    pub message_reference: Option<MessageReference>,
    // components: MessageComponents, // TODO
    pub sticker_ids: Vec<StickerId>,
    // files: file contents // TODO
    // payload_json: String // TODO
    pub attachments: Vec<Attachment>,
    pub flags: u32,
}

impl CreateMessageParams {
    /// Set content
    pub fn content<S>(mut self, content: S) -> Self
    where
        S: Into<String>,
    {
        self.content = content.into();
        self
    }

    /// Set text to speech
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = tts;
        self
    }

    /// Set embeds
    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Add embed
    pub fn embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Set message reference
    pub fn reference(mut self, message_reference: MessageReference) -> Self {
        self.message_reference = Some(message_reference);
        self
    }

    /// Set stickers
    pub fn stickers(mut self, stickers: Vec<StickerId>) -> Self {
        self.sticker_ids = stickers;
        self
    }

    /// Set attachments
    pub fn attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments;
        self
    }
}

/// Generate a send message request
pub fn create_message(
    channel_id: ChannelId,
    params: CreateMessageParams,
) -> Request<CreateMessageParams> {
    let req = Request::builder()
        .uri(Route::ChannelMessages { channel_id })
        .method(Method::POST)
        .body(params)
        .unwrap();

    req
}
