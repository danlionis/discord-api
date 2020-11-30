use crate::model::gateway::intents;
use crate::model::gateway::{DispatchEvent, GatewayEvent, GatewayEventSeed, Ready};
use async_tungstenite::{
    self as ws,
    tokio::ConnectStream,
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};
use futures::prelude::*;
use serde::de::DeserializeSeed;
use std::task::Poll;

const GATEWAY_VERSION: u16 = 8;

/// `GatewaySocket` forwards
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

    pub async fn reconnect(
        &mut self,
        gateway_url: &str,
        token: &str,
        session_id: &str,
        seq: u64,
    ) -> Result<u64, WsError> {
        if let Some(mut s) = self.inner.take() {
            s.close(None).await?;
        }

        let (stream, _) = ws::tokio::connect_async(gateway_url).await?;
        self.inner = Some(stream);

        let hello = {
            match self.next().await.unwrap() {
                GatewayEvent::Hello(h) => h,
                _ => unreachable!("hello should be first packet to recieve"),
            }
        };
        self.send(resume_payload(token, session_id, seq)).await?;

        Ok(hello.heartbeat_interval)
    }

    pub async fn connect(
        &mut self,
        gateway_url: &str,
        token: &str,
    ) -> Result<(u64, Ready), WsError> {
        let (stream, _) = ws::tokio::connect_async(gateway_url).await?;

        self.inner = Some(stream);

        let hello = {
            match self.next().await.unwrap() {
                GatewayEvent::Hello(h) => h,
                _ => unreachable!("hello should be first packet to recieve"),
            }
        };

        self.send(identify_payload(token)).await?;
        let ready = {
            match self.next().await.unwrap() {
                GatewayEvent::Dispatch(_, DispatchEvent::Ready(ready)) => ready,
                GatewayEvent::InvalidSession(_reconnectable) => {
                    dbg!(_reconnectable);
                    todo!()
                }
                _ => unreachable!(),
            }
        };

        Ok((hello.heartbeat_interval, ready))
    }

    pub async fn send(&mut self, content: String) -> Result<(), WsError> {
        let stream = self.inner.as_mut().expect("socket not connected");

        stream.send(WsMessage::Text(content)).await?;

        Ok(())
    }
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
