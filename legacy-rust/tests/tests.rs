
// use std::{collections::HashMap, path::PathBuf};
// use buildserver::aquery::{
//     bazel_to_uri, build_artifact_ids, build_file_path,
//     query_result::{DepSetOfFiles, PathFragment},
// };

// #[test]
// fn test_bazel_to_uri() {
//     let base = PathBuf::from("/Users/sean7218/bazel/hello-bazel/");
//     let name = String::from("//Sources/Components:Components");
//     let uri = bazel_to_uri(&base, &name);
//     println!("{:?}", uri);

//     let ext = String::from("@rules_swift//swift:toolchain");
//     let uri = bazel_to_uri(&base, &ext);
//     println!("{:?}", uri);

//     let ext = String::from("@my~lib//src:core-lib");
//     let uri = bazel_to_uri(&base, &ext);
//     println!("{:?}", uri);

//     let ext = String::from("@vendor.pkg//:init");
//     let uri = bazel_to_uri(&base, &ext);
//     println!("{:?}", uri);
// }

// #[test]
// fn test_build_artifact_ids() {
//     let one = DepSetOfFiles {
//         id: 1,
//         direct_artifact_ids: Some(vec![11]),
//         transitive_dep_set_ids: Some(vec![2, 3]),
//     };
//     let two = DepSetOfFiles {
//         id: 2,
//         direct_artifact_ids: Some(vec![12]),
//         transitive_dep_set_ids: None,
//     };
//     let three = DepSetOfFiles {
//         id: 3,
//         direct_artifact_ids: Some(vec![13]),
//         transitive_dep_set_ids: None,
//     };
//     let mut files = HashMap::new();
//     files.insert(1, one);
//     files.insert(2, two);
//     files.insert(3, three);
//     let file = files.get(&1).unwrap();
//     let artifact_ids = build_artifact_ids(&file, &files);
//     let expected_ids = vec![11, 12, 13];
//     assert_eq!(artifact_ids, expected_ids);
// }

// #[test]
// fn test_build_file_path() {
//     let leaf = PathFragment {
//         id: 1,
//         label: "Button.swift".to_string(),
//         parent_id: Some(2),
//     };
//     let parent = PathFragment {
//         id: 2,
//         label: "Components".to_string(),
//         parent_id: Some(3),
//     };
//     let root = PathFragment {
//         id: 3,
//         label: "Sources".to_string(),
//         parent_id: None,
//     };
//     let mut fragments = HashMap::new();
//     fragments.insert(1, leaf.clone());
//     fragments.insert(2, parent.clone());
//     fragments.insert(3, root.clone());

//     let out = build_file_path(&fragments, &leaf.clone());
//     let expected = "Sources/Components/Button.swift".to_string();
//     assert_eq!(out, expected);
// }

// #[test]
// fn test_input_files() {
//     let dir = std::env::current_dir()
//         .expect("Failed to find current_dir!")
//         .join("example");

//     let targets = super::aquery("//Sources/Components", &dir);

//     for target in targets {
//         println!("{:#?}", target);
//     }
// }
