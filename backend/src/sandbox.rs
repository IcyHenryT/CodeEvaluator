use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use evaluator::checks::check_result::CheckResult;
use evaluator::{EvaluationResult, Requirements, analyze_syntax, requirement_feedback};

const IMAGE: &str = "code-evaluator-sandbox";
const OUTPUT_LIMIT: usize = 64 * 1024;
const INPUT_LIMIT: usize = 1024 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Container(String);
impl Drop for Container {
    fn drop(&mut self) {
        let _ = run_docker(&["rm", "-f", &self.0], None, CONTROL_TIMEOUT);
    }
}

fn capture_output(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut retained = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let keep = count.min(OUTPUT_LIMIT.saturating_sub(retained.len()));
        retained.extend_from_slice(&buffer[..keep]);
    }
    Ok(retained)
}

fn read_output(reader: impl Read + Send + 'static) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(capture_output(reader));
    });
    rx
}

fn run_docker(args: &[&str], input: Option<Vec<u8>>, budget: Duration) -> Result<CheckResult> {
    let deadline = Instant::now() + budget;
    let mut child = Command::new("docker")
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Docker can deadlock if one pipe fills while we're waiting for the other!
    let stdout = read_output(child.stdout.take().unwrap());
    let stderr = read_output(child.stderr.take().unwrap());
    if let Some(bytes) = input {
        let mut stdin = child.stdin.take().unwrap();
        thread::spawn(move || {
            let _ = stdin.write_all(&bytes);
        });
    }
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            outcome => {
                let _ = child.kill();
                thread::spawn(move || {
                    let _ = child.wait();
                });
                if let Err(error) = outcome {
                    return Err(error.into());
                }
                break None;
            }
        }
    };

    let collect = |rx: mpsc::Receiver<std::io::Result<Vec<u8>>>| -> Result<String> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        Ok(
            String::from_utf8_lossy(&rx.recv_timeout(remaining).map_err(|error| {
                anyhow::anyhow!("Docker output did not close before controller deadline: {error}")
            })??)
            .into_owned(),
        )
    };
    let mut result = CheckResult {
        success: status.is_some_and(|status| status.success()),
        stdout: collect(stdout)?,
        stderr: collect(stderr)?,
    };

    if status.is_some_and(|status| matches!(status.code(), Some(124 | 137))) {
        result
            .stderr
            .push_str("\nSandbox deadline exceeded or resource limit terminated the process.");
    }
    if status.is_none() {
        result
            .stderr
            .push_str("\nDocker controller deadline exceeded");
    }
    Ok(result)
}

fn require_docker_success(args: &[&str], input: Option<Vec<u8>>) -> Result<()> {
    let result = run_docker(args, input, CONTROL_TIMEOUT)?;
    if !result.success {
        bail!("Docker sandbox operation failed: {}", result.stderr);
    }
    Ok(())
}

fn archive_sources(files: &[(&str, &str)]) -> Result<Vec<u8>> {
    let total: usize = files.iter().map(|(_, contents)| contents.len()).sum();
    if total > INPUT_LIMIT {
        bail!("sandbox project exceeds 1 MiB");
    }
    let mut tar = Vec::new();
    for (name, contents) in files {
        if name.len() > 100
            || name.starts_with('/')
            || name.split('/').any(|part| part == ".." || part.is_empty())
        {
            bail!("invalid sandbox archive path");
        }

        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[100..108].copy_from_slice(b"0000644\0");
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{:011o}\0", contents.len()).as_bytes());
        header[136..148].copy_from_slice(b"00000000000\0");
        // tar rejects unless the checksum treats its own field as spaces.
        header[148..156].fill(b' ');
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u32 = header.iter().map(|byte| *byte as u32).sum();
        header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        tar.extend_from_slice(&header);
        tar.extend_from_slice(contents.as_bytes());

        tar.resize(tar.len().next_multiple_of(512), 0);
    }

    tar.extend_from_slice(&[0; 1024]);
    Ok(tar)
}

pub struct SandboxEvaluation {
    pub result: EvaluationResult,
    pub compilation_ms: f64,
    pub runtime_ms: Option<f64>,
    pub requirement_feedback: Vec<String>,
}

pub fn evaluate(
    source: &str,
    tests: &str,
    models: &str,
    requirements: Requirements,
) -> Result<SandboxEvaluation> {
    let lib = format!("mod models;\npub use models::*;\n\n{source}");
    let project = archive_sources(&[
        ("lib.rs", &lib),
        ("models.rs", models),
        ("evaluation.rs", tests),
    ])?;
    let name = format!(
        "code-evaluator-{}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );

    let container = Container(name);
    // restrict privileges & network
    require_docker_success(
        &[
            "create",
            "--name",
            &container.0,
            "--network",
            "none",
            "--read-only",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--memory",
            "512m",
            "--memory-swap",
            "512m",
            "--cpus",
            "1",
            "--pids-limit",
            "64",
            "--env",
            "TERM=xterm-256color",
            "--user",
            "65534:65534",
            "--tmpfs",
            "/work:rw,exec,nosuid,size=128m,mode=1777",
            "--tmpfs",
            "/tmp:rw,nosuid,noexec,size=16m",
            "--log-driver",
            "none",
            IMAGE,
            // The binary must survive
            "sleep",
            "infinity",
        ],
        None,
    )?;
    require_docker_success(&["start", &container.0], None)?;
    require_docker_success(
        &[
            "exec",
            "-i",
            &container.0,
            "tar",
            "--no-same-owner",
            "-xf",
            "-",
            "-C",
            "/work",
        ],
        Some(project),
    )?;
    let compilation_start = Instant::now();
    let compilation = run_docker(
        &[
            "exec",
            "-w",
            "/work",
            &container.0,
            "timeout",
            "--signal=KILL",
            "30s",
            "sh",
            "-c",
            "rustc --color=always --edition=2024 --crate-name submission --crate-type lib lib.rs -o libsubmission.rlib && rustc --color=always --edition=2024 --test evaluation.rs --extern submission=/work/libsubmission.rlib -o /work/evaluation",
        ],
        None,
        Duration::from_secs(32),
    )?;
    let compilation_ms = compilation_start.elapsed().as_secs_f64() * 1000.0;
    if !compilation.success {
        return Ok(SandboxEvaluation {
            result: EvaluationResult {
                success: false,
                compilation,
                behavior: None,
                syntax: None,
                requirements_met: None,
            },
            compilation_ms,
            runtime_ms: None,
            requirement_feedback: Vec::new(),
        });
    }
    let syntax = analyze_syntax(source)?;
    let requirement_feedback = requirement_feedback(source, &requirements)?;
    let requirements_met = requirement_feedback.is_empty();
    let behavior = run_docker(
        &[
            "exec",
            "-w",
            "/work",
            &container.0,
            "bash",
            "-c",
            "TIMEFORMAT='%R'; { time timeout --signal=KILL 5s /work/evaluation --nocapture --color=always 2>&3; } 3>&2 2>/work/runtime.txt",
        ],
        None,
        Duration::from_secs(7),
    )?;
    let runtime_ms = run_docker(
        &["exec", &container.0, "cat", "/work/runtime.txt"],
        None,
        CONTROL_TIMEOUT,
    )
    .ok()
    .filter(|output| output.success)
    .and_then(|output| output.stdout.trim().parse::<f64>().ok())
    .filter(|seconds| seconds.is_finite() && *seconds >= 0.0 && *seconds <= 10.0)
    .map(|seconds| seconds * 1000.0);
    Ok(SandboxEvaluation {
        result: EvaluationResult {
            success: behavior.success && requirements_met,
            compilation,
            behavior: Some(behavior),
            syntax: Some(syntax),
            requirements_met: Some(requirements_met),
        },
        compilation_ms,
        runtime_ms,
        requirement_feedback,
    })
}
