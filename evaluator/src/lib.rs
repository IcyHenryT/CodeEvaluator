use checks::check_result::CheckResult;

pub use checks::requirements::{Require, Requirements, meets_requirements, requirement_feedback};
pub use checks::syntax::{SyntaxAnalysis, analyze_requirement_syntax, analyze_syntax};

pub mod checks;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    pub success: bool,
    pub compilation: CheckResult,
    pub behavior: Option<CheckResult>,
    pub syntax: Option<SyntaxAnalysis>,
    pub requirements_met: Option<bool>,
}

pub fn evaluate(
    code: &str,
    tests: &str,
    models: &str,
    requirements: Requirements,
) -> anyhow::Result<EvaluationResult> {
    let compilation = checks::compiler::check(code, models)?;

    if !compilation.success {
        return Ok(EvaluationResult {
            success: false,
            compilation,
            behavior: None,
            syntax: None,
            requirements_met: None,
        });
    }

    let syntax = checks::syntax::analyze_syntax(code)?;
    let requirements_met = checks::requirements::meets_requirements(code, &requirements)?;
    let behavior = checks::behavior::test(code, tests, models)?;

    Ok(EvaluationResult {
        success: behavior.success && requirements_met,
        compilation,
        behavior: Some(behavior),
        syntax: Some(syntax),
        requirements_met: Some(requirements_met),
    })
}
