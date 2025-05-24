use serde::{Deserialize, Serialize};
use url::Url;

use crate::support_types::build_target::BuildTargetIdentifier;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTargetSourcesRequest {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTargetSourcesResponse {
    pub items: Vec<SourcesItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourcesItem {
    pub target: BuildTargetIdentifier,
    pub sources: Vec<SourceItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceItem {
    pub uri: Url,
    pub kind: u8,
    pub generated: bool,
}

impl SourceItem {
    pub fn from_url(url: &Url) -> Self {
        SourceItem {
            uri: url.clone(),
            kind: SourceItemKind::File.to_u8(),
            generated: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceItemKind {
    File = 1,
    Directory = 2,
}

impl SourceItemKind {
    pub fn to_u8(&self) -> u8 {
        match self {
           Self::File => 1,
           Self::Directory => 2 
        }
    }
}