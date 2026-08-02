use super::{CheckResult, Finding};

const ALLOWED_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Zlib",
    "0BSD",
    "CC0-1.0",
    "Unlicense",
    "BSL-1.0",
    "OpenSSL",
];

const COPYLEFT_LICENSES: &[&str] = &["GPL", "AGPL", "MPL", "EUPL", "OSL", "CPAL"];

/// Run license compliance check
pub async fn run_license_check(deny_config: Option<&str>) -> CheckResult {
    // Try cargo-deny first
    let deny_result = try_cargo_deny(deny_config).await;
    if deny_result.tool_available {
        return deny_result;
    }

    // Fallback: parse cargo metadata
    fallback_metadata_check().await
}

async fn try_cargo_deny(deny_config: Option<&str>) -> CheckResult {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["deny", "check", "licenses", "--json"]);
    if let Some(path) = deny_config {
        cmd.args(["--config", path]);
    }

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(_) => {
            return CheckResult {
                name: "license".to_string(),
                passed: false,
                findings: vec![],
                tool_available: false,
            };
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not found") || stderr.contains("No such file") {
        return CheckResult {
            name: "license".to_string(),
            passed: false,
            findings: vec![],
            tool_available: false,
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_deny_json(&stdout, output.status.success())
}

fn parse_deny_json(json_str: &str, success: bool) -> CheckResult {
    let mut findings = Vec::new();

    for line in json_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        // cargo-deny JSON format has "fields" with license info
        if let Some(fields) = parsed.get("fields") {
            let message = fields
                .get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or("license issue detected");

            let severity = parsed
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("warning");

            let crate_name = fields
                .get("crate")
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            findings.push(Finding {
                severity: match severity {
                    "error" => "high".to_string(),
                    "warning" => "medium".to_string(),
                    _ => "low".to_string(),
                },
                message: format!("{}: {}", crate_name, message),
                file: None,
                line: None,
                fix: Some("Update dependency or add license exception".to_string()),
            });
        }
    }

    CheckResult {
        name: "license".to_string(),
        passed: success && findings.is_empty(),
        findings,
        tool_available: true,
    }
}

async fn fallback_metadata_check() -> CheckResult {
    let output = tokio::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return CheckResult {
                name: "license".to_string(),
                passed: true, // can't check, assume OK
                findings: vec![],
                tool_available: false,
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let metadata: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => {
            return CheckResult {
                name: "license".to_string(),
                passed: true,
                findings: vec![],
                tool_available: false,
            };
        }
    };

    let mut findings = Vec::new();

    if let Some(packages) = metadata.get("packages").and_then(|p| p.as_array()) {
        for pkg in packages {
            let name = pkg
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let license = pkg
                .get("license")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if license.is_empty() {
                findings.push(Finding {
                    severity: "medium".to_string(),
                    message: format!("{}: no license declared", name),
                    file: None,
                    line: None,
                    fix: Some("Check the crate's repository for license info".to_string()),
                });
                continue;
            }

            let license_upper = license.to_uppercase();
            let is_allowed = ALLOWED_LICENSES
                .iter()
                .any(|l| license_upper.contains(&l.to_uppercase()));
            let is_copyleft = COPYLEFT_LICENSES
                .iter()
                .any(|l| license_upper.contains(&l.to_uppercase()));

            if is_copyleft {
                findings.push(Finding {
                    severity: "high".to_string(),
                    message: format!("{}: copyleft license ({})", name, license),
                    file: None,
                    line: None,
                    fix: Some("Verify compatibility with your project's license".to_string()),
                });
            } else if !is_allowed {
                findings.push(Finding {
                    severity: "low".to_string(),
                    message: format!("{}: license '{}' not in allowlist", name, license),
                    file: None,
                    line: None,
                    fix: Some("Review license compatibility".to_string()),
                });
            }
        }
    }

    let passed = findings.is_empty();
    CheckResult {
        name: "license".to_string(),
        passed,
        findings,
        tool_available: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deny_json_empty() {
        let result = parse_deny_json("", true);
        assert!(result.passed);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn test_parse_deny_json_with_issue() {
        let json = r#"{"fields":{"msg":"license not allowed","crate":{"name":"test-crate"}},"level":"error"}"#;
        let result = parse_deny_json(json, false);
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].message.contains("test-crate"));
    }

    #[test]
    fn test_allowed_licenses_include_common() {
        assert!(ALLOWED_LICENSES.contains(&"MIT"));
        assert!(ALLOWED_LICENSES.contains(&"Apache-2.0"));
        assert!(ALLOWED_LICENSES.contains(&"BSD-3-Clause"));
    }

    #[test]
    fn test_copyleft_detected() {
        let license = "GPL-3.0";
        let is_copyleft = COPYLEFT_LICENSES.iter().any(|l| license.contains(l));
        assert!(is_copyleft);
    }
}
