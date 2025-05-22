use std::{error::Error, fmt::Display, fs::read};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::log_debug;


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerConfig {
    pub name: String,
    pub argv: Vec<String>,
    pub version: String,
    pub bsp_version: String,
    pub languages: Vec<String>
}

impl BuildServerConfig {
    pub fn parse(root_uri: &str) -> Result<BuildServerConfig, ConfigError> {
        let root_uri = match Url::parse(root_uri) {
            Ok(v) => v,
            Err(_) => return Err(ConfigError::InvalidUri)
        };

        let root_path = match root_uri.to_file_path() {
            Ok(v) => v,
            Err(()) => return Err(ConfigError::InvalidUri)
        };

        let config_path = root_path.join("buildServer.json");

        let bytes = match read(config_path) {
            Ok(v) => v,
            Err(_) => return Err(ConfigError::ConfigNotFound)
        };

        match serde_json::from_slice(&bytes) {
            Ok(v) => return v,
            Err(e) => {
                log_debug!(&e);
                return Err(ConfigError::ConfigNotParsed)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ConfigError {
    InvalidUri,
    ConfigNotFound,
    ConfigNotParsed,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUri => {
                return write!(f, "ConfigError: Invalid Uri.")
            }
            Self::ConfigNotFound => {
                return write!(f, "ConfigError: Config not found.")
            }
            Self::ConfigNotParsed => {
                write!(f, "ConfigError: Config can't be parsed.")
            }
        }
    }
}

impl Error for ConfigError {}

impl From<ConfigError> for std::io::Error {
    fn from(value: ConfigError) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            value.to_string()
        )
    }
}

