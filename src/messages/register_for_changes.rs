use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterForChanges {
    pub uri: Url,
    pub action: String, // either register or unregister
}

#[allow(dead_code)]
impl RegisterForChanges {
    const METHOD: &'static str = "textDocument/registerForChanges";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisterAction {
    Register,
    Unregister,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOptionsChangedNotification {
    pub updated_options: Options,
    pub uri: Url,
}

impl FileOptionsChangedNotification {
    pub const METHOD: &'static str = "build/sourceKitOptionsChanged";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub options: Vec<String>,
    pub working_directory: Option<String>,
}
