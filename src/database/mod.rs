use crate::config::{Connection as ConnectionConfig, Database as DatabaseConfig};
use crate::database::model::Users;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use thiserror::Error;

mod model;
mod schema;

#[cfg(feature = "mysql")]
const MYSQL_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/mysql");
#[cfg(feature = "postgres")]
const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/postgres");

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    ConnectionError(#[from] diesel::r2d2::PoolError),
    #[error(transparent)]
    QueryError(#[from] diesel::result::Error),
    #[error("{0}")]
    MigrationError(String),
    #[error("{0}")]
    MissingConfiguration(String),
}

pub enum Database {
    #[cfg(feature = "mysql")]
    MySQL(Pool<ConnectionManager<MysqlConnection>>),
    #[cfg(feature = "postgres")]
    Postgres(Pool<ConnectionManager<PgConnection>>),
}

impl Database {
    pub(crate) fn authenticate_user(
        &self,
        usr: &str,
        pwd: &str,
    ) -> Result<Option<Users>, DatabaseError> {
        use schema::users::dsl::*;

        let digest = md5::compute(pwd);
        let pwd = format!("{:x}", digest);

        let select = users.filter(user_id.eq(usr).and(password.eq(pwd)));
        let result = match self {
            #[cfg(feature = "mysql")]
            Database::MySQL(conn) => {
                let mut conn = conn.get()?;
                select.load::<Users>(&mut conn)?
            }
            #[cfg(feature = "postgres")]
            Database::Postgres(conn) => {
                let mut conn = conn.get()?;
                select.load::<Users>(&mut conn)?
            }
        };

        Ok(result.into_iter().next())
    }
}

impl TryFrom<DatabaseConfig> for Database {
    type Error = DatabaseError;

    fn try_from(value: DatabaseConfig) -> Result<Self, Self::Error> {
        let username = value.username();
        let password = value
            .password()
            .map_err(DatabaseError::MissingConfiguration)?;
        let database = value.name();
        match value.connection() {
            #[cfg(feature = "mysql")]
            ConnectionConfig::MySQL(url) => {
                let url = format!("mysql://{username}:{password}@{url}/{database}");
                let manager = ConnectionManager::<MysqlConnection>::new(url);
                let pool = Pool::builder().test_on_check_out(true).build(manager)?;

                // Running migrations
                pool.get()?
                    .run_pending_migrations(MYSQL_MIGRATIONS)
                    .map_err(|error| {
                        Self::Error::MigrationError(format!("Can't run mysql migration : {error}"))
                    })?;

                Ok(Self::MySQL(pool))
            }
            #[cfg(feature = "postgres")]
            ConnectionConfig::Postgres(url) => {
                let url = format!("postgres://{username}:{password}@{url}/{database}");
                let manager = ConnectionManager::<PgConnection>::new(url);
                let pool = Pool::builder().test_on_check_out(true).build(manager)?;

                // Running migrations
                pool.get()?
                    .run_pending_migrations(POSTGRES_MIGRATIONS)
                    .map_err(|error| {
                        Self::Error::MigrationError(format!(
                            "Can't run postgres migration : {error}"
                        ))
                    })?;

                Ok(Self::Postgres(pool))
            }
        }
    }
}
