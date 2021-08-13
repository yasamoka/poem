use std::io::{Error as IoError, Result as IoResult};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use hyper::upgrade::Upgraded;

use super::utils::{tungstenite_error_to_io_error, tungstenite_msg_to_message};
use super::Message;
use crate::websocket::utils::msg_to_tungstenite_message;

pub struct WebSocketStream {
    inner: tokio_tungstenite::WebSocketStream<Upgraded>,
}

impl WebSocketStream {
    pub(crate) fn new(inner: tokio_tungstenite::WebSocketStream<Upgraded>) -> Self {
        Self { inner }
    }
}

impl Stream for WebSocketStream {
    type Item = IoResult<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(msg))) => Poll::Ready(Some(Ok(tungstenite_msg_to_message(msg)))),
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(tungstenite_error_to_io_error(err))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Sink<Message> for WebSocketStream {
    type Error = IoError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready_unpin(cx)
            .map_err(tungstenite_error_to_io_error)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.inner
            .start_send_unpin(msg_to_tungstenite_message(item))
            .map_err(tungstenite_error_to_io_error)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_flush_unpin(cx)
            .map_err(tungstenite_error_to_io_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_close_unpin(cx)
            .map_err(tungstenite_error_to_io_error)
    }
}
