use crate::config_merge;
use crate::paths;
use anyhow::Result;
use std::fs;

pub fn run(name: Option<String>) -> Result<()> {
    let p = paths::find_config_or_exit()?;
    let marker = p.repo_path.join(".ralf_machine");

    if let Some(n) = name {
        let n = n.trim();
        if let Some(dir) = marker.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&marker, n.as_bytes())?;

        let (mp, lp) = config_merge::overlay_paths(&p, n);
        println!("Machine set to '{}'", n);
        println!("Machine overlay: {}", mp.display());
        println!("Local overlay:   {}", lp.display());
        println!("Tip: run 'ralf edit machine' to edit the machine overlay.");
    } else {
        let mid = config_merge::resolve_machine_id(&p);
        let (mp, lp) = config_merge::overlay_paths(&p, &mid);
        println!("Current machine: {}", mid);
        println!(
            "Machine overlay: {} ({})",
            mp.display(),
            if mp.exists() { "exists" } else { "missing" }
        );
        println!(
            "Local overlay:   {} ({})",
            lp.display(),
            if lp.exists() { "exists" } else { "missing" }
        );
    }
    Ok(())
}
