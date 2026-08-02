use std::path::PathBuf;

/// Install pre-push hook in .git/hooks/
pub fn install_hook() -> anyhow::Result<PathBuf> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let hook_path = hooks_dir.join("pre-push");

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path)?;
        if !content.contains("# Installed by gh-opp") {
            anyhow::bail!(
                "Hook exists at {} but wasn't installed by gh-opp. Remove manually.",
                hook_path.display()
            );
        }
    }

    let hook_content = r#"#!/bin/sh
# Installed by gh-opp
echo "Running pre-push security checks..."
gh-opp security
result=$?
if [ $result -ne 0 ]; then
    echo "Security checks failed. Push blocked."
    echo "Run 'gh-opp security' for details."
    exit 1
fi
"#;

    std::fs::write(&hook_path, hook_content)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    Ok(hook_path)
}

/// Remove pre-push hook if installed by us
pub fn remove_hook() -> anyhow::Result<PathBuf> {
    let git_dir = find_git_dir()?;
    let hook_path = git_dir.join("hooks").join("pre-push");

    if !hook_path.exists() {
        anyhow::bail!("No pre-push hook found at {}", hook_path.display());
    }

    let content = std::fs::read_to_string(&hook_path)?;
    if !content.contains("# Installed by gh-opp") {
        anyhow::bail!(
            "Hook at {} wasn't installed by gh-opp. Remove manually.",
            hook_path.display()
        );
    }

    std::fs::remove_file(&hook_path)?;
    Ok(hook_path)
}

fn find_git_dir() -> anyhow::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let git_dir = dir.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
        if !dir.pop() {
            anyhow::bail!("Not inside a git repository");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_git_dir() {
        // We're in a git repo, so this should work
        let result = find_git_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().join("HEAD").exists());
    }

    #[test]
    fn test_hook_content_format() {
        let hook = r#"#!/bin/sh
# Installed by gh-opp
echo "Running pre-push security checks..."
gh-opp security
result=$?
if [ $result -ne 0 ]; then
    echo "Security checks failed. Push blocked."
    echo "Run 'gh-opp security' for details."
    exit 1
fi
"#;
        assert!(hook.contains("# Installed by gh-opp"));
        assert!(hook.contains("gh-opp security"));
    }
}
