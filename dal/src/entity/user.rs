use mysql::{params, PooledConn, Row};
use mysql::prelude::Queryable;
use crate::{DalResult, Error, generate_id, Mysql};

#[derive(Clone)]
pub struct User {
    mysql: Mysql,
    pub id: String,
}

pub struct AuthorizationStart {
    pub id: String,
    pub user: User,
    pub timestamp: i64,
    pub caller: String,
    pub exact_scopes: String,
}

pub enum OAuth2Tokentype {
    Access,
    Refresh
}

impl OAuth2Tokentype {
    fn get_token_type_string(&self) -> &'static str {
        match self {
            Self::Access => TOKEN_TYPE_ACCESS,
            Self::Refresh => TOKEN_TYPE_REFRESH,
        }
    }
}

pub struct OAuth2Token {
    pub token: String,
    pub expiry: i64,
    pub token_type: OAuth2Tokentype
}

// Set in the database schema for the oauth2_tokens.token_type
const TOKEN_TYPE_ACCESS: &str = "Access";
const TOKEN_TYPE_REFRESH: &str = "Refresh";

impl User {
    pub fn list_all(mysql: Mysql) -> DalResult<Vec<Self>> {
        let mut conn = mysql.get_conn()?;
        let rows: Vec<Row> = conn.query("SELECT id FROM users")?;
        let users = rows.into_iter()
            .map(|x| User::get_by_id_impl(mysql.clone(), &x.get::<String, &str>("id").unwrap(), &mut conn))
            .collect::<DalResult<Vec<_>>>()?
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        Ok(users)
    }

    pub fn get_by_id(mysql: Mysql, id: &str) -> DalResult<Option<Self>> {
        let mut conn = mysql.get_conn()?;
        Self::get_by_id_impl(mysql, id, &mut conn)
    }

    fn get_by_id_impl(mysql: Mysql, id: &str, conn: &mut PooledConn) -> DalResult<Option<Self>> {
        let _: Row = match conn.exec_first("SELECT 1 FROM users WHERE id = :id", params! {
            "id" => id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        Ok(Some(Self {
            mysql,
            id: id.to_string(),
        }))
    }

    pub fn create(mysql: Mysql, id: &str) -> DalResult<Self> {
        let mut conn = mysql.get_conn()?;
        conn.exec_drop("INSERT INTO users (id) VALUES (:id)", params! {
            "id" => id
        })?;

        Ok(Self {
            mysql,
            id: id.to_string()
        })
    }

    pub fn start_authorization(&self, exact_scopes: &str, caller: &str) -> DalResult<AuthorizationStart> {
        let id = generate_id(32);
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let mut conn = self.mysql.get_conn()?;
        conn.exec_drop("INSERT INTO oauth2_authorization_start (id, user_id, timestamp, caller, scopes) VALUES (:id, :user_id, :timestamp, :caller, :scopes)", params! {
            "id" => &id,
            "user_id" => &self.id,
            "timestamp" => now,
            "caller" => caller,
            "scopes" => exact_scopes
        })?;

        Ok(AuthorizationStart {
            user: self.clone(),
            id,
            exact_scopes: exact_scopes.to_string(),
            timestamp: now,
            caller: caller.to_string(),
        })
    }

    pub fn get_by_authorization_start_id(mysql: Mysql, id: &str) -> DalResult<Option<AuthorizationStart>> {
        let mut conn = mysql.get_conn()?;
        let row: Row = match conn.exec_first("SELECT user_id, timestamp, caller, scopes FROM oauth2_authorization_start WHERE id = :id", params! {
            "id" => id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let user_id: String = row.get("user_id").unwrap();
        let timestamp: i64 = row.get("timestamp").unwrap();
        let caller: String = row.get("caller").unwrap();
        let scopes: String = row.get("scopes").unwrap();

        let user = Self::get_by_id_impl(mysql, &user_id, &mut conn)?
            .ok_or(Error::InvalidState("Missing user for existing oauth2 authorization start".into()))?;

        Ok(Some(AuthorizationStart {
            user,
            id: id.to_string(),
            timestamp,
            exact_scopes: scopes,
            caller
        }))
    }

    pub fn set_access_token(&self, token: &str, expiry: i64) -> DalResult<()> {
        self.set_token(token, expiry, OAuth2Tokentype::Access)
    }

    pub fn set_refresh_token(&self, token: &str, expiry: i64) -> DalResult<()> {
        self.set_token(token, expiry, OAuth2Tokentype::Refresh)
    }

    pub fn get_access_token(&self) -> DalResult<Option<OAuth2Token>> {
        self.get_token(OAuth2Tokentype::Access)
    }

    pub fn get_refresh_token(&self) -> DalResult<Option<OAuth2Token>> {
        self.get_token(OAuth2Tokentype::Refresh)
    }

    fn set_token(&self, token: &str, expiry: i64, token_type: OAuth2Tokentype) -> DalResult<()> {
        let token_type_string = token_type.get_token_type_string();
        let mut conn = self.mysql.get_conn()?;

        let has_token = self.get_token(token_type)?.is_some();
        if has_token {
            conn.exec_drop("UPDATE oauth2_tokens SET token = :token, expiry = :expiry WHERE user_id = :user_id AND token_type = :token_type", params! {
                "token" => token,
                "expiry" => expiry,
                "user_id" => &self.id,
                "token_type" => token_type_string,
            })?;
        } else {
            conn.exec_drop("INSERT INTO oauth2_tokens (user_id, token, token_type, expiry) VALUES (:user_id, :token, :token_type, :expiry)", params! {
                "token" => token,
                "expiry" => expiry,
                "user_id" => &self.id,
                "token_type" => token_type_string,
            })?;
        }

        Ok(())
    }

    fn get_token(&self, token_type: OAuth2Tokentype) -> DalResult<Option<OAuth2Token>> {
        let mut conn = self.mysql.get_conn()?;
        let row: Row = match conn.exec_first("SELECT token,expiry FROM oauth2_tokens WHERE user_id = :user_id AND token_type = :token_type", params! {
            "user_id" => &self.id,
            "token_type" => token_type.get_token_type_string(),
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let token: String = row.get("token").unwrap();
        let expiry: i64 = row.get("expiry").unwrap();

        Ok(Some(OAuth2Token {
            token,
            expiry,
            token_type,
        }))
    }
}