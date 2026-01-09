use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let rc = crate::paths::env_rc_file();
    let aliases = crate::paths::env_aliases_file();

    let mut removed: Vec<String> = Vec::new();

    if rc.exists() {
        fs::remove_file(&rc).with_context(|| format!("failed removing {}", rc.display()))?;
        removed.push(crate::paths::friendly(&rc));
    }

    if aliases.exists() {
        fs::remove_file(&aliases).with_context(|| format!("failed removing {}", aliases.display()))?;
        removed.push(crate::paths::friendly(&aliases));
    }

    // Try to find repo path from rc content (if it existed), else guess ./ralf-conf
    let mut repo: Option<PathBuf> = None;
    if let Ok(content) = fs::read_to_string(&rc) {
        let s = content.trim();
        if !s.is_empty() {
            repo = Some(PathBuf::from(s));
        }
    }
    if repo.is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            let guess = cwd.join("ralf-conf");
            if guess.exists() {
                repo = Some(guess);
            }
        }
    }

    if let Some(repo_path) = repo {
        for f in [".ralf_machine", "ralf.local.conf", ".alf_machine", "alf.local.conf"] {
            let p = repo_path.join(f);
            if p.exists() {
                let _ = fs::remove_file(&p);
                removed.push(crate::paths::friendly(&p));
            }
        }
    }

    if removed.is_empty() {
        println!("Nothing to reset.");
    } else {
        println!("Removed:");
        for r in removed {
            println!("- {}", r);
        }
    }

    Ok(())
}
