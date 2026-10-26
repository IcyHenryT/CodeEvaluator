use std::fs;
use std::io;
use std::process::Command;

use crate::checks::check_result::CheckResult;
use crate::checks::compiler::TempProject;

pub fn test(code: &str, tests: &str, models: &str) -> io::Result<CheckResult> {
    let project = TempProject::new(code, models)?;
    let tests_directory = project.path.join("tests");
    fs::create_dir(&tests_directory)?;
    fs::write(tests_directory.join("evaluation.rs"), tests)?;

    let output = Command::new("cargo")
        .args([
            "test",
            "--quiet",
            "--test",
            "evaluation",
            "--",
            "--nocapture",
        ])
        .current_dir(&project.path)
        .output()?;

    Ok(CheckResult::from_output(output))
}
