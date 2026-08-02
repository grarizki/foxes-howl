use super::{CheckResult, Finding};

/// Run cargo audit and parse results
pub async fn run_audit() -> CheckResult {
    let output = tokio::process::Command::new("cargo")
        .args(["audit", "--json"])
        .output()
        .await;

    let output = match output {
        Ok(o) => o,
        Err(_) => {
            return CheckResult {
                name: "cargo-audit".to_string(),
                passed: false,
                findings: vec![],
                tool_available: false,
            };
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file") {
            return CheckResult {
                name: "cargo-audit".to_string(),
                passed: false,
                findings: vec![],
                tool_available: false,
            };
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_audit_json(&stdout)
}

fn parse_audit_json(json_str: &str) -> CheckResult {
    let mut findings = Vec::new();

    // cargo audit --json outputs one JSON object per line (NDJSON-like)
    for line in json_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        // Look for vulnerability records
        if let Some(vulns) = parsed.get("vulnerabilities").and_then(|v| v.get("list")) {
            if let Some(list) = vulns.as_array() {
                for vuln in list {
                    let id = vuln
                        .get("advisory")
                        .and_then(|a| a.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let title = vuln
                        .get("advisory")
                        .and_then(|a| a.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown vulnerability");
                    let severity = vuln
                        .get("advisory")
                        .and_then(|a| a.get("severity"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("medium");

                    let package = vuln
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    findings.push(Finding {
                        severity: severity.to_lowercase(),
                        message: format!("[{}] {} in {}", id, title, package),
                        file: None,
                        line: None,
                        fix: vuln
                            .get("patched")
                            .and_then(|p| p.as_array())
                            .and_then(|p| p.first())
                            .and_then(|v| v.as_str())
                            .map(|v| format!("Update to {}", v)),
                    });
                }
            }
        }
    }

    let passed = findings.is_empty();
    CheckResult {
        name: "cargo-audit".to_string(),
        passed,
        findings,
        tool_available: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_json() {
        let result = parse_audit_json("");
        assert!(result.passed);
        assert!(result.findings.is_empty());
        assert!(result.tool_available);
    }

    #[test]
    fn test_parse_no_vulnerabilities() {
        let json = r#"{"vulnerabilities":{"list":[]}}"#;
        let result = parse_audit_json(json);
        assert!(result.passed);
    }

    #[test]
    fn test_parse_with_vulnerability() {
        let json = r#"{"vulnerabilities":{"list":[{"advisory":{"id":"RUSTSEC-2021-0001","title":"Test vuln","severity":"high"},"package":{"name":"test-crate"},"patched":[">=1.2.3"]}]}}"#;
        let result = parse_audit_json(json);
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].severity, "high");
        assert!(result.findings[0].message.contains("RUSTSEC-2021-0001"));
        assert!(result.findings[0].fix.as_ref().unwrap().contains("1.2.3"));
    }
}
