use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;
    let content = crate::generator::generate_config()?;

    println!("Saving to {}", p.aliases_file.display());
    if let Some(parent) = p.aliases_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&p.aliases_file, content)?;

    println!("To apply the new aliases to the current session, run:");
    println!("$ source {}", crate::paths::friendly(&p.aliases_file));
    Ok(())
}
