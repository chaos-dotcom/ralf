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
    if let Ok(s) = std::env::var("ralf_RC_FILE") {
        PathBuf::from(shellexpand::tilde(&s).into_owned())
    } else {
        home_dir().unwrap().join(".ralfrc")
    }
}

pub fn env_aliases_file() -> PathBuf {
    if let Ok(s) = std::env::var("ralf_ALIASES_FILE") {
        PathBuf::from(shellexpand::tilde(&s).into_owned())
    } else {
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

    // Bash parity:
    // repo_path defaults to "$PWD/ralf-conf"
    // config_file defaults to "ralf.conf" (in current dir)
    let mut repo_path = cwd.join("ralf-conf");
    let mut config_file = cwd.join("ralf.conf");

    if rc_file.exists() {
        let content = fs::read_to_string(&rc_file)?.trim().to_string();
        if !content.is_empty() {
            repo_path = PathBuf::from(content);
            config_file = repo_path.join("ralf.conf");
        }
    }

    if !config_file.exists() {
        // Match bash-src/lib/find_config.sh exact text
        print!("ERROR: Cannot find config file\n\n");
        println!("You should either:");
        println!("- Run this command in a folder with 'ralf.conf' file, or");
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
