//! RestClient connections to the Discord API
// TODO: remove hyper, only create the requests
// let the appliction handle sending

mod gateway;
mod message;

pub use gateway::*;
pub use message::*;

mod routes;
pub use routes::Route;

#[cfg(feature = "rest")]
mod client;
#[cfg(feature = "rest")]
pub use client::*;

// /// test
// pub async fn create_message2(
//     &self,
//     channel_id: ChannelId,
//     content: &str,
//     reference: Option<MessageReference>,
// ) -> Result<Message, Error> {
//     let params = CreateMessageParams {
//         content: content.to_string(),
//         tts: false,
//         embeds: Vec::new(),
//         message_reference: reference,
//         sticker_ids: Vec::new(),
//         attachments: Vec::new(),
//         flags: 0,
//     };
//     let (part, body) =
//         crate::rest::message::create_message(&self.inner.token, channel_id, params)
//             .into_parts();
//     let client = reqwest::Client::new();

//     let uri = Url::parse(&part.uri.to_string()).unwrap();

//     let res = client
//         .request(part.method, uri)
//         .headers(part.headers)
//         .version(part.version)
//         .json(&body)
//         .send()
//         .await
//         .unwrap();
//     let msg = res.json::<Message>().await.unwrap();

//     Ok(msg)
// }
