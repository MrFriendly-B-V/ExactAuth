use mrauth::auth_proto::AuthorizationFailureResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Failed to encode protobuf: {0:?}")]
    ProstEncode(prost::EncodeError),
    #[error("Failed to decode protobuf: {0:?}")]
    ProstDecode(prost::DecodeError),
    #[error("Authentication failure: {0:?}")]
    Auth(AuthorizationFailureResponse)
}

impl From<crate::protobuf_ext::DecodeError> for Error {
    fn from(x:crate::protobuf_ext:: DecodeError) -> Self {
        match x {
            crate::protobuf_ext::DecodeError::Reqwest(e) => Self::Reqwest(e),
            crate::protobuf_ext::DecodeError::ProstDecode(e) => Self::ProstDecode(e),
        }
    }
}

impl From<prost::EncodeError> for Error {
    fn from(x: prost::EncodeError) -> Self {
        Self::ProstEncode(x)
    }
}