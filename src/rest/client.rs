//! Basic REST client for the discord api
//!
//! This client uses [reqwest] in async mode under the hood and thus requires the [tokio] runtime

use std::fmt::Debug;

use http::Request;
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};

use crate::model::{id::ChannelId, Channel, Guild, Message, User};

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
        self.get(req).await
    }

    /// Send a message to a text channel
    pub async fn create_message(
        &self,
        channel_id: ChannelId,
        message_params: CreateMessageParams,
    ) -> Result<Message, reqwest::Error> {
        let req = crate::rest::message::create_message(channel_id, message_params);
        self.post(req).await
    }

    /// Create a dm channel with a recipient
    pub async fn create_dm(&self, dm_params: CreateDmParams) -> Result<Channel, reqwest::Error> {
        let req = crate::rest::create_dm(dm_params);
        self.post(req).await
    }

    /// Get the current user
    pub async fn get_current_user(&self) -> reqwest::Result<User> {
        let req = crate::rest::get_current_user();
        self.get(req).await
    }

    // /// Get current user guilds
    // pub async fn get_current_user_guilds(&self) -> reqwest::Result<Vec<Guild>> {
    //     let req = crate::rest::get_current_user_guilds();
    //     self.get(req).await
    // }

    async fn post<T, R>(&self, req: Request<T>) -> reqwest::Result<R>
    where
        T: Serialize + Sized + Debug,
        R: DeserializeOwned,
    {
        log::debug!("req= {:?}", req);
        let (part, body) = req.into_parts();
        let uri = Url::parse(&part.uri.to_string()).unwrap();
        self.client
            .request(part.method, uri)
            .header(reqwest::header::AUTHORIZATION, &self.token)
            .json(&body)
            .send()
            .await?
            .json::<R>()
            .await
    }

    async fn get<R>(&self, req: Request<()>) -> reqwest::Result<R>
    where
        R: DeserializeOwned,
    {
        log::debug!("req= {:?}", req);
        let (part, _) = req.into_parts();
        let uri = Url::parse(&part.uri.to_string()).unwrap();
        self.client
            .request(part.method, uri)
            .header(reqwest::header::AUTHORIZATION, &self.token)
            .send()
            .await?
            .json::<R>()
            .await
    }
}
