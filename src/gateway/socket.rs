use crate::model::gateway::intents;
use crate::model::gateway::{GatewayCommand, GatewayEvent, GatewayEventSeed};
use async_tungstenite::{
    self as ws,
    tokio::ConnectStream,
    tungstenite::{Error, Message as WsMessage},
    WebSocketStream,
};
use futures::prelude::*;
use serde::de::DeserializeSeed;
use std::task::Poll;

const GATEWAY_VERSION: u16 = 8;

/// `GatewaySocket` forwards GatewayEvents from and to the Gateway
///
pub struct GatewaySocket {
    inner: Option<WebSocketStream<ConnectStream>>,
}

impl GatewaySocket {
    pub fn new() -> Self {
        GatewaySocket { inner: None }
    }

    pub fn connected(&self) -> bool {
        self.inner.is_some()
    }

    /// start the connection to the gateway websocket
    pub async fn connect(&mut self, gateway_url: &str) -> Result<(), Error> {
        let (stream, _) = ws::tokio::connect_async(gateway_url).await?;

        self.inner = Some(stream);

        Ok(())
    }

    /// terminate the websocket connection if it exists
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(mut s) = self.inner.take() {
            s.close(None).await?
        }
        Ok(())
    }

    /// close the current connection if it exisits and reconnect
    pub async fn reconnect(&mut self, gateway_url: &str) -> Result<(), Error> {
        self.disconnect().await?;
        self.connect(gateway_url).await
    }

    //    pub async fn send(&mut self, content: String) -> Result<(), Error> {
    //        let stream = self.inner.as_mut().expect("socket not connected");
    //
    //        stream.send(WsMessage::Text(content)).await?;
    //
    //        Ok(())
    //    }
}

impl Stream for GatewaySocket {
    type Item = GatewayEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.inner.is_none() {
            return Poll::Ready(None);
        }

        let stream = self.inner.as_mut().unwrap();

        match stream.next().poll_unpin(cx) {
            Poll::Ready(msg) => {
                let msg = msg.unwrap().unwrap();

                let msg = msg.into_text().unwrap();
                let msg = msg.trim();

                let seed = GatewayEventSeed::from_json_str(&msg);
                let mut deserializer = serde_json::Deserializer::from_str(&msg);

                let event = seed
                    .deserialize(&mut deserializer)
                    .expect(&format!("could not deserialize: {}", msg));

                Poll::Ready(Some(event))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Sink<GatewayCommand> for GatewaySocket {
    type Error = Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let stream = self.inner.as_mut().expect("socket not connected");
        stream.poll_ready_unpin(cx)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: GatewayCommand,
    ) -> Result<(), Self::Error> {
        let stream = self.inner.as_mut().expect("socket not connected");

        stream.start_send_unpin(WsMessage::Text(
            serde_json::to_string(&item).expect("deserialize GatewayCommand"),
        ))
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let stream = self.inner.as_mut().expect("socket not connected");

        stream.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let stream = self.inner.as_mut().expect("socket not connected");

        stream.poll_close_unpin(cx)
    }
}

fn identify_payload(token: &str) -> String {
    format!(
        r#"{{ "op": 2, "d": {{ "token": "{}", "intents": {}, "properties": {{ "$os": "linux", "$browser": "donbot", "$device": "donbot" }} }} }}"#,
        token,
        intents::ALL
    )
}

fn resume_payload(token: &str, session_id: &str, seq: u64) -> String {
    format!(
        r#"{{ "op": 6, "d": {{ "token": "{}", "session_id": "{}", "seq": {} }} }}"#,
        token, session_id, seq
    )
}
