use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use which::which;

fn ensure_git() -> Result<()> {
    which("git").context("git executable not found in PATH")?;
    Ok(())
}

pub fn ensure_ralf_gitignore(repo_path: &Path) -> Result<()> {
    let gi = repo_path.join(".gitignore");
    let mut cur = fs::read_to_string(&gi).unwrap_or_default();
    let mut changed = false;
    for entry in [".ralf_machine", "alf.conf.save"] {
        if !cur.lines().any(|l| l.trim() == entry) {
            if !cur.is_empty() && !cur.ends_with('\n') {
                cur.push('\n');
            }
            cur.push_str(entry);
            cur.push('\n');
            changed = true;
        }
    }
    if changed {
        if let Some(parent) = gi.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&gi, cur)?;
    }
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
    ensure_ralf_gitignore(dest)?;
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

    ensure_ralf_gitignore(repo_path)?;

    // Untrack files that should be ignored (ignore failures)
    let _ = Command::new("git")
        .args(["rm", "--cached", "--ignore-unmatch", "--quiet", ".ralf_machine"])
        .current_dir(repo_path)
        .status();
    let _ = Command::new("git")
        .args(["rm", "--cached", "--ignore-unmatch", "--quiet", "alf.conf.save"])
        .current_dir(repo_path)
        .status();

    // Stage new files so they get committed
    let _ = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_path)
        .status();

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
