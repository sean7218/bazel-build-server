use std::fs::read;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{log_debug, log_str};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerConfig {
    pub name: String,
    pub argv: Vec<String>,
    pub version: String,
    pub bsp_version: String,
    pub languages: Vec<String>,
}

impl BuildServerConfig {
    pub fn parse(root_uri: &str) -> Option<BuildServerConfig> {
        let root_uri = match Url::parse(root_uri) {
            Ok(v) => v,
            Err(e) => {
                log_debug!(&e);
                return None;
            }
        };

        let root_path = match root_uri.to_file_path() {
            Ok(v) => v,
            Err(()) => {
                log_str!("Invalid path for root_uri.");
                return None;
            }
        };

        let config_path = root_path.join("buildServer.json");

        let bytes = match read(config_path) {
            Ok(v) => v,
            Err(e) => {
                log_debug!(&e);
                return None;
            }
        };

        match serde_json::from_slice(&bytes) {
            Ok(v) => return v,
            Err(e) => {
                log_debug!(&e);
                return None;
            }
        }
    }
}
