use dirs::home_dir;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Paths {
    pub rc_file: PathBuf,
    pub aliases_file: PathBuf,
    pub repo_path: PathBuf,
    pub config_file: PathBuf,
}

pub fn env_rc_file() -> PathBuf {
    if let Ok(s) = std::env::var("RALF_RC_FILE")
        .or_else(|_| std::env::var("ralf_RC_FILE"))
        .or_else(|_| std::env::var("ALF_RC_FILE"))
    {
        PathBuf::from(shellexpand::tilde(&s).into_owned())
    } else {
        let home = home_dir().unwrap();
        let ralf = home.join(".ralfrc");
        let alf = home.join(".alfrc");
        if ralf.exists() {
            ralf
        } else if alf.exists() {
            alf
        } else {
            ralf
        }
    }
}

pub fn env_aliases_file() -> PathBuf {
    if let Ok(s) = std::env::var("RALF_ALIASES_FILE")
        .or_else(|_| std::env::var("ralf_ALIASES_FILE"))
        .or_else(|_| std::env::var("ALF_ALIASES_FILE"))
    {
        PathBuf::from(shellexpand::tilde(&s).into_owned())
    } else {
        let is_fish = std::env::var("FISH_VERSION").is_ok()
            || std::env::var("SHELL")
                .ok()
                .map(|s| s.ends_with("fish") || s.contains("/fish"))
                .unwrap_or(false);
        if is_fish {
            return home_dir().unwrap().join(".config").join("fish").join("conf.d").join("ralf.fish");
        }
        let is_zsh = std::env::var("ZSH_VERSION").is_ok()
            || std::env::var("SHELL")
                .ok()
                .map(|s| s.ends_with("zsh") || s.contains("/zsh"))
                .unwrap_or(false);
        if is_zsh {
            home_dir().unwrap().join(".zsh_aliases")
        } else {
            home_dir().unwrap().join(".bash_aliases")
        }
    }
}

pub fn find_config_or_exit() -> anyhow::Result<Paths> {
    let rc_file = env_rc_file();
    let aliases_file = env_aliases_file();
    let cwd = std::env::current_dir()?;
    let rc_env_set = std::env::var("RALF_RC_FILE").is_ok()
        || std::env::var("ralf_RC_FILE").is_ok()
        || std::env::var("ALF_RC_FILE").is_ok();

    // Bash parity:
    // repo_path defaults to "$PWD/ralf-conf"
    // config_file defaults to "ralf.conf" (in current dir)
    let mut repo_path = cwd.join("ralf-conf");
    let mut config_file = cwd.join("ralf.conf");

    if rc_env_set && rc_file.exists() {
        let content = fs::read_to_string(&rc_file)?.trim().to_string();
        if !content.is_empty() {
            repo_path = PathBuf::from(content);
            config_file = repo_path.join("ralf.conf");
        }
    }

    // Legacy detection and migration to ralf
    let mut migrated = false;

    // If repo dir is alf-conf, rename to ralf-conf (unless ralf-conf already exists)
    if repo_path
        .file_name()
        .map(|n| n == "alf-conf")
        .unwrap_or(false)
    {
        let new_repo = repo_path.with_file_name("ralf-conf");
        if !new_repo.exists() {
            if fs::rename(&repo_path, &new_repo).is_ok() {
                repo_path = new_repo;
                migrated = true;
            }
        }
    }

    // If config file missing, look for alf.conf and migrate it to ralf.conf
    if !config_file.exists() {
        let alf_in_repo = repo_path.join("alf.conf");
        if alf_in_repo.exists() {
            let new_cf = repo_path.join("ralf.conf");
            if !new_cf.exists() {
                let _ = fs::rename(&alf_in_repo, &new_cf);
            }
            config_file = new_cf;
            migrated = true;
        } else {
            // Search common locations in CWD and typical repo folders
            let ralf_cwd = cwd.join("ralf.conf");
            let alf_cwd = cwd.join("alf.conf");
            if ralf_cwd.exists() {
                config_file = ralf_cwd;
                repo_path = cwd.clone();
            } else if alf_cwd.exists() {
                let new_cf = cwd.join("ralf.conf");
                if !new_cf.exists() {
                    let _ = fs::rename(&alf_cwd, &new_cf);
                }
                config_file = new_cf;
                repo_path = cwd.clone();
                migrated = true;
            } else {
                for d in ["ralf-conf", "alf-conf"] {
                    let dir = cwd.join(d);
                    let cf_ralf = dir.join("ralf.conf");
                    let cf_alf = dir.join("alf.conf");
                    if cf_ralf.exists() {
                        repo_path = dir.clone();
                        config_file = cf_ralf;
                        break;
                    } else if cf_alf.exists() {
                        let new_cf = dir.join("ralf.conf");
                        if !new_cf.exists() {
                            let _ = fs::rename(&cf_alf, &new_cf);
                        }
                        if d == "alf-conf" {
                            let new_dir = cwd.join("ralf-conf");
                            if !new_dir.exists() {
                                let _ = fs::rename(&dir, &new_dir);
                                repo_path = new_dir;
                            } else {
                                repo_path = dir;
                            }
                        } else {
                            repo_path = dir;
                        }
                        config_file = new_cf;
                        migrated = true;
                        break;
                    }
                }
            }
        }
    }

    // Migrate local overlay and machine marker if present
    let alf_local = repo_path.join("alf.local.conf");
    let ralf_local = repo_path.join("ralf.local.conf");
    if alf_local.exists() && !ralf_local.exists() {
        let _ = fs::rename(&alf_local, &ralf_local);
        migrated = true;
    }
    let alf_machine = repo_path.join(".alf_machine");
    let ralf_machine = repo_path.join(".ralf_machine");
    if alf_machine.exists() && !ralf_machine.exists() {
        let _ = fs::rename(&alf_machine, &ralf_machine);
        migrated = true;
    }

    // If rc file is .alfrc, migrate to .ralfrc (if not already present)
    if rc_env_set && rc_file
        .file_name()
        .map(|n| n == ".alfrc")
        .unwrap_or(false)
    {
        let new_rc = rc_file.with_file_name(".ralfrc");
        let rc_file = if !new_rc.exists() && fs::rename(&rc_file, &new_rc).is_ok() {
            new_rc
        } else {
            new_rc
        };
        // Write updated repo_path to the (new) rc file
        if let Some(parent) = rc_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&rc_file, format!("{}\n", repo_path.to_string_lossy()))?;
        // Rebuild Paths locals to continue with the migrated rc path
        let aliases_file = aliases_file;
        return Ok(Paths {
            rc_file,
            aliases_file,
            repo_path,
            config_file,
        });
    }

    // If anything migrated, ensure rc file content points to the new repo_path
    if migrated && rc_env_set && rc_file.exists() {
        if let Some(parent) = rc_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&rc_file, format!("{}\n", repo_path.to_string_lossy()))?;
    }

    if !config_file.exists() {
        // Match bash-src/lib/find_config.sh exact text
        print!("ERROR: Cannot find config file\n\n");
        println!("You should either:");
        println!("- Run this command in a folder with 'ralf.conf' or 'alf.conf' file, or");
        println!("- Run 'ralf connect' to properly connect to a remote config");
        std::process::exit(1);
    }

    Ok(Paths {
        rc_file,
        aliases_file,
        repo_path,
        config_file,
    })
}

pub fn friendly(path: &Path) -> String {
    let p = path.to_string_lossy().into_owned();
    if let Some(home) = home_dir() {
        let h = home.to_string_lossy().into_owned();
        if p.starts_with(&h) {
            return p.replacen(&h, "~", 1);
        }
    }
    p
}
