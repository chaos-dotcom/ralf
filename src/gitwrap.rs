use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use which::which;

fn ensure_git() -> Result<()> {
    which("git").context("git executable not found in PATH")?;
    Ok(())
}

pub fn clone(url: &str, dest: &Path) -> Result<()> {
    ensure_git()?;
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(dest)
        .status()
        .context("failed to spawn git clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }
    Ok(())
}

pub fn pull(repo_path: &Path) -> Result<()> {
    ensure_git()?;
    let status = Command::new("git")
        .arg("pull")
        .current_dir(repo_path)
        .status()
        .context("failed to spawn git pull")?;
    if !status.success() {
        anyhow::bail!("git pull failed");
    }
    Ok(())
}

pub fn commit_all_and_push(repo_path: &Path) -> Result<()> {
    ensure_git()?;

    // Commit tracked changes; ignore non-zero (nothing to commit)
    let _ = Command::new("git")
        .args(["commit", "-am", "automatic push"])
        .current_dir(repo_path)
        .status();

    let status = Command::new("git")
        .arg("push")
        .current_dir(repo_path)
        .status()
        .context("failed to spawn git push")?;
    if !status.success() {
        anyhow::bail!("git push failed");
    }
    Ok(())
}
