use std::process::Command;

#[allow(dead_code)]
pub fn aquery() {
    let output = Command::new("echo")
        .arg("Hello World")
        .output()
        .expect("Failed to start aquery process");
    let output_string = String::from_utf8(output.stdout);

    println!("Output: {:?}", output_string);
}
