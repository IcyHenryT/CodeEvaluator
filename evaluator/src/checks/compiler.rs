use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::checks::check_result::CheckResult;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub fn check(code: &str) -> io::Result<CheckResult> {
    let project = TempProject::new(code)?;
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&project.path)
        .output()?;

    Ok(CheckResult {
        success: output.status.success(),
    })
}

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn new(code: &str) -> io::Result<Self> {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("code-evaluator-{}-{id}", std::process::id()));

        fs::create_dir_all(path.join("src"))?;
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"submission\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )?;
        fs::write(path.join("src/main.rs"), code)?;

        Ok(Self { path })
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
