// #![allow(dead_code)]
mod query_result;
use query_result::QueryResult;
use serde_json::from_slice;
use std::{
    collections::HashMap,
    process::{Command},
};


/// Outputs list of targets, each target should have set of input files
/// and
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
    for action in query_result.actions {
        for id in action.input_dep_set_ids {
            let mut artifact_ids: Vec<u8> = vec![];
            let mut path_ids: Vec<u8> = vec![];
            let mut input_files: Vec<String> = vec![];

            let file_set = files.get(&id).unwrap();
            let directive = file_set.direct_artifact_ids.clone();
            let transitive = file_set.transitive_dep_set_ids.clone();

            // println!("inputDepSetId: {:?}", &id);
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


mod test {
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