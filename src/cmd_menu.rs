use anyhow::Result;
use std::process::Command;
use std::io::{self, Write};

pub fn run() -> Result<()> {
    loop {
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
                    run_child(&["connect", &repo, "--tui"])?;
                }
            }
            Some(1) => { run_child(&["download"])?; }
            Some(2) => { run_child(&["upload"])?; }
            Some(3) => { run_child(&["generate"])?; }
            Some(4) => { run_child(&["save"])?; }
            Some(5) => { run_child(&["edit"])?; }
            Some(6) => { run_child(&["edit", "machine"])?; }
            Some(7) => {
                if let Some(code) = crate::tui::input("Enter alias code (top-level), Esc to cancel")? {
                    let sub = crate::tui::input("Enter subcommand (optional), Enter for none, Esc to cancel")?;
                    if let Some(s) = sub.filter(|s| !s.is_empty()) {
                        run_child(&["which", &code, &s])?;
                    } else {
                        run_child(&["which", &code])?;
                    }
                }
            }
            Some(8) => { run_child(&["info"])?; }
            Some(9) => { run_child(&["help"])?; }
            Some(10) | None => break,
            _ => {}
        }
    }
    Ok(())
}

fn run_child(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let _status = Command::new(exe).args(args).status()?;
    // Let the user read output before returning to the menu
    print!("\nPress Enter to return to the main menu...");
    io::stdout().flush().ok();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    Ok(())
}
