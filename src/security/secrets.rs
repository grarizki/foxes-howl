use super::{CheckResult, Finding};

/// Scan git-tracked files for secrets using regex patterns
pub fn scan_secrets(extra_patterns: &[String]) -> CheckResult {
    let files = match git_ls_files() {
        Ok(f) => f,
        Err(_) => {
            return CheckResult {
                name: "secrets".to_string(),
                passed: false,
                findings: vec![],
                tool_available: false,
            };
        }
    };

    let patterns = builtin_patterns();
    let mut findings = Vec::new();

    for file_path in &files {
        if should_skip(file_path) {
            continue;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue, // binary or unreadable
        };

        // Check for null bytes (binary file)
        if content.contains('\0') {
            continue;
        }

        for (line_num, line) in content.lines().enumerate() {
            for (name, regex) in &patterns {
                if regex.is_match(line) {
                    findings.push(Finding {
                        severity: "critical".to_string(),
                        message: format!("Potential {} detected", name),
                        file: Some(file_path.clone()),
                        line: Some(line_num as u32 + 1),
                        fix: Some("Remove and rotate this credential".to_string()),
                    });
                }
            }

            // Extra user-defined patterns
            for pattern in extra_patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(line) {
                        findings.push(Finding {
                            severity: "high".to_string(),
                            message: "Matches custom secret pattern".to_string(),
                            file: Some(file_path.clone()),
                            line: Some(line_num as u32 + 1),
                            fix: Some("Review and remove if secret".to_string()),
                        });
                    }
                }
            }
        }
    }

    // Deduplicate by file+line (multiple patterns can match same line)
    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.message.cmp(&b.message))
    });
    findings.dedup_by(|a, b| a.file == b.file && a.line == b.line);

    let passed = findings.is_empty();
    CheckResult {
        name: "secrets".to_string(),
        passed,
        findings,
        tool_available: true,
    }
}

fn git_ls_files() -> anyhow::Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["ls-files"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git ls-files failed");
    }
    let files = String::from_utf8_lossy(&output.stdout);
    Ok(files.lines().map(|l| l.to_string()).collect())
}

fn should_skip(path: &str) -> bool {
    let skip_prefixes = [".git/", "target/", "node_modules/", "vendor/"];
    skip_prefixes.iter().any(|p| path.starts_with(p))
}

use std::sync::LazyLock;

static BUILTIN_PATTERNS: LazyLock<Vec<(&'static str, regex::Regex)>> = LazyLock::new(|| {
    vec![
        (
            "AWS Access Key",
            regex::Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        ),
        (
            "AWS Secret Key",
            regex::Regex::new(r#"(?i)aws.{0,20}['"][0-9a-zA-Z/+]{40}['"]"#).unwrap(),
        ),
        (
            "GitHub Token",
            regex::Regex::new(r"gh[ps]_[a-zA-Z0-9]{36,}").unwrap(),
        ),
        (
            "Private Key",
            regex::Regex::new(r"-----BEGIN.{0,20}PRIVATE KEY-----").unwrap(),
        ),
        (
            "Generic Secret",
            regex::Regex::new(
                r#"(?i)(api[_-]?key|token|secret|password).{0,10}['"][0-9a-zA-Z]{32,}['"]"#,
            )
            .unwrap(),
        ),
    ]
});

fn builtin_patterns() -> Vec<(&'static str, regex::Regex)> {
    BUILTIN_PATTERNS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip() {
        assert!(should_skip(".git/config"));
        assert!(should_skip("target/debug/build"));
        assert!(should_skip("node_modules/package/index.js"));
        assert!(!should_skip("src/main.rs"));
        assert!(!should_skip("Cargo.toml"));
    }

    #[test]
    fn test_aws_key_pattern() {
        let patterns = builtin_patterns();
        let aws_pattern = patterns
            .iter()
            .find(|(n, _)| *n == "AWS Access Key")
            .unwrap();
        assert!(aws_pattern.1.is_match("AKIAIOSFODNN7EXAMPLE"));
        assert!(!aws_pattern.1.is_match("normal text"));
    }

    #[test]
    fn test_github_token_pattern() {
        let patterns = builtin_patterns();
        let gh_pattern = patterns.iter().find(|(n, _)| *n == "GitHub Token").unwrap();
        assert!(gh_pattern
            .1
            .is_match("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234"));
        assert!(!gh_pattern.1.is_match("ghp_short"));
    }

    #[test]
    fn test_private_key_pattern() {
        let patterns = builtin_patterns();
        let pk_pattern = patterns.iter().find(|(n, _)| *n == "Private Key").unwrap();
        assert!(pk_pattern.1.is_match("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(pk_pattern.1.is_match("-----BEGIN EC PRIVATE KEY-----"));
        assert!(!pk_pattern.1.is_match("-----BEGIN CERTIFICATE-----"));
    }

    #[test]
    fn test_generic_secret_pattern() {
        let patterns = builtin_patterns();
        let gen_pattern = patterns
            .iter()
            .find(|(n, _)| *n == "Generic Secret")
            .unwrap();
        assert!(gen_pattern
            .1
            .is_match(r#"api_key = "abcdefghijklmnopqrstuvwxyz12345678""#));
        assert!(gen_pattern
            .1
            .is_match(r#"TOKEN="ABCDEFGHIJKLMNOP1234567890abcdef""#));
    }
}
