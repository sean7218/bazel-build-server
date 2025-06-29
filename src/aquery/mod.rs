#![allow(dead_code)]
mod query_result;
use crate::error::Result;
use query_result::{Action, Artifact, DepSetOfFiles, PathFragment, QueryResult};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
    process::Command,
};
use url::Url;

use crate::log_str;

/// Outputs list of targets, each target should have set of input files
/// params:
///   - target: full name of the target (example: //Libraries/Utils:UtilsLib)
///   - current_dir: the directory where the bazel WORKSPACE is
pub fn aquery(
    target: &str,
    current_dir: &PathBuf,
    execroot_path: &PathBuf,
    sdk: &str,
    aquery_args: &[String],
    extra_includes: &[String],
    extra_frameworks: &[String],
) -> Result<Vec<BazelTarget>> {
    let mut command_args: Vec<String> = vec![];
    let mnemonic = format!("mnemonic(\"SwiftCompile\", deps({}))", target);

    command_args.push(String::from("aquery"));
    command_args.push(mnemonic);
    command_args.push(String::from("--output=jsonproto"));
    command_args.extend(aquery_args.to_owned());

    log_str!("✨ aquery command: {:#?}", &command_args);

    let output = Command::new("bazel")
        .args(command_args)
        .current_dir(current_dir.clone())
        .output()?;

    let query_result: QueryResult = from_slice(&output.stdout)?;

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

    let is_swift = |url: &Url| -> bool { url.as_str().ends_with(".swift") };
    let to_url = |s: &String| -> Option<Url> {
        let path = current_dir.join(s);
        match Url::from_file_path(path) {
            Ok(v) => return Some(v),
            Err(e) => {
                log_str!("{:#?}", &e);
                return None;
            }
        }
    };

    // construct all input files
    let mut bazel_targets: Vec<BazelTarget> = vec![];
    for action in query_result.actions {
        let input_files: Vec<Url> = build_input_files(&artifacts, &files, &fragments, &action)
            .iter()
            .filter_map(to_url)
            .filter(is_swift)
            .collect();

        let mut compiler_arguments: Vec<String> = vec![];

        let mut index: usize = 0;
        let count = action.arguments.iter().count();
        while index < count {
            let arg = action.arguments.get(index).unwrap().clone();

            if arg.contains("-Xfrontend") {
                if let Some(next) = action.arguments.get(index + 1) {
                    if next.contains("-const-gather-protocols-file") {
                        index += 2;
                        continue;
                    }
                }
            }

            if arg.contains("-Xfrontend") {
                if let Some(next) = action.arguments.get(index + 1) {
                    if next.contains("const_protocols_to_gather.json") {
                        index += 2;
                        continue;
                    }
                }
            }

            if arg.contains("__BAZEL_XCODE_SDKROOT__") {
                let _arg = arg.replace("__BAZEL_XCODE_SDKROOT__", &sdk);
                compiler_arguments.push(_arg);
                index += 1;
                continue;
            }

            if arg.contains("-Xwrapped-swift")
                || arg.ends_with("worker")
                || arg.starts_with("swiftc")
            {
                index += 1;
                continue;
            }

            if arg.contains("bazel-out/") {
                let prefix = format!("{}/bazel-out/", execroot_path.to_string_lossy());
                let _arg = arg.replace("bazel-out/", &prefix);
                compiler_arguments.push(_arg);
                index += 1;
                continue
            }

            if arg.contains("external/") {
                let prefix = format!("{}/external/", execroot_path.to_string_lossy());
                let _arg = arg.replace("external/", &prefix);
                compiler_arguments.push(_arg);
                index += 1;
                continue;
            }

            compiler_arguments.push(arg);
            index += 1;
        }

        let target = query_result
            .targets
            .iter()
            .find(|t| t.id == action.target_id)
            .ok_or("target_id not found")?;

        let uri = bazel_to_uri(&current_dir, &target.label, &target.id)?;

        // adding addtion include paths for swiftmodules
        for include in extra_includes {
            let arg = format!("-I{}", include);
            compiler_arguments.push(arg);
        }

        // adding extra framework search paths
        for fmwk in extra_frameworks {
            let arg = format!("-F{}", fmwk);
            compiler_arguments.push(arg);
        }

        let bazel_target = BazelTarget {
            id: action.target_id,
            uri: uri,
            label: target.label.clone(),
            input_files,
            compiler_arguments,
        };
        bazel_targets.push(bazel_target);
    }

    // dedup bazel_targets
    let target_set: HashSet<BazelTarget> = bazel_targets.into_iter().collect();
    let targets: Vec<BazelTarget> = target_set.into_iter().collect();
    Ok(targets)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BazelTarget {
    pub id: u32,
    pub uri: Url,
    pub label: String,
    pub input_files: Vec<Url>,
    pub compiler_arguments: Vec<String>,
}

impl PartialEq for BazelTarget {
    fn eq(&self, other: &Self) -> bool {
        self.uri.eq(&other.uri) && self.id.eq(&other.id)
    }
}

impl Eq for BazelTarget {}

impl Hash for BazelTarget {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.uri.hash(state);
    }
}

pub fn build_input_files(
    artifacts: &HashMap<u32, Artifact>,
    files: &HashMap<u32, DepSetOfFiles>,
    fragments: &HashMap<u32, PathFragment>,
    action: &Action,
) -> Vec<String> {
    let mut input_files: Vec<String> = vec![];
    for id in action.input_dep_set_ids.clone() {
        let file_set = files.get(&id).unwrap();
        let artifact_ids = build_artifact_ids(file_set, &files);

        // println!("artifact_ids: {:?}", artifact_ids);

        let mut path_ids: Vec<u32> = vec![];
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
    files: &HashMap<u32, DepSetOfFiles>,
) -> Vec<u32> {
    let direct_ids = file_set.direct_artifact_ids.clone();
    let transitive_ids = file_set.transitive_dep_set_ids.clone();

    // take care the direct files
    let mut artifact_ids: Vec<u32> = vec![];
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
pub fn build_file_path(fragments: &HashMap<u32, PathFragment>, leaf: &PathFragment) -> String {
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

/// Convert bazel target name to Uri-compatible encoding
pub fn bazel_to_uri(base: &PathBuf, name: &String, id: &u32) -> Result<Url> {
    let trimmed = name.trim_start_matches("//");
    let joined = base.join(trimmed).join(id.to_string());
    let url = Url::from_file_path(joined).map_err(|_| "failed to create uri for target".into());
    return url;
}
