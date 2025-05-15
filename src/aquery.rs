#![allow(dead_code)]
use serde::Deserialize;
use serde_json::from_slice;
use std::{
    collections::HashMap,
    io::{Read, Stderr},
    process::{Command, Stdio},
};

#[allow(dead_code)]
pub fn aquery() {
    let output = Command::new("bazel")
        .args(&[
            "aquery",
            "mnemonic(\"SwiftCompile\", deps(//Sources/Components))",
            "--output=jsonproto",
        ])
        .current_dir("/Users/sean7218/bazel/buildserver/example/")
        .output()
        .expect("Failed to start aquery process");

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    let query_result: QueryResult = from_slice(&output.stdout).expect("failed to parse output");

    // convert artifacts, depSetOfFiles, and pathFragments into hashmap
    // to reduce time complexity
    let mut artifacts = HashMap::new();
    for artifact in query_result.artifacts {
        artifacts.insert(artifact.id, artifact);
    }
    let mut files = HashMap::new();
    for file in query_result.dep_set_of_files {
        files.insert(file.id, file);
    }
    let mut fragments = HashMap::new();
    for fragment in query_result.path_fragments {
        fragments.insert(fragment.id, fragment);
    }

    // construct all input files and filter out non-swift files
    // since sourcekit-lsp only cares about swift
    for action in query_result.actions {
        for id in action.input_dep_set_ids {
            let mut artifact_ids: Vec<u8> = vec![];
            let mut path_ids: Vec<u8> = vec![];
            let mut input_files: Vec<String> = vec![];

            let file_set = files.get(&id).unwrap();
            let directive = file_set.direct_artifact_ids.clone();
            let transitive = file_set.transitive_dep_set_ids.clone();

            println!("inputDepSetId: {:?}", &id);
            if let Some(mut directive) = directive {
                artifact_ids.append(&mut directive);
            } else if let Some(transitive) = transitive {
                for id in transitive {
                    let file_set = files.get(&id).unwrap();
                    let mut directive = file_set.direct_artifact_ids.clone().unwrap();
                    artifact_ids.append(&mut directive);
                }
            } else {
                panic!("Action has input files but no artifact_ids found.");
            }

            // println!("artifact_ids: {:?}", artifact_ids);

            for id in artifact_ids {
                let artifact = artifacts.get(&id).unwrap();
                path_ids.push(artifact.path_fragment_id);
            }

            // println!("path_ids: {:?}", path_ids);
            for id in path_ids {
                let mut file_path = String::new();
                let mut fragment = fragments.get(&id).unwrap();
                while let Some(parent_id) = fragment.parent_id {
                    let parent = fragments.get(&parent_id).unwrap();
                    if !file_path.is_empty() {
                        file_path.insert_str(0, "/");
                    }
                    file_path.insert_str(0, &fragment.label);
                    fragment = parent;
                }
                // println!("file_path: {:?}", file_path);
                input_files.push(file_path); 
            }

            println!("input_files: {:?}", input_files);
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    artifacts: Vec<Artifact>,
    actions: Vec<Action>,
    targets: Vec<Target>,
    rule_classes: Vec<RuleClass>,
    dep_set_of_files: Vec<DepSetOfFiles>,
    path_fragments: Vec<PathFragment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    id: u8,
    path_fragment_id: u8,
    is_tree_artifact: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    target_id: u8,
    action_key: String,
    mnemonic: String,
    configuration_id: u8,
    arguments: Vec<String>,
    environment_variables: Vec<EnvironmentVariable>,
    input_dep_set_ids: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    id: u8,
    label: String,
    rule_class_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct RuleClass {
    id: u8,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentVariable {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathFragment {
    id: u8,
    label: String,
    parent_id: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepSetOfFiles {
    id: u8,
    direct_artifact_ids: Option<Vec<u8>>,
    transitive_dep_set_ids: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub enum FileSet {
    direct(Vec<u8>),
    transistive(Vec<u8>),
}
