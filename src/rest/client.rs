use http::Request;
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};

use crate::model::{id::ChannelId, Message};

use super::{gateway::GetGatewayBot, message::CreateMessageParams};

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

    async fn post<T, R>(&self, req: Request<T>) -> reqwest::Result<R>
    where
        T: Serialize + Sized,
        R: DeserializeOwned,
    {
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
