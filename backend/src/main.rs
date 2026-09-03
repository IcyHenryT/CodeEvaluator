use anyhow::Result;
use std::process::ExitCode;

use evaluator::evaluate;

fn main() -> Result<ExitCode> {
    let result = evaluate(
        r#"fn main() {
            println!("hello");
        }"#,
    )?;

    if result.success {
        println!("Compiler check passed!");
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("Compiler check failed!");
        Ok(ExitCode::FAILURE)
    }
}
