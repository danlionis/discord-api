//! Basic REST client for the discord api
//!
//! This client uses [reqwest] in async mode under the hood and thus requires the [tokio] runtime

use std::fmt::Debug;

use http::{Method, Request};
use serde::{de::DeserializeOwned, Serialize};

use crate::model::{
    id::{ChannelId, MessageId},
    Channel, Message, User,
};

use super::{gateway::GetGatewayBot, message::CreateMessageParams, CreateDmParams};

/// Discord rest client
#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    token: String,
}

impl Client {
    /// create a new rest client
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        let token = format!("Bot {}", token);

        Client { client, token }
    }

    /// return bot connection information
    pub async fn get_gateway_bot(&self) -> Result<GetGatewayBot, reqwest::Error> {
        let req = crate::rest::gateway::get_gateway_bot();
        self.request(req).await
    }

    /// Send a message to a text channel
    pub async fn create_message(
        &self,
        channel_id: ChannelId,
        message_params: CreateMessageParams,
    ) -> Result<Message, reqwest::Error> {
        let req = crate::rest::message::create_message(channel_id, message_params);
        self.request(req).await
    }

    /// Create a dm channel with a recipient
    pub async fn create_dm(&self, dm_params: CreateDmParams) -> Result<Channel, reqwest::Error> {
        let req = crate::rest::create_dm(dm_params);
        self.request(req).await
    }

    /// Get the current user
    pub async fn get_current_user(&self) -> reqwest::Result<User> {
        let req = crate::rest::get_current_user();
        self.request(req).await
    }

    /// Get the current user
    pub async fn create_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: String,
    ) -> reqwest::Result<User> {
        let req = crate::rest::create_reaction(channel_id, message_id, emoji);
        self.request(req).await
    }

    async fn request<T: 'static, R>(&self, req: Request<T>) -> reqwest::Result<R>
    where
        T: Serialize + Sized + Debug,
        R: DeserializeOwned,
    {
        log::debug!("req= {:?}", req);
        let (part, body) = req.into_parts();
        let mut req = self
            .client
            .request(part.method.clone(), part.uri.to_string())
            .header(reqwest::header::AUTHORIZATION, &self.token);

        if part.method != Method::GET {
            req = req.json(&body);
        }

        req.send().await?.json::<R>().await
    }
}
