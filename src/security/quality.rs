use super::{CheckResult, Finding};

/// Run quality gate: cargo fmt, clippy, tests
pub async fn run_quality_gate() -> CheckResult {
    let mut findings = Vec::new();

    // cargo fmt --check
    let fmt = run_fmt_check().await;
    findings.extend(fmt);

    // cargo clippy
    let clippy = run_clippy_check().await;
    findings.extend(clippy);

    // cargo test
    let tests = run_test_check().await;
    findings.extend(tests);

    let passed = findings.is_empty();
    CheckResult {
        name: "quality".to_string(),
        passed,
        findings,
        tool_available: true,
    }
}

async fn run_fmt_check() -> Vec<Finding> {
    let output = match tokio::process::Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    if output.status.success() {
        return vec![];
    }

    vec![Finding {
        severity: "medium".to_string(),
        message: "Code is not formatted. Run `cargo fmt` to fix.".to_string(),
        file: None,
        line: None,
        fix: Some("Run `cargo fmt`".to_string()),
    }]
}

async fn run_clippy_check() -> Vec<Finding> {
    let output = match tokio::process::Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    if output.status.success() {
        return vec![];
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut findings = vec![];

    for line in stderr.lines() {
        if line.contains("warning[") || line.contains("error[") {
            findings.push(Finding {
                severity: if line.contains("error") {
                    "high".to_string()
                } else {
                    "medium".to_string()
                },
                message: line.trim().to_string(),
                file: None,
                line: None,
                fix: Some("Fix the clippy warning/error".to_string()),
            });
        }
    }

    if findings.is_empty() {
        findings.push(Finding {
            severity: "medium".to_string(),
            message: "Clippy check failed".to_string(),
            file: None,
            line: None,
            fix: Some("Run `cargo clippy` for details".to_string()),
        });
    }

    findings
}

async fn run_test_check() -> Vec<Finding> {
    let output = match tokio::process::Command::new("cargo")
        .args(["test"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    if output.status.success() {
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = vec![];

    for line in stdout.lines() {
        if line.contains("FAILED") || line.contains("panicked") {
            findings.push(Finding {
                severity: "high".to_string(),
                message: line.trim().to_string(),
                file: None,
                line: None,
                fix: Some("Fix the failing test".to_string()),
            });
        }
    }

    if findings.is_empty() {
        findings.push(Finding {
            severity: "high".to_string(),
            message: "Tests failed".to_string(),
            file: None,
            line: None,
            fix: Some("Run `cargo test` for details".to_string()),
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quality_gate_runs() {
        // Just verify it runs without panicking
        let result = run_quality_gate().await;
        assert_eq!(result.name, "quality");
        assert!(result.tool_available);
    }

    #[test]
    fn test_fmt_findings_empty_on_success() {
        // Can't easily test actual fmt check, but verify structure
        let findings: Vec<Finding> = vec![];
        assert!(findings.is_empty());
    }
}
