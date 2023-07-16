use std::{path::PathBuf, process::Command};

use assert_cmd::prelude::*;

pub fn run_with_args(end_args: Vec<String>) -> Result<Command, Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("hypnagogic-cli")?;

    let mut args = vec![];
    args.push("--dont-wait".to_string());
    args.push("--templates".to_string());
    let mut templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    templates_dir.pop();
    templates_dir.push("templates");
    args.push(templates_dir.to_str().unwrap().to_string());
    args.extend(end_args);
    command.args(args);

    Ok(command)
}
