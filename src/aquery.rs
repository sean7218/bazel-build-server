// #![allow(dead_code)]
mod query_result;
use query_result::{Action, Artifact, DepSetOfFiles, PathFragment, QueryResult};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_string_pretty};
use std::{
    collections::HashMap, fmt, path::PathBuf, process::Command
};


/// Outputs list of targets, each target should have set of input files
/// params:
///   - target: full name of the target (example: //Libraries/Utils:UtilsLib)
///   - current_dir: the directory where the bazel WORKSPACE is
pub fn aquery(target: &str, current_dir: &str) {
    let mnemonic = format!("mnemonic(\"SwiftCompile\", deps({}))", target);
    let output = Command::new("bazel")
        .args(&[
            "aquery",
            &mnemonic,
            "--output=jsonproto",
        ])
        .current_dir(current_dir)
        .output()
        .expect("Failed to start aquery process");

    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    let query_result: QueryResult = from_slice(&output.stdout)
        .expect("Failed to parse output");

    // convert array into hashmap to reduce time complexity
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

    // construct all input files 
    let mut bazel_targets: Vec<BazelTarget> = vec![];
    for action in query_result.actions {
        let input_files = build_input_files(
            &artifacts,
            &files, 
            &fragments, 
            &action
        );

        let mut compiler_arguments: Vec<String> = vec![];
        for arg in action.arguments {
            // println!("{}", arg);
            if arg.contains("-Xwrapped-swift") {
                // skip
            } else if arg.contains("__BAZEL_XCODE_SDKROOT__") {
                let _arg = arg.replace(
                    "__BAZEL_XCODE_SDKROOT__",
                    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk"
                );
                compiler_arguments.push(_arg);
            } else {
                compiler_arguments.push(arg);
            }
        }
        // println!("args: {:?}", compiler_arguments);

        let target = query_result.targets
            .iter()
            .find(|t| t.id == 1)
            .unwrap();
        let bazel_target = BazelTarget {
            id: action.target_id,
            label: target.label.clone(),
            input_files,
            compiler_arguments
        };
        bazel_targets.push(bazel_target);
    }

    let targets = serde_json::to_value(bazel_targets).expect("");
    let str = to_string_pretty(&targets).expect("");
    println!("bazel_targets: {}", str);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BazelTarget {
    pub id: u8,
    pub label: String,
    pub input_files: Vec<String>,
    pub compiler_arguments: Vec<String>,
}

pub fn build_input_files(
    artifacts: &HashMap<u8, Artifact>,
    files: &HashMap<u8, DepSetOfFiles>,
    fragments: &HashMap<u8, PathFragment>,
    action: &Action
) -> Vec<String> {
    let mut input_files: Vec<String> = vec![];
    for id in action.input_dep_set_ids.clone() {
        let file_set = files.get(&id).unwrap();
        let artifact_ids = build_artifact_ids(file_set, &files);

        // println!("artifact_ids: {:?}", artifact_ids);

        let mut path_ids: Vec<u8> = vec![];
        for id in artifact_ids {
            let artifact = artifacts.get(&id).unwrap();
            path_ids.push(artifact.path_fragment_id);
        }

        // println!("path_ids: {:?}", path_ids);

        for id in path_ids {
            let leaf = fragments.get(&id).unwrap();
            let file_path = build_file_path(&fragments, leaf);
            input_files.push(file_path); 
        }

        // println!("input_files: {:?}", input_files);
    }
    return input_files;
}


/// each file set can have both direct ids and transitive sets
///  "depSetOfFiles": [
///    {
///      "id": 2,
///      "directArtifactIds": [1, 2, 3, 4],
///      "transitiveDepSetIds": [2, 3]
///    },
///  ]
///  return artifact_ids
pub fn build_artifact_ids(
    file_set: &DepSetOfFiles,
    files: &HashMap<u8, DepSetOfFiles>
) -> Vec<u8> {
    let direct_ids = file_set.direct_artifact_ids.clone();
    let transitive_ids = file_set.transitive_dep_set_ids.clone();

    // take care the direct files
    let mut artifact_ids: Vec<u8> = vec![];
    if let Some(mut direct_ids) = direct_ids {
        artifact_ids.append(&mut direct_ids);
    }

    // take care the transitive files
    if let Some(transitive) = transitive_ids {
        for id in transitive {
            let file_set = files.get(&id).unwrap();
            let mut directive = file_set.direct_artifact_ids.clone().unwrap();
            artifact_ids.append(&mut directive);
        }
    }
    return artifact_ids;
}

/// building files list until the parent id is none
/// "pathFragments": [
///    { "id": 1, "label": "Button.swift", "parentId": 2 },
///    { "id": 2, "label": "Components", "parentId": 3 },
///    { "id": 3, "label": "Sources", "parentId": None },
/// ]
pub fn build_file_path(
    fragments: &HashMap<u8, PathFragment>,
    leaf: &PathFragment
) -> String {
    let mut file_path = String::new();
    let mut current = Some(leaf);
    while let Some(fragment) = current {
        if fragment.parent_id.is_some() {
            file_path.insert_str(0, &format!("/{}", fragment.label));
        } else {
            file_path.insert_str(0, &fragment.label);
        }

        if let Some(parent_id) = fragment.parent_id {
            current = fragments.get(&parent_id);
        } else {
            current = None;
        }
    }

    // println!("file_path: {:?}", file_path);

    return file_path;
}

mod tests {
    use std::collections::HashMap;

    use super::{build_artifact_ids, build_file_path, query_result::{DepSetOfFiles, PathFragment}};

    #[test]
    fn test_build_artifact_ids() {
        let one = DepSetOfFiles {
            id: 1,
            direct_artifact_ids: Some(vec![11]),
            transitive_dep_set_ids: Some(vec![2, 3])
        };
        let two = DepSetOfFiles {
            id: 2,
            direct_artifact_ids: Some(vec![12]),
            transitive_dep_set_ids: None,
        };
        let three = DepSetOfFiles {
            id: 3,
            direct_artifact_ids: Some(vec![13]),
            transitive_dep_set_ids: None,
        };
        let mut files = HashMap::new();
        files.insert(1, one);
        files.insert(2, two);
        files.insert(3, three);
        let file = files.get(&1).unwrap();
        let artifact_ids = build_artifact_ids(&file, &files);
        let expected_ids = vec![11, 12, 13];
        assert_eq!(artifact_ids, expected_ids);
    }

    #[test]
    fn test_build_file_path() {
        let leaf = PathFragment {
            id: 1,
            label: "Button.swift".to_string(),
            parent_id: Some(2)
        };
        let parent = PathFragment {
            id: 2,
            label: "Components".to_string(),
            parent_id: Some(3)
        };
        let root = PathFragment {
            id: 3,
            label: "Sources".to_string(),
            parent_id: None
        };
        let mut fragments = HashMap::new();
        fragments.insert(1, leaf.clone());
        fragments.insert(2, parent.clone());
        fragments.insert(3, root.clone());

        let out = build_file_path(&fragments, &leaf.clone());
        let expected = "Sources/Components/Button.swift".to_string();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_input_files() {
        let dir = std::env::current_dir()
            .expect("Failed to find current_dir!")
            .join("example");

        super::aquery(
            "//Sources/Components",
            dir.to_str().unwrap()
        );
    }
}
