use crate::error::{CloseCode, Error};
use crate::model::gateway::{GatewayCommand, GatewayEvent, GatewayEventSeed};
use futures::prelude::*;
use serde::de::DeserializeSeed;
use std::task::Poll;
use tokio::net::TcpStream;
use tokio_tungstenite::{self as ws, tungstenite, MaybeTlsStream as AutoStream, WebSocketStream};
use tungstenite::{
    protocol::frame::coding::CloseCode as WsCloseCode, protocol::CloseFrame, Error as WsError,
    Message as WsMessage,
};

const GATEWAY_VERSION: u16 = 8;

/// `GatewaySocket` forwards GatewayEvents from and to the Gateway
///
pub struct GatewaySocket {
    inner: Option<WebSocketStream<AutoStream<TcpStream>>>,
}

impl GatewaySocket {
    pub fn new() -> Self {
        GatewaySocket { inner: None }
    }

    pub fn connected(&self) -> bool {
        self.inner.is_some()
    }

    /// start the connection to the gateway websocket
    pub async fn connect(&mut self, gateway_url: &str) -> Result<(), WsError> {
        let (stream, _) = ws::connect_async(gateway_url).await?;
        log::debug!("websocket connection established");
        self.inner = Some(stream);
        Ok(())
    }

    /// Gracefully close the gateway connection
    ///
    /// This method sends an appropriate CloseCode so that the gateway knows we want to close the
    /// session. The gateway session will be invalidated
    pub async fn close(&mut self) -> Result<(), WsError> {
        if let Some(mut s) = self.inner.take() {
            let close_frame = CloseFrame {
                code: WsCloseCode::Normal,
                reason: "".into(),
            };
            s.close(Some(close_frame)).await?
        }
        log::debug!("websocket connection closed");
        Ok(())
    }

    /// close the current connection if it exisits and reconnect keeping sessions active
    pub async fn reconnect(&mut self, gateway_url: &str) -> Result<(), WsError> {
        if let Some(mut s) = self.inner.take() {
            s.close(None).await?
        }
        self.connect(gateway_url).await
    }
}

impl Stream for GatewaySocket {
    type Item = Result<GatewayEvent, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.inner.is_none() {
            return Poll::Ready(None);
        }

        let stream = self.inner.as_mut().unwrap();

        match stream.next().poll_unpin(cx) {
            Poll::Ready(Some(Ok(WsMessage::Text(msg)))) => {
                let event = {
                    let seed = GatewayEventSeed::from_json_str(&msg);
                    let mut deserializer = serde_json::Deserializer::from_str(&msg);
                    seed.deserialize(&mut deserializer)
                        .expect(&format!("could not deserialize: {}", msg))
                };

                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(Some(Ok(WsMessage::Close(frame)))) => {
                let code = frame
                    .map(|close| CloseCode::from(close.code))
                    .unwrap_or_else(|| CloseCode::UnknownError);

                Poll::Ready(Some(Err(Error::GatewayClosed(code))))
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(Some(other)) => {
                panic!("received unexpected packet {:?}", other)
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Sink<GatewayCommand> for GatewaySocket {
    type Error = WsError;

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
