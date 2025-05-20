use serde::Deserialize;

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
    pub id: u8,
    pub path_fragment_id: u8,
    pub is_tree_artifact: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub target_id: u8,
    pub action_key: String,
    pub mnemonic: String,
    pub configuration_id: u8,
    pub arguments: Vec<String>,
    pub environment_variables: Vec<EnvironmentVariable>,
    pub input_dep_set_ids: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: u8,
    pub label: String,
    pub rule_class_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct RuleClass {
    pub id: u8,
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
    pub id: u8,
    pub label: String,
    pub parent_id: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepSetOfFiles {
    pub id: u8,
    pub direct_artifact_ids: Option<Vec<u8>>,
    pub transitive_dep_set_ids: Option<Vec<u8>>,
}
