use chik_protocol::Message;
use chik_traits::chik_error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<R> {
    #[error("{0:?}")]
    Chik(#[from] chik_error::Error),

    #[error("{0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("{0:?}")]
    InvalidResponse(Message),

    #[error("missing response")]
    MissingResponse,

    #[error("rejection")]
    Rejection(R),
}
