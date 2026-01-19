use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn run(purge: bool) -> Result<()> {
    let rc = crate::paths::env_rc_file();
    let aliases = crate::paths::env_aliases_file();

    // Capture repo path before we delete rc
    let mut repo: Option<PathBuf> = fs::read_to_string(&rc)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    if repo.is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            let guess = cwd.join("ralf-conf");
            if guess.exists() {
                repo = Some(guess);
            }
        }
    }

    let mut removed: Vec<String> = Vec::new();

    // Remove init stubs/snippets
    if let Some(home) = dirs::home_dir() {
        // fish conf.d stub
        let fish_stub = home
            .join(".config")
            .join("fish")
            .join("conf.d")
            .join("ralf.fish");
        if fish_stub.exists() {
            let _ = fs::remove_file(&fish_stub);
            removed.push(crate::paths::friendly(&fish_stub));
        }
        // tagged snippets in .bashrc and .zshrc
        let tag = "# >>> ralf init >>>";
        let close = "# <<< ralf init <<<";
        for rcfile in [home.join(".bashrc"), home.join(".zshrc")] {
            if let Ok(mut s) = std::fs::read_to_string(&rcfile) {
                if let (Some(start), Some(end)) = (s.find(tag), s.find(close)) {
                    let end_idx = end + close.len();
                    s.replace_range(start..end_idx, "");
                    let s2 = s.replace("\n\n\n", "\n\n");
                    std::fs::write(&rcfile, s2).ok();
                    removed.push(crate::paths::friendly(&rcfile));
                }
            }
        }
        // theme file
        let theme = home.join(".ralf_theme");
        if theme.exists() {
            let _ = fs::remove_file(&theme);
            removed.push(crate::paths::friendly(&theme));
        }
    }

    // Remove unified files
    let cfg = crate::paths::config_dir();
    for name in ["aliases.sh", "aliases.fish"] {
        let p = cfg.join(name);
        if p.exists() {
            let _ = fs::remove_file(&p);
            removed.push(crate::paths::friendly(&p));
        }
    }
    // Try to remove config dir if empty; purge removes regardless
    if purge {
        if cfg.exists() {
            let _ = fs::remove_dir_all(&cfg);
            removed.push(crate::paths::friendly(&cfg));
        }
    } else if cfg.exists()
        && fs::read_dir(&cfg)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false)
    {
        let _ = fs::remove_dir(&cfg);
        removed.push(crate::paths::friendly(&cfg));
    }

    // Remove rc and aliases files (back-compat locations)
    if rc.exists() {
        fs::remove_file(&rc).with_context(|| format!("failed removing {}", rc.display()))?;
        removed.push(crate::paths::friendly(&rc));
    }
    if aliases.exists() {
        fs::remove_file(&aliases)
            .with_context(|| format!("failed removing {}", aliases.display()))?;
        removed.push(crate::paths::friendly(&aliases));
    }

    // Remove repo-local markers
    if let Some(repo_path) = &repo {
        for f in [
            ".ralf_machine",
            "ralf.local.conf",
            ".alf_machine",
            "alf.local.conf",
        ] {
            let p = repo_path.join(f);
            if p.exists() {
                let _ = fs::remove_file(&p);
                removed.push(crate::paths::friendly(&p));
            }
        }
    }

    // Optional: purge the repo directory (only if it looks like ralf/alf)
    if purge {
        if let Some(repo_path) = repo {
            if repo_path.exists() && repo_path.is_dir() {
                let ok_name = repo_path
                    .file_name()
                    .map(|n| n == "ralf-conf" || n == "alf-conf")
                    .unwrap_or(false);
                if ok_name {
                    fs::remove_dir_all(&repo_path)
                        .with_context(|| format!("failed removing {}", repo_path.display()))?;
                    removed.push(crate::paths::friendly(&repo_path));
                } else {
                    eprintln!(
                        "Skipping repo deletion: not a ralf-conf/alf-conf dir ({})",
                        repo_path.display()
                    );
                }
            }
        }
    }

    if removed.is_empty() {
        println!("Nothing to clean.");
    } else {
        println!("Cleaned:");
        for r in removed {
            println!("- {}", r);
        }
        if !purge {
            println!("Tip: run 'ralf clean --purge' to also delete the connected repo directory.");
        }
    }
    Ok(())
}
