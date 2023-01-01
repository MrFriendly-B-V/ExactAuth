use mrauth::auth_proto::AuthorizationFailureResponse;
use reqwest::Client;
use reqwest_protobuf::{ProtobufRequestExt, ProtobufResponseExt};

mod error;
pub use error::*;
use proto::GetAccessTokenResponse;
use crate::protobuf_ext::{ProtobufRequestExt, ProtobufResponseExt};

#[derive(Clone)]
pub struct ExactAuthClient {
    base_url: String,
    client: Client,
}

pub struct AccessToken {
    pub token: String,
    pub expires_at: i64,
}

impl From<GetAccessTokenResponse> for AccessToken {
    fn from(x: GetAccessTokenResponse) -> Self {
        Self {
            token: x.token,
            expires_at: x.expires_at
        }
    }
}

impl ExactAuthClient {
    pub fn new(base_url: String, user_agent: &str) -> reqwest::Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .build()?;
        Ok(Self {
            base_url,
            client
        })
    }

    pub fn get_url(&self, path: &str) -> String {
        format!("{}{path}", &self.base_url)
    }

    pub async fn get_exact_access_token(&self, mrauth_bearer: &str) -> Result<AccessToken, Error> {
        let response = self.client
            .get(self.get_url("/api/v1/access-token"))
            .bearer_auth(mrauth_bearer)
            .accept_protobuf()
            .send()
            .await?;

        if response.status() == 403 {
            let payload: AuthorizationFailureResponse = response.protobuf().await?;
            return Err(Error::Auth(payload));
        }

        response.error_for_status_ref()?;

        let payload: GetAccessTokenResponse = response.protobuf().await?;
        Ok(AccessToken {
            token: payload.token,
            expires_at: payload.expires_at,
        })
    }
}