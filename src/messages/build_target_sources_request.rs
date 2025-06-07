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
    pub roots: Option<Vec<Url>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceItem {
    pub uri: Url,
    pub kind: u8, // 1 = file and 2 = directory
    pub generated: bool,
    pub data_kind: String, // should always be sourceKit
    pub data: SourceKitSourceItemData,
}

impl SourceItem {
    pub fn from_url(url: &Url) -> Self {
        SourceItem {
            uri: url.clone(),
            kind: 1,
            generated: false,
            data_kind: String::from("sourceKit"),
            data: SourceKitSourceItemData {
                language: None,
                kind: Some(String::from("source")),
                output_path: None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceKitSourceItemData {
    /// The language of the source file. If `nil`, the language is inferred from the file extension.
    pub language: Option<String>,

    /// The kind of source file that this source item represents. If omitted, the item is assumed to be a normal source file,
    /// ie. omitting this key is equivalent to specifying it as `source`.
    pub kind: Option<String>,

    /// The output path that is used during indexing for this file, ie. the `-index-unit-output-path`, if it is specified
    /// in the compiler arguments or the file that is passed as `-o`, if `-index-unit-output-path` is not specified.
    ///
    /// This allows SourceKit-LSP to remove index entries for source files that are removed from a target but remain
    /// present on disk.
    ///
    /// The server communicates during the initialize handshake whether it populates this property by setting
    /// `outputPathsProvider: true` in `SourceKitInitializeBuildResponseData`.
    pub output_path: Option<String>,
}
