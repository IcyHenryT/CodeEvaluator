use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use evaluator::{Require, Requirements};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

mod challenges;
mod sandbox;

#[derive(Clone)]
struct AppState {
    challenges_path: PathBuf,
    evaluations: Arc<Semaphore>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        evaluations: Arc::new(Semaphore::new(2)),
        challenges_path: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("challenges"),
    };
    let app = Router::new()
        .route("/api/submit", post(submit))
        .route("/api/challenges", get(challenges::list))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    Ok(axum::serve(listener, app).await?)
}

#[derive(Deserialize)]
struct SubmitRequest {
    challenge: String,
    submission: String,
}

#[derive(Serialize)]
struct SubmitResponse {
    success: bool,
    compilation: Option<CheckOutput>,
    behavior: Option<CheckOutput>,
    requirements_met: Option<bool>,
    requirement_feedback: Vec<String>,
    timing: Option<EvaluationTiming>,
    error: Option<String>,
}

#[derive(Serialize)]
struct EvaluationTiming {
    compilation_ms: f64,
    runtime_ms: Option<f64>,
}

#[derive(Serialize)]
struct CheckOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

async fn submit(
    State(state): State<AppState>,
    Json(request): Json<SubmitRequest>,
) -> (StatusCode, Json<SubmitResponse>) {
    if !is_valid_challenge_name(&request.challenge) {
        return error_response(StatusCode::BAD_REQUEST, "invalid challenge name");
    }

    let challenge_path = state.challenges_path.join(&request.challenge);
    if !challenge_path.is_dir() {
        return error_response(StatusCode::NOT_FOUND, "challenge not found");
    }

    let permit = match state.evaluations.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "sandbox is busy; retry later",
            );
        }
    };
    let submission = request.submission;
    let evaluation = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let (tests, models, requirements) = load_challenge(&challenge_path)?;
        sandbox::evaluate(&submission, &tests, &models, requirements)
    })
    .await;

    let sandbox::SandboxEvaluation {
        result,
        compilation_ms,
        runtime_ms,
        requirement_feedback,
    } = match evaluation {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("evaluation failed: {error:#}"),
            );
        }
        Err(error) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("evaluation task failed: {error}"),
            );
        }
    };

    let compilation = CheckOutput {
        success: result.compilation.success,
        stdout: result.compilation.stdout,
        stderr: result.compilation.stderr,
    };
    let behavior = result.behavior.map(|check| CheckOutput {
        success: check.success,
        stdout: check.stdout,
        stderr: check.stderr,
    });

    (
        StatusCode::OK,
        Json(SubmitResponse {
            success: result.success,
            compilation: Some(compilation),
            behavior,
            requirements_met: result.requirements_met,
            requirement_feedback,
            timing: Some(EvaluationTiming {
                compilation_ms,
                runtime_ms,
            }),
            error: None,
        }),
    )
}

fn is_valid_challenge_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<SubmitResponse>) {
    (
        status,
        Json(SubmitResponse {
            success: false,
            compilation: None,
            behavior: None,
            requirements_met: None,
            requirement_feedback: Vec::new(),
            timing: None,
            error: Some(message.to_owned()),
        }),
    )
}

fn load_challenge(path: &Path) -> Result<(String, String, Requirements)> {
    let tests = fs::read_to_string(path.join("tests.rs"))?;
    let models = fs::read_to_string(path.join("models.rs"))?;
    let json = fs::read_to_string(path.join("requirements.json"))?;
    let requirements: RequirementsFile = serde_json::from_str(&json)?;
    Ok((tests, models, requirements.into_requirements()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequirementsFile {
    entry_point: Option<String>,
    loops: Option<Range<u8>>,
    #[serde(rename = "unsafe")]
    unsafe_allowed: Option<bool>,
    recursion: Option<bool>,
    clones: Option<Range<u8>>,
    variables: Option<Range<u8>>,
    mutable_bindings: Option<Range<u8>>,
    assignments: Option<Range<u8>>,
    if_expressions: Option<Range<u8>>,
    while_loops: Option<Range<u8>>,
    for_loops: Option<Range<u8>>,
}

impl RequirementsFile {
    fn into_requirements(self) -> Requirements {
        [
            self.entry_point.map(Require::EntryPoint),
            self.loops.map(Require::Loops),
            self.unsafe_allowed.map(Require::Unsafe),
            self.recursion.map(Require::Recursion),
            self.clones.map(Require::Clones),
            self.variables.map(Require::Variables),
            self.mutable_bindings.map(Require::MutableBindings),
            self.assignments.map(Require::Assignments),
            self.if_expressions.map(Require::IfExpressions),
            self.while_loops.map(Require::WhileLoops),
            self.for_loops.map(Require::ForLoops),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
