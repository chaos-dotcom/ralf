use anyhow::{bail, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor).arg(&p.config_file).status()?;
    if !status.success() {
        bail!("editor exited with non-zero status");
    }
    Ok(())
}
