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
            "Machine",
            "Info",
            "Help",
            "Clean",
            "Reset",
            "Upgrade",
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
            Some(8) => {
                // Machine: prompt to set, or show current if blank
                if let Some(mut name) = crate::tui::input("Enter machine name to set, Enter to show current, Esc to cancel")? {
                    name = name.trim().to_string();
                    if name.is_empty() {
                        run_child_capture(&["machine"])?;
                    } else {
                        run_child_capture(&["machine", &name])?;
                    }
                }
            }
            Some(9) => { run_child_capture(&["info"])?; }
            Some(10) => { run_child_capture(&["help"])?; }
            Some(11) => {
                // Clean: confirm optional purge
                let purge = crate::tui::confirm("Also delete the connected repo directory? [yN]")?;
                if purge {
                    run_child_capture(&["clean", "--purge"])?;
                } else {
                    run_child_capture(&["clean"])?;
                }
            }
            Some(12) => { run_child_capture(&["reset"])?; }
            Some(13) => { run_child_capture(&["upgrade"])?; }
            Some(14) => { run_theme_settings()?; }
            Some(15) | None => break,
            _ => {}
        }
    }
    Ok(())
}

fn run_child_capture(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let out = Command::new(exe)
        .args(args)
        .env("RALF_TUI", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let ok = out.status.success();
    let code = out.status.code().unwrap_or_default();

    let mut title = if ok { "Done" } else { "Error" }.to_string();

    let stdout_s = String::from_utf8_lossy(&out.stdout).trim_end().to_string();
    let stderr_s = String::from_utf8_lossy(&out.stderr).trim_end().to_string();

    let mut body = String::new();
    if ok {
        if stdout_s.is_empty() {
            body.push_str("Done.");
        } else {
            body.push_str(&stdout_s);
            if !body.ends_with('\n') { body.push('\n'); }
        }
        if !stderr_s.is_empty() {
            if !body.ends_with('\n') { body.push('\n'); }
            body.push_str("Warnings:\n");
            body.push_str(&stderr_s);
            if !body.ends_with('\n') { body.push('\n'); }
        }
    } else {
        title = format!("Error (code {})", code);
        if !stderr_s.is_empty() {
            body.push_str("Error:\n");
            body.push_str(&stderr_s);
            if !body.ends_with('\n') { body.push('\n'); }
        }
        if !stdout_s.is_empty() {
            if !body.ends_with('\n') { body.push('\n'); }
            body.push_str("Output:\n");
            body.push_str(&stdout_s);
            if !body.ends_with('\n') { body.push('\n'); }
        }
        if stderr_s.is_empty() && stdout_s.is_empty() {
            body.push_str(&format!("Command failed (code {}).", code));
        }
    }

    crate::tui::view_text(&format!("ralf — {}", title), &body)
}

fn run_child_passthrough(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let status = Command::new(exe).args(args).env("RALF_TUI", "1").status()?;
    let msg = if status.success() {
        "Done.".to_string()
    } else {
        format!("Failed (code {})", status.code().unwrap_or_default())
    };
    crate::tui::notify("ralf", &msg)
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
