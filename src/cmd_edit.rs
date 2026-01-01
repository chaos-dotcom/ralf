use anyhow::{bail, Result};
use std::process::Command;

pub fn run(what: Option<String>) -> Result<()> {
    if matches!(what.as_deref(), Some("machine")) {
        let p = crate::paths::find_config_or_exit()?;
        let mid = crate::config_merge::resolve_machine_id(&p);
        let (machine_overlay, _) = crate::config_merge::overlay_paths(&p, &mid);
        if let Some(parent) = machine_overlay.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Create empty file if missing
        if !machine_overlay.exists() {
            std::fs::write(&machine_overlay, b"")?;
        }
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let status = std::process::Command::new(editor).arg(&machine_overlay).status()?;
        if !status.success() {
            anyhow::bail!("editor exited with non-zero status");
        }
        return Ok(());
    }

    let p = crate::paths::find_config_or_exit()?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor).arg(&p.config_file).status()?;
    if !status.success() {
        bail!("editor exited with non-zero status");
    }
    Ok(())
}
