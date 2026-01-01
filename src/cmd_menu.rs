use anyhow::Result;
use std::process::Command;
use std::process::Stdio;

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
            "Theme",
            "Exit",
        ];
        let sel = crate::tui::select("ralf — choose an action", &items)?;
        match sel {
            Some(0) => {
                if let Some(repo) = crate::tui::input("Enter repository (user[/repo] or full URL), Esc to cancel")? {
                    run_child_passthrough(&["connect", &repo, "--tui"])?;
                }
            }
            Some(1) => { run_child_capture(&["download"])?; }
            Some(2) => { run_child_capture(&["upload"])?; }
            Some(3) => { run_child_capture(&["generate"])?; }
            Some(4) => { run_child_capture(&["save"])?; }
            Some(5) => { run_child_passthrough(&["edit"])?; }
            Some(6) => { run_child_passthrough(&["edit", "machine"])?; }
            Some(7) => {
                if let Some(code) = crate::tui::input("Enter alias code (top-level), Esc to cancel")? {
                    let sub = crate::tui::input("Enter subcommand (optional), Enter for none, Esc to cancel")?;
                    if let Some(s) = sub.filter(|s| !s.is_empty()) {
                        run_child_capture(&["which", &code, &s])?;
                    } else {
                        run_child_capture(&["which", &code])?;
                    }
                }
            }
            Some(8) => { run_child_capture(&["info"])?; }
            Some(9) => { run_child_capture(&["help"])?; }
            Some(10) => { run_theme_settings()?; }
            Some(11) | None => break,
            _ => {}
        }
    }
    Ok(())
}

fn run_child_capture(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe()?;
    // Capture both stdout and stderr
    let out = Command::new(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let mut body = String::new();
    body.push_str(&format!("$ ralf {}\n\n", args.join(" ")));
    if !out.stdout.is_empty() {
        body.push_str(&String::from_utf8_lossy(&out.stdout));
        if !body.ends_with('\n') { body.push('\n'); }
    }
    if !out.stderr.is_empty() {
        if !body.ends_with('\n') { body.push('\n'); }
        body.push_str("--- stderr ---\n");
        body.push_str(&String::from_utf8_lossy(&out.stderr));
        if !body.ends_with('\n') { body.push('\n'); }
    }
    let status = out.status.code().unwrap_or_default();
    body.push_str(&format!("\n(exit status: {})\n", status));
    crate::tui::view_text("ralf output", &body)
}

fn run_child_passthrough(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let status = Command::new(exe).args(args).status()?;
    let msg = if let Some(code) = status.code() {
        format!("Command exited with status {}", code)
    } else {
        "Command terminated".to_string()
    };
    crate::tui::notify("Completed", &msg)
}

fn run_theme_settings() -> Result<()> {
    let options = crate::tui::theme_options();
    if let Some(idx) = crate::tui::select("Choose a background theme", options)? {
        let name = options[idx];
        if name != "Cancel" {
            // map display to slug
            let slug = match name {
                "Trans" => "trans",
                "Lesbian" => "lesbian",
                "Bisexual" => "bisexual",
                "Non-binary" => "non-binary",
                "Intersex" => "intersex",
                "Progress" => "progress",
                _ => "trans",
            };
            crate::tui::set_theme_by_name(slug)?;
            // The next menu render will show the new background
            crate::tui::notify("Theme updated", &format!("Theme set to {}", name))?;
        }
    }
    Ok(())
}
