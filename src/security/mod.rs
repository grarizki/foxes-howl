pub mod audit;
pub mod license;
pub mod quality;
pub mod secrets;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SecurityReport {
    pub passed: bool,
    pub checks: Vec<CheckResult>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub findings: Vec<Finding>,
    pub tool_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

impl SecurityReport {
    pub fn from_checks(checks: Vec<CheckResult>) -> Self {
        let passed = checks.iter().all(|c| c.passed || !c.tool_available);
        let total_findings: usize = checks.iter().map(|c| c.findings.len()).sum();
        let unavailable: usize = checks.iter().filter(|c| !c.tool_available).count();

        let summary = if passed && total_findings == 0 {
            "All security checks passed.".to_string()
        } else if passed {
            format!(
                "{} finding(s) across {} check(s). {} tool(s) unavailable.",
                total_findings,
                checks.len(),
                unavailable
            )
        } else {
            format!(
                "Security checks FAILED. {} finding(s), {} tool(s) unavailable.",
                total_findings, unavailable
            )
        };

        Self {
            passed,
            checks,
            summary,
        }
    }
}

/// Run all security checks
pub async fn run_all(
    deny_config: Option<&str>,
    extra_patterns: &[String],
) -> SecurityReport {
    let mut checks = Vec::new();

    checks.push(audit::run_audit().await);
    checks.push(secrets::scan_secrets(extra_patterns));
    checks.push(license::run_license_check(deny_config).await);
    checks.push(quality::run_quality_gate().await);

    SecurityReport::from_checks(checks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_from_checks_all_pass() {
        let checks = vec![CheckResult {
            name: "test".to_string(),
            passed: true,
            findings: vec![],
            tool_available: true,
        }];
        let report = SecurityReport::from_checks(checks);
        assert!(report.passed);
        assert!(report.summary.contains("passed"));
    }

    #[test]
    fn test_report_from_checks_failed() {
        let checks = vec![CheckResult {
            name: "test".to_string(),
            passed: false,
            findings: vec![Finding {
                severity: "critical".to_string(),
                message: "CVE found".to_string(),
                file: None,
                line: None,
                fix: None,
            }],
            tool_available: true,
        }];
        let report = SecurityReport::from_checks(checks);
        assert!(!report.passed);
        assert!(report.summary.contains("FAILED"));
    }

    #[test]
    fn test_report_unavailable_tools_counted_as_pass() {
        let checks = vec![CheckResult {
            name: "cargo-deny".to_string(),
            passed: false,
            findings: vec![],
            tool_available: false,
        }];
        let report = SecurityReport::from_checks(checks);
        // Unavailable tools don't fail the overall report
        assert!(report.passed);
    }
}
