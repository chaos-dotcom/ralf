use crate::paths;
use anyhow::Result;

pub fn run() -> Result<()> {
    let rc = paths::env_rc_file();
    if !rc.exists() {
        println!("Cannot find {}", rc.display());
        println!("Please connect ralf to a repository first");
        std::process::exit(1);
    }

    let p = paths::find_config_or_exit()?;
    println!("Pulling from repository to {}", p.repo_path.display());
    crate::gitwrap::pull(&p.repo_path)?;
    crate::gitwrap::ensure_ralf_gitignore(&p.repo_path)?;
    crate::cmd_save::run()?;
    Ok(())
}
