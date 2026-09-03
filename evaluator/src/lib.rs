use anyhow;

use checks::check_result::CheckResult;

pub mod checks;

pub fn evaluate(code: &str) -> anyhow::Result<CheckResult> {
    Ok(checks::compiler::check(code)?)
}
