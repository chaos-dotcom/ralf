use anyhow::Result;
use std::process::Command;
use std::io::{self, Write};

pub fn run() -> Result<()> {
    let items = [
        "Connect",
        "Download",
        "Upload",
        "Generate",
        "Save",
        "Edit base config",
        "Edit machine config",
        "Which alias",
        "Info",
        "Help",
        "Exit",
    ];
    let sel = crate::tui::select("ralf — choose an action", &items)?;
    match sel {
        Some(0) => {
            if let Some(repo) = crate::tui::input("Enter repository (user[/repo] or full URL), Esc to cancel")? {
                let args = crate::cli::ConnectArgs { repo, ssh: false, https: false, yes: false, tui: true };
                crate::cmd_connect::run(args)?;
            }
        }
        Some(1) => { crate::cmd_download::run()?; }
        Some(2) => { crate::cmd_upload::run()?; }
        Some(3) => { crate::cmd_generate::run()?; }
        Some(4) => { crate::cmd_save::run()?; }
        Some(5) => { crate::cmd_edit::run(None)?; }
        Some(6) => { crate::cmd_edit::run(Some("machine".to_string()))?; }
        Some(7) => {
            if let Some(code) = crate::tui::input("Enter alias code (top-level), Esc to cancel")? {
                let sub = crate::tui::input("Enter subcommand (optional), Enter for none, Esc to cancel")?;
                let sub = sub.filter(|s| !s.is_empty());
                crate::cmd_which::run(code, sub)?;
            }
        }
        Some(8) => { crate::cmd_info::run()?; }
        Some(9) => { crate::cmd_help::run(None)?; }
        _ => {}
    }
    Ok(())
}
