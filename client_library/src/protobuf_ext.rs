use async_trait::async_trait;
use prost::{EncodeError, Message};
use reqwest::{RequestBuilder, Response};
use thiserror::Error;

pub trait ProtobufRequestExt where Self: Sized {
    fn accept_protobuf(self) -> Self;
    fn protobuf<T: Message + Default>(self, value: T) -> Result<Self, EncodeError>;
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Failed to extract body bytes: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Failed to decode protobuf: {0:?}")]
    ProstDecode(prost::DecodeError)
}

impl From<prost::DecodeError> for DecodeError {
    fn from(x: prost::DecodeError) -> Self {
        Self::ProstDecode(x)
    }
}


#[async_trait]
pub trait ProtobufResponseExt {
    async fn protobuf<T: Message + Default>(self) -> Result<T, DecodeError>;
}

impl ProtobufRequestExt for RequestBuilder {
    fn accept_protobuf(self) -> Self {
        self.header("Accept", "application/protobuf")
    }

    fn protobuf<T: Message + Default>(self, value: T) -> Result<Self, EncodeError> {
        let mut buf = Vec::new();
        value.encode(&mut buf)?;
        let this = self.header("Content-Type", "application/protobuf");
        Ok(this.body(buf))
    }
}

#[async_trait]
impl ProtobufResponseExt for Response {
    async fn protobuf<T: Message + Default>(self) -> Result<T, DecodeError> {
        let body = self.bytes().await?;
        let decoded = T::decode(body)?;
        Ok(decoded)
    }
}