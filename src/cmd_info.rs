use anyhow::Result;
use std::fs;
use std::process::Command;
use which::which;

pub fn run() -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;

    // Git remote
    let remote = if p.repo_path.join(".git").exists() {
        let out = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .current_dir(&p.repo_path)
            .output()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            "unset".into()
        } else {
            s
        }
    } else {
        "unset".into()
    };

    // ralfrc content / aliases status
    let ralfrc_content = if p.rc_file.exists() {
        format!("exists with '{}'", fs::read_to_string(&p.rc_file)?.trim())
    } else {
        "does not exist".into()
    };
    let aliases_status = if p.aliases_file.exists() {
        "exists"
    } else {
        "does not exist"
    };

    // Executable path (command -v ralf)
    let exe = which("ralf")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".into())
        });

    // Print (match bash spacing/text)
    println!("Executable:");
    println!("  path:              {}", exe);
    println!();

    println!("Environment:");
    println!(
        "  ralf_RC_FILE:       {}",
        std::env::var("ralf_RC_FILE").unwrap_or_else(|_| "unset".into())
    );
    println!(
        "  ralf_ALIASES_FILE:  {}",
        std::env::var("ralf_ALIASES_FILE").unwrap_or_else(|_| "unset".into())
    );
    println!(
        "  ALF_RC_FILE:       {}",
        std::env::var("ALF_RC_FILE").unwrap_or_else(|_| "unset".into())
    );
    println!(
        "  ALF_ALIASES_FILE:  {}",
        std::env::var("ALF_ALIASES_FILE").unwrap_or_else(|_| "unset".into())
    );
    println!(
        "  ralf_MACHINE:      {}",
        std::env::var("ralf_MACHINE").unwrap_or_else(|_| "unset".into())
    );
    println!();

    println!("Paths:");
    println!("  ralfrc path:        {}", p.rc_file.display());
    println!("  aliases path:      {}", p.aliases_file.display());
    println!("  repo path:         {}", p.repo_path.display());
    println!("  config path:       {}", p.config_file.display());
    println!();

    println!("Files:");
    println!("  ralfrc:             {}", ralfrc_content);
    println!("  aliases:           {}", aliases_status);
    println!();

    let mid = crate::config_merge::resolve_machine_id(&p);
    let (mp, lp) = crate::config_merge::overlay_paths(&p, &mid);
    println!("Machine:");
    println!("  id:               {}", mid);
    println!(
        "  machine overlay:  {} ({})",
        mp.display(),
        if mp.exists() { "exists" } else { "missing" }
    );
    println!(
        "  local overlay:    {} ({})",
        lp.display(),
        if lp.exists() { "exists" } else { "missing" }
    );

    println!("GitHub:");
    println!("  remote:            {}", remote);

    Ok(())
}
