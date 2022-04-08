//! RestClient connections to the Discord API

mod ratelimit;
mod routes;

use crate::{
    error::Error,
    model::{
        id::{ChannelId, MessageId, UserId},
        Channel, Message, MessageReference,
    },
    util::RestWrapper,
};
use std::sync::Arc;

/// Discord Rest API Client
pub struct Rest {
    inner: Arc<Inner>,
}

impl Clone for Rest {
    fn clone(&self) -> Self {
        Rest {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl std::fmt::Debug for Rest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestClient").finish()
    }
}

impl Rest {
    /// Create a new ApiClient
    pub fn new(token: &str) -> Self {
        Rest {
            inner: Arc::new(Inner::new(token)),
        }
    }

    /// Wrap a value with this ApiClient
    pub fn wrap<T>(&self, inner: T) -> RestWrapper<T> {
        RestWrapper::new(inner, self.clone())
    }

    // pub async fn get_guilds(&self) {
    // let res = self
    //     .client
    //     .get(routes::guilds().parse().unwrap())
    //     .await
    //     .unwrap();
    // // println!("{}", res.text().await.unwrap());
    // let guilds: Vec<PartialGuild> = res.json().await.unwrap();
    // dbg!(guilds);
    // unimplemented!()
    // }

    // pub async fn get_guild_by_id(&self, _: u64) {
    //     // let res = self.get(routes::guild(id).parse().unwrap()).await;
    //     // println!("{}", res.unwrap().await.unwrap());
    //     unimplemented!()
    // }

    // pub async fn get_guild_channels(&self, _: u64) {
    //     let res = self.client.get(&routes::guild_channels(id)).send().await;
    //     println!("{}", res.unwrap().text().await.unwrap());
    //     unimplemented!()
    // }

    /// Trigger the Typing Indicator for a Channel
    pub async fn trigger_typing_indicator(&self, channel_id: ChannelId) {
        self.inner
            .post(
                routes::trigger_typing_indicator(channel_id.into())
                    .parse()
                    .unwrap(),
                serde_json::Value::Null,
            )
            .await
            .unwrap();
    }

    /// Sends a string message into a TextChannel
    pub async fn create_message(
        &self,
        channel_id: ChannelId,
        content: &str,
        reference: Option<MessageReference>,
    ) -> Result<Message, Error> {
        let body = serde_json::json!({ "content": content, "message_reference": reference });

        let res = self
            .inner
            .post(
                routes::channel_messages(channel_id.into()).parse().unwrap(),
                body,
            )
            .await?;

        let bytes = hyper::body::to_bytes(res).await.unwrap();

        serde_json::from_slice::<Message>(&bytes).map_err(Error::from)
    }

    /// Create a DM Channel for a given User
    pub async fn create_dm(&self, recipient_id: UserId) -> Result<Channel, Error> {
        let body = serde_json::json!({ "recipient_id": recipient_id });

        let res = self
            .inner
            .post(routes::user_dm().parse().unwrap(), body)
            .await?;
        let bytes = hyper::body::to_bytes(res).await.unwrap();
        serde_json::from_slice(&bytes).map_err(Error::from)
    }

    /// Delete a Message
    pub async fn delete_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        self.inner
            .delete(
                routes::text_message(channel_id.into(), message_id.into())
                    .parse()
                    .unwrap(),
            )
            .await?;
        Ok(())
    }

    /// Get the Gateway Url
    pub async fn get_gateway(&self) -> Result<String, Error> {
        let url = routes::gateway().parse().unwrap();

        let res = self.inner.get(url).await?;

        let buf = hyper::body::to_bytes(res).await?;

        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        let gateway_url = v.as_object().unwrap().get("url").unwrap().as_str().unwrap();

        Ok(gateway_url.to_owned())
    }
}

struct Inner {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    token: String,
}

impl Inner {
    fn new(token: &str) -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        Inner {
            client,
            token: format!("Bot {}", token),
        }
    }

    fn get_req_builder(
        &self,
        uri: hyper::Uri,
        method: hyper::Method,
    ) -> hyper::http::request::Builder {
        hyper::Request::builder()
            .method(method)
            .uri(uri)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
    }

    async fn get(&self, uri: hyper::Uri) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        let builder = self.get_req_builder(uri, hyper::Method::GET);
        let req = builder.body(hyper::Body::empty()).unwrap();

        let res = self.client.request(req).await?;

        Ok(res)
    }

    async fn post(
        &self,
        uri: hyper::Uri,
        body: serde_json::Value,
    ) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        let builder = self.get_req_builder(uri, hyper::Method::POST);
        let req = builder
            .body(hyper::Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let res = self.client.request(req).await?;
        dbg!(&res);

        Ok(res)
    }

    async fn patch(
        &self,
        uri: hyper::Uri,
        body: serde_json::Value,
    ) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        let builder = self.get_req_builder(uri, hyper::Method::PATCH);
        let req = builder
            .body(hyper::Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let res = self.client.request(req).await?;

        Ok(res)
    }

    async fn delete(&self, uri: hyper::Uri) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        let builder = self.get_req_builder(uri, hyper::Method::DELETE);
        let req = builder.body(hyper::Body::empty()).unwrap();
        let res = self.client.request(req).await?;
        Ok(res)
    }
}
