use serde::Deserialize;

/// Proto Scheme based on the following
/// https://github.com/bazelbuild/bazel/blob/master/src/main/protobuf/analysis_v2.proto
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub artifacts: Vec<Artifact>,
    pub actions: Vec<Action>,
    pub targets: Vec<Target>,
    pub rule_classes: Vec<RuleClass>,
    pub dep_set_of_files: Vec<DepSetOfFiles>,
    pub path_fragments: Vec<PathFragment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: u32,
    pub path_fragment_id: u32,
    pub is_tree_artifact: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub target_id: u32,
    pub action_key: String,
    pub mnemonic: String,
    pub configuration_id: u32,
    pub arguments: Vec<String>,
    pub environment_variables: Vec<EnvironmentVariable>,
    pub input_dep_set_ids: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: u32,
    pub label: String,
    pub rule_class_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct RuleClass {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PathFragment {
    pub id: u32,
    pub label: String,
    pub parent_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepSetOfFiles {
    pub id: u32,
    pub direct_artifact_ids: Option<Vec<u32>>,
    pub transitive_dep_set_ids: Option<Vec<u32>>,
}
