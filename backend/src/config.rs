use clap::{Parser, ValueEnum};
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

use crate::DbPool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Prod,
    Dev,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum DatabaseLocation {
    File(PathBuf),
    Memory,
}

impl FromStr for DatabaseLocation {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ":memory:" | "memory" => Self::Memory,
            path => Self::File(PathBuf::from(path)),
        })
    }
}

impl fmt::Display for DatabaseLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => f.write_str(":memory:"),
            Self::File(path) => f.write_str(&path.to_string_lossy()),
        }
    }
}

impl From<DatabaseLocation> for String {
    fn from(location: DatabaseLocation) -> Self {
        location.to_string()
    }
}

impl TryFrom<String> for DatabaseLocation {
    type Error = Infallible;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl DatabaseLocation {
    pub fn connect(&self) -> DbPool {
        let url = self.to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(url);
        let mut builder = r2d2::Pool::builder();
        if matches!(self, Self::Memory) {
            builder = builder
                .max_size(1)
                .idle_timeout(None)
                .max_lifetime(None);
        }
        builder
            .build(manager)
            .expect("failed to create db pool")
    }
}

#[derive(Parser, Debug, Serialize)]
#[command(name = "finapp-backend")]
struct Args {
    #[arg(long, default_value = "finapp.yaml")]
    #[serde(skip)]
    config: PathBuf,

    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<Mode>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    database: Option<DatabaseLocation>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    dev_token: Option<String>,
}

pub const DEFAULT_DEV_TOKEN: &str = "dev-secret";
pub const DEFAULT_PROD_DATABASE: &str = "finapp.db";

#[derive(Deserialize)]
struct RawConfig {
    mode: Mode,
    #[serde(default)]
    database: Option<DatabaseLocation>,
    port: u16,
    #[serde(default)]
    dev_token: Option<String>,
}

#[derive(Serialize)]
struct BaseDefaults {
    mode: Mode,
    port: u16,
}

impl Default for BaseDefaults {
    fn default() -> Self {
        Self {
            mode: Mode::Prod,
            port: 3000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    pub mode: Mode,
    pub database: DatabaseLocation,
    pub port: u16,
    pub dev_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Prod,
            database: DatabaseLocation::File(PathBuf::from(DEFAULT_PROD_DATABASE)),
            port: 3000,
            dev_token: None,
        }
    }
}

impl TryFrom<RawConfig> for Config {
    type Error = String;
    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        let database = raw.database.unwrap_or_else(|| match raw.mode {
            Mode::Dev => DatabaseLocation::Memory,
            Mode::Prod => DatabaseLocation::File(PathBuf::from(DEFAULT_PROD_DATABASE)),
        });

        let dev_token = match (raw.mode, raw.dev_token) {
            (Mode::Dev, None) => Some(DEFAULT_DEV_TOKEN.to_string()),
            (_, token) => token,
        };

        match (raw.mode, &database) {
            (Mode::Prod, DatabaseLocation::Memory) => {
                Err("memory database forbidden in prod mode".into())
            }
            _ => Ok(Self {
                mode: raw.mode,
                database,
                port: raw.port,
                dev_token,
            }),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, figment::Error> {
        let args = Args::parse();
        Self::from_sources(&args)
    }

    fn from_sources(args: &Args) -> Result<Self, figment::Error> {
        Figment::from(Serialized::defaults(BaseDefaults::default()))
            .merge(Yaml::file(&args.config))
            .merge(Env::prefixed("FINAPP_"))
            .merge(Serialized::defaults(args))
            .extract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_location_parses_memory_aliases() {
        assert_eq!(
            ":memory:".parse::<DatabaseLocation>().unwrap(),
            DatabaseLocation::Memory
        );
        assert_eq!(
            "memory".parse::<DatabaseLocation>().unwrap(),
            DatabaseLocation::Memory
        );
    }

    #[test]
    fn database_location_parses_file_path() {
        assert_eq!(
            "finapp.db".parse::<DatabaseLocation>().unwrap(),
            DatabaseLocation::File(PathBuf::from("finapp.db"))
        );
    }

    #[test]
    fn database_location_round_trip_via_display() {
        assert_eq!(DatabaseLocation::Memory.to_string(), ":memory:");
        assert_eq!(
            DatabaseLocation::File(PathBuf::from("foo.db")).to_string(),
            "foo.db"
        );
    }

    #[test]
    fn config_default_is_prod_file_finapp_db() {
        let config = Config::default();
        assert_eq!(config.mode, Mode::Prod);
        assert_eq!(
            config.database,
            DatabaseLocation::File(PathBuf::from("finapp.db"))
        );
        assert_eq!(config.port, 3000);
        assert_eq!(config.dev_token, None);
    }

    #[test]
    fn validation_rejects_prod_with_memory_database() {
        let raw = RawConfig {
            mode: Mode::Prod,
            database: Some(DatabaseLocation::Memory),
            port: 3000,
            dev_token: None,
        };
        let result = Config::try_from(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("memory database"));
    }

    #[test]
    fn dev_mode_defaults_database_to_memory() {
        let raw = RawConfig {
            mode: Mode::Dev,
            database: None,
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(config.database, DatabaseLocation::Memory);
    }

    #[test]
    fn dev_mode_defaults_dev_token() {
        let raw = RawConfig {
            mode: Mode::Dev,
            database: None,
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(config.dev_token.as_deref(), Some(DEFAULT_DEV_TOKEN));
    }

    #[test]
    fn dev_mode_explicit_database_overrides_default() {
        let raw = RawConfig {
            mode: Mode::Dev,
            database: Some(DatabaseLocation::File(PathBuf::from("custom.db"))),
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(
            config.database,
            DatabaseLocation::File(PathBuf::from("custom.db"))
        );
    }

    #[test]
    fn dev_mode_explicit_token_overrides_default() {
        let raw = RawConfig {
            mode: Mode::Dev,
            database: None,
            port: 3000,
            dev_token: Some("custom-token".into()),
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(config.dev_token.as_deref(), Some("custom-token"));
    }

    #[test]
    fn prod_mode_defaults_database_to_finapp_db() {
        let raw = RawConfig {
            mode: Mode::Prod,
            database: None,
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(
            config.database,
            DatabaseLocation::File(PathBuf::from(DEFAULT_PROD_DATABASE))
        );
    }

    #[test]
    fn prod_mode_does_not_default_dev_token() {
        let raw = RawConfig {
            mode: Mode::Prod,
            database: None,
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(config.dev_token, None);
    }

    #[test]
    fn validation_accepts_prod_with_file_database() {
        let raw = RawConfig {
            mode: Mode::Prod,
            database: Some(DatabaseLocation::File(PathBuf::from("finapp.db"))),
            port: 3000,
            dev_token: None,
        };
        let config = Config::try_from(raw).unwrap();
        assert_eq!(config.mode, Mode::Prod);
    }
}
