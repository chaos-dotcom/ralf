use crate::paths;
use anyhow::Result;

pub fn run() -> Result<()> {
    let rc = paths::env_rc_file();
    if !rc.exists() {
        println!("Cannot find {}", rc.display());
        println!("Please connect alf to a repository first");
        std::process::exit(1);
    }

    let p = paths::find_config_or_exit()?;
    println!("Pushing {} to repository", p.repo_path.display());
    crate::gitwrap::commit_all_and_push(&p.repo_path)?;
    Ok(())
}
