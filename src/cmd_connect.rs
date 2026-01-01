use crate::cli::ConnectArgs;
use crate::gitwrap;
use crate::paths;
use crate::tui;
use anyhow::Result;
use std::path::PathBuf;

pub fn run(args: ConnectArgs) -> Result<()> {
    let input = args.repo;
    let mut partial_github = false;
    let mut repo_url = String::new();

    if input.contains(':') {
        repo_url = input; // full URL
    } else if input.contains('/') {
        partial_github = true;
        repo_url = format!("{}.git", input);
    } else {
        partial_github = true;
        repo_url = format!("{}/ralf-conf.git", input);
    }

    // Destination and rc paths
    let cwd = std::env::current_dir()?;
    let dest = cwd.join("ralf-conf");
    let rc_file = paths::env_rc_file();

    // Non-interactive flags
    let mut accepted = false;
    if args.ssh {
        if partial_github {
            repo_url = format!("git@github.com:{}", repo_url);
        }
        println!("Connecting to {}", repo_url);
        accepted = true;
    } else if args.https {
        if partial_github {
            repo_url = format!("https://github.com/{}", repo_url);
        }
        println!("Connecting to {}", repo_url);
        accepted = true;
    } else if args.yes {
        if partial_github {
            println!("Error: Cannot determine the full URL for the repository");
            println!("To connect to GitHub use --ssh or --https");
            println!("To connect to another repository, provide the full URL");
            std::process::exit(1);
        }
        println!("Connecting to {}", repo_url);
        accepted = true;
    } else {
        // Interactive summary (bash parity)
        println!("This operation will:");
        println!();
        if !dest.exists() {
            println!("  clone  {}", repo_url);
            println!("     to  ./ralf-conf");
            println!();
        }
        println!("  write  {}/ralf-conf", cwd.display());
        println!("     to  {}", rc_file.display());
        println!();

        if partial_github {
            println!("Would you like to connect to GitHub via SSH or HTTPS?");
            println!();
            match tui::choose_github_protocol()? {
                tui::ConnectChoice::Ssh => {
                    repo_url = format!("git@github.com:{}", repo_url);
                    accepted = true;
                    println!();
                    println!("Using {}", repo_url);
                }
                tui::ConnectChoice::Https => {
                    repo_url = format!("https://github.com/{}", repo_url);
                    accepted = true;
                    println!();
                    println!("Using {}", repo_url);
                }
                tui::ConnectChoice::Abort => {
                    println!("Aborting");
                    return Ok(());
                }
            }
        } else if !tui::confirm("Continue? [yN]")? {
            println!("Aborting");
            return Ok(());
        } else {
            accepted = true;
        }
    }

    if !accepted {
        println!("Aborting");
        return Ok(());
    }

    if dest.exists() {
        println!("Skipping clone, directory already exists");
    } else {
        gitwrap::clone(&repo_url, &dest)?;
    }

    // Write "$PWD/ralf-conf" to rc file (overwrite)
    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = format!("{}/ralf-conf", cwd.display());
    std::fs::write(&rc_file, contents)?;
    println!("Storing location in {}", rc_file.display());

    // Regenerate aliases
    crate::cmd_save::run()?;
    Ok(())
}
