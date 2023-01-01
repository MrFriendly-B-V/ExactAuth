use crate::error::DalResult;
use mysql::{OptsBuilder, Pool};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// The mysql connection.
///
/// `Self` is always considered to be partially equal to another `Self`, irregardless
/// of what database is used.
#[derive(Clone)]
pub struct Mysql(Pool);

impl PartialEq for Mysql {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Debug for Mysql {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mysql {{ ... }}")
    }
}

impl Deref for Mysql {
    type Target = Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Mysql {
    /// Create a new self
    ///
    /// # Errors
    ///
    /// - If the supplied credentials are incorrect
    /// - If the supplied host isn't reachable
    /// - If the supplied database doesn't exist
    /// - If creating the connection fails for any other reason ([See more](Pool::new))
    /// - If applying the migrations fails
    pub fn new(user: &str, password: &str, host: &str, database: &str) -> DalResult<Self> {
        let opts = OptsBuilder::new()
            .user(Some(user))
            .pass(Some(password))
            .ip_or_hostname(Some(host))
            .db_name(Some(database));
        let pool = Pool::new(opts)?;

        let mut conn = pool.get_conn()?;
        migrations::migrations::runner()
            .set_migration_table_name("__mrauth_migrations")
            .run(&mut conn)?;

        Ok(Self(pool))
    }
}

/// Embedded migrations
mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}
