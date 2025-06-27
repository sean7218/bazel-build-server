use std::fs::read;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Result;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerConfig {
    pub name: String,
    pub argv: Vec<String>,
    pub version: String,
    pub bsp_version: String,
    pub languages: Vec<String>,
    pub target: String,
    pub sdk: String,
    pub index_store_path: String,
    pub index_database_path: String,
    pub execution_root: String,
    pub aquery_args: Vec<String>,
    pub extra_includes: Vec<String>,
    pub extra_frameworks: Vec<String>,
}

impl BuildServerConfig {
    pub fn parse(root_uri: &Url) -> Result<BuildServerConfig> {
        let root_path = match root_uri.to_file_path() {
            Ok(v) => v,
            Err(()) => {
                return Err("Invalid path for root_uri.".into());
            }
        };

        let config_path = root_path.join("buildServer.json");
        let bytes = read(config_path)?;
        let config: BuildServerConfig = serde_json::from_slice(&bytes)?;

        Ok(config)
    }
}
