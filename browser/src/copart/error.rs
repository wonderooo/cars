use crate::copart::browser::CorrelationId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum BrowserError {
    #[error("chromium oxide error: {0}")]
    CdpError(String),
    #[error("could not build cdp command from given arguments")]
    CdpCommandBuild(String),
    #[error("argument is not valid utf8: {0}")]
    InvalidUtf8(String),
    #[error("could not marshall/unmarshall given argument: {0}")]
    Json(String),
    #[error("could not send copart browser cmd/response to channel")]
    ChannelSend,
    #[error("intercepted browser request/response which is not handled")]
    UnhandledInterception(String),
    #[error("correlation id not found in url")]
    CorrelationIdNotFound(String),
    #[error("page number not found in query params")]
    PageNumberNotFound,
    #[error("could not decode to base64 from given argument: {0}")]
    Base64Decode(String),
    #[error("could not build valid URL from given argument")]
    InvalidUrl(String),
    #[error("could not parse to int: {0}")]
    ParseInt(String),
}

impl From<std::num::ParseIntError> for BrowserError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value.to_string())
    }
}

impl From<url::ParseError> for BrowserError {
    fn from(value: url::ParseError) -> Self {
        Self::InvalidUrl(value.to_string())
    }
}

impl From<chromiumoxide::error::CdpError> for BrowserError {
    fn from(value: chromiumoxide::error::CdpError) -> Self {
        Self::CdpError(value.to_string())
    }
}

impl From<base64::DecodeError> for BrowserError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64Decode(value.to_string())
    }
}

impl From<serde_json::Error> for BrowserError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

impl From<std::str::Utf8Error> for BrowserError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8(value.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for BrowserError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelSend
    }
}

#[derive(Debug, Error)]
#[error("copart browser error for correlation id `{correlation_id}`: {err}")]
pub struct BrowserErrorWithCorrelation {
    pub err: BrowserError,
    pub correlation_id: CorrelationId,
}

impl BrowserErrorWithCorrelation {
    pub fn new(err: BrowserError, correlation_id: CorrelationId) -> Self {
        Self {
            err,
            correlation_id,
        }
    }
}

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("could not send cmd/response to channel")]
    ChannelSend,
    #[error("worker nodes vecdeque is empty")]
    NodesEmpty,
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for PoolError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelSend
    }
}
