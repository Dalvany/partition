use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

// Root config environments
static ENV_LISTEN: &str = "PARTITION_LISTEN";
static ENV_LOG_CONFIG: &str = "PARTITION_LOG_CONFIG";
static ENV_HEADERS: &str = "PARTITION_HEADERS_";

// Library config environments
static ENV_LIBRARY_PATH: &str = "PARTITION_LIBRARY_PATH";
static ENV_LIBRARY_TMP: &str = "PARTITION_LIBRARY_TMP";

// Index config environments
static ENV_INDEXING_PATH: &str = "PARTITION_INDEXING_PATH";

// UI config environments
static ENV_UI_PATH: &str = "PARTITION_UI_PATH";

// Database config environments
static ENV_DATABASE_USERNAME: &str = "PARTITION_DATABASE_USERNAME";
static ENV_DATABASE_PASSWORD: &str = "PARTITION_DATABASE_PASSWORD";
static ENV_DATABASE_NAME: &str = "PARTITION_DATABASE_NAME";
#[cfg(feature = "mysql")]
static ENV_DATABASE_CONNECTION_MYSQL: &str = "PARTITION_DATABASE_CONNECTION_MYSQL";
#[cfg(feature = "postgres")]
static ENV_DATABASE_CONNECTION_POSTGRES: &str = "PARTITION_DATABASE_CONNECTION_POSTGRES";

/// Partition music server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLine {
    /// Path to configuration file
    #[arg(short, long, default_value = "./partition.toml")]
    configuration: String,
}

impl CommandLine {
    pub fn configuration(&self) -> Result<MainConfig> {
        let content =
            std::fs::read_to_string(Path::new(&self.configuration)).with_context(|| {
                format!(
                    "Unable to read configuration file at '{}'.",
                    self.configuration
                )
            })?;

        match toml::from_str(&content) {
            Err(error) => Err(anyhow!("Can't parse configuration file : {error}")),
            Ok(config) => Ok(config),
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct MainConfig {
    listen: Option<String>,
    log_config: String,
    headers: Option<BTreeMap<String, String>>,
    library: Library,
    indexing: Indexing,
    database: Database,
    ui: Option<UI>,
}

impl MainConfig {
    /// Host to bind to. Default to `127.0.0.1:8000`
    pub fn listen(&self) -> String {
        std::env::var(ENV_LISTEN)
            .ok()
            .or_else(|| self.listen.clone())
            .unwrap_or_else(|| String::from("127.0.0.1:8000"))
    }

    /// Headers
    pub(crate) fn headers(&self) -> BTreeMap<String, String> {
        // HTTP headers are case insensitive : https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
        // We lowercase header key so that merge with environment variables works and doesn't duplicate headers.
        let mut headers: BTreeMap<String, String> = self
            .headers
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|(key, value)| (key.to_lowercase(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();

        for (key, value) in std::env::vars() {
            if key.starts_with(ENV_HEADERS) {
                let key = key.strip_prefix(ENV_HEADERS).unwrap_or_default();
                if !key.is_empty() {
                    // HTTP headers are case insensitive : https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
                    // We lowercase header key so that merge works and doesn't duplicate headers.
                    headers.insert(key.to_lowercase(), value);
                }
            }
        }

        headers
    }

    /// log4rs configuration file
    pub fn log_config(&self) -> String {
        std::env::var(ENV_LOG_CONFIG).unwrap_or_else(|_| self.log_config.clone())
    }

    /// Library configuration
    pub fn library(&self) -> Library {
        self.library.clone()
    }

    /// Indexing configuration
    pub fn indexing(&self) -> &Indexing {
        &self.indexing
    }

    /// Database
    pub fn database(&self) -> Database {
        self.database.clone()
    }

    /// UI
    pub fn ui(&self) -> Option<&UI> {
        self.ui.as_ref()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Library {
    path: String,
    tmp: String,
}

impl Library {
    /// Path to library
    pub fn path(&self) -> PathBuf {
        let path = std::env::var(ENV_LIBRARY_PATH).unwrap_or_else(|_| self.path.clone());
        PathBuf::from(path)
    }

    /// Path to temporary folder. Upload files will
    /// go there before being parsed for tag
    pub fn tmp(&self) -> PathBuf {
        let path = std::env::var(ENV_LIBRARY_TMP).unwrap_or_else(|_| self.tmp.clone());
        PathBuf::from(path)
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Indexing {
    path: String,
}

impl Indexing {
    /// Path to index
    pub fn path(&self) -> PathBuf {
        let path = std::env::var(ENV_INDEXING_PATH).unwrap_or_else(|_| self.path.clone());
        PathBuf::from(path)
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct UI {
    path: String,
}

impl UI {
    /// Path to UI files
    pub fn path(&self) -> PathBuf {
        let path = std::env::var(ENV_UI_PATH).unwrap_or_else(|_| self.path.clone());
        PathBuf::from(path)
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Database {
    connection: Connection,
    username: String,
    password: Option<String>,
    name: String,
}

impl Database {
    pub fn username(&self) -> String {
        std::env::var(ENV_DATABASE_USERNAME).unwrap_or_else(|_| self.username.clone())
    }

    pub fn password(&self) -> Result<String, String> {
        std::env::var(ENV_DATABASE_PASSWORD)
            .or_else(|_| self.password.clone().ok_or_else(|| format!("Missing database password. Please fill configuration file or use {ENV_DATABASE_PASSWORD} environment variable")))
    }

    pub fn name(&self) -> String {
        std::env::var(ENV_DATABASE_NAME).unwrap_or_else(|_| self.name.clone())
    }

    pub fn connection(&self) -> Connection {
        match &self.connection {
            #[cfg(feature = "mysql")]
            Connection::MySQL(host) => Connection::MySQL(
                std::env::var(ENV_DATABASE_CONNECTION_MYSQL)
                    .ok()
                    .unwrap_or_else(|| host.clone()),
            ),
            #[cfg(feature = "postgres")]
            Connection::Postgres(host) => Connection::Postgres(
                std::env::var(ENV_DATABASE_CONNECTION_POSTGRES)
                    .ok()
                    .unwrap_or_else(|| host.clone()),
            ),
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    #[cfg(feature = "mysql")]
    MySQL(String),
    #[cfg(feature = "postgres")]
    Postgres(String),
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "mysql")]
            Connection::MySQL(url) => write!(f, "MySQL {url}"),
            #[cfg(feature = "postgres")]
            Connection::Postgres(url) => write!(f, "Postgres {url}"),
        }
    }
}
