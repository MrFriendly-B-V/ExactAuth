use actix_multiresponse::Payload;
use mrauth::actix::BearerHeader;
use dal::User;
use crate::{AuthData, MysqlData};
use crate::error::{Error, WebResult};
use proto::GetAccessTokenResponse;

pub const SCOPE: &str = "nl.mrfriendly.exact";

pub async fn access_token(mysql: MysqlData, auth: AuthData, bearer: BearerHeader) -> WebResult<Payload<GetAccessTokenResponse>> {
    let auth_user = mrauth::User::get_user(&auth, &bearer, SCOPE).await?;
    let user = User::get_by_id(mysql.as_ref().clone(), &auth_user.id)?
        .ok_or(Error::NotFound)?;

    let access_token = user.get_access_token()?
        .ok_or(Error::NotFound)?;

    Ok(Payload(GetAccessTokenResponse {
        token: access_token.token,
        expires_at: access_token.expiry
    }))
}