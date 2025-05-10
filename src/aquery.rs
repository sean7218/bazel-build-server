use std::{io::{Read, Stderr}, process::{Command, Stdio}};
use serde::Deserialize;
use serde_json;

#[allow(dead_code)]
pub fn aquery() {

    /*
    let mut child = Command::new("bazel")
        .args(&[
            "aquery",
            "mnemonic(\"SwiftCompile\", deps(//Sources/Components))",
            "--output=jsonproto"
        ])
        .current_dir("/Users/sean7218/bazel/buildserver/example/")
        .stdout(Stdio::piped())
        .spawn()
        .expect("bazel build");

    let output = child.wait_with_output().expect("");
    let stdout = String::from_utf8(output.stdout);
    println!("{:?}", stdout);
    */

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

    let query_json = output.stdout;
    let query_result = serde_json::from_slice::<QueryResult>(&query_json)
        .expect("failed to parse output");
    let actions = query_result.actions;
    for action in actions {
        let args = action.arguments;
        println!("{:?}", args);
    }
    
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    actions: Vec<Action>,
    targets: Vec<Target>,
    rule_classes: Vec<RuleClass>,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    id: u8,
    label: String,
    rule_class_id: u8
}

#[derive(Debug, Deserialize)]
pub struct RuleClass {
    id: u8,
    name: String
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentVariable {
    key: String,
    value: String
}
