use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use thiserror::Error;
use crate::exact_api::TokenError;

pub type WebResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal server error")]
    Dal(#[from] dal::Error),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Error at upstream partner")]
    Reqwest(#[from] reqwest::Error),
    #[error("Token exchange error: {0}")]
    TokenExchangeError(#[from] TokenError),
    #[error("Authorization error: {0}")]
    AuthClient(#[from] mrauth::Error),
    #[error("Authorization failed")]
    AuthError(#[from] mrauth::actix::AuthError),
    #[error("Not found")]
    NotFound
}

impl ResponseError for Error {

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Reqwest(_) => StatusCode::BAD_GATEWAY,
            Self::TokenExchangeError(e) => match e {
                TokenError::Reqwest(_) => StatusCode::BAD_GATEWAY,
                TokenError::InvalidGrant => StatusCode::BAD_REQUEST,
                TokenError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::AuthClient(e) => match e {
                mrauth::Error::Reqwest(_) => StatusCode::BAD_GATEWAY,
                mrauth::Error::UnknownToken
                | mrauth::Error::MissingScopes => StatusCode::FORBIDDEN,
                mrauth::Error::ProtocolError(_)
                | mrauth::Error::EncodeDecodeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::AuthError(e) => e.status_code(),
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Self::AuthError(e) => e.error_response(),
            _ => HttpResponse::build(self.status_code()).body(self.to_string().into_bytes())
        }
    }

}