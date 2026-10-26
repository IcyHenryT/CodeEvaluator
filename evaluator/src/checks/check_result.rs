#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CheckResult {
    pub fn from_output(output: std::process::Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}
