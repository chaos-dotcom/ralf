use crate::cli::ConnectArgs;
use crate::gitwrap;
use crate::paths;
use crate::tui;
use anyhow::Result;

pub fn run(args: ConnectArgs) -> Result<()> {
    let input = args.repo;
    let only_user = !input.contains(':') && !input.contains('/');
    let mut alt_repo_url: Option<String> = None;
    let mut partial_github = false;
    let mut repo_url: String;

    if input.contains(':') {
        repo_url = input; // full URL
    } else if input.contains('/') {
        partial_github = true;
        repo_url = format!("{}.git", input);
    } else {
        partial_github = true;
        repo_url = format!("{}/ralf-conf.git", input);
        alt_repo_url = Some(format!("{}/alf-conf.git", input));
    }

    // Destination and rc paths
    let cwd = std::env::current_dir()?;
    let dir_ralf = cwd.join("ralf-conf");
    let dir_alf = cwd.join("alf-conf");
    let mut dest = if dir_ralf.exists() {
        dir_ralf
    } else if dir_alf.exists() {
        dir_alf
    } else {
        let name = repo_url
            .rsplit('/')
            .next()
            .unwrap_or("ralf-conf.git")
            .trim_end_matches(".git");
        cwd.join(name)
    };
    let rc_file = paths::env_rc_file();

    // Non-interactive flags
    let mut accepted: bool;
    if args.ssh {
        if partial_github {
            repo_url = format!("git@github.com:{}", repo_url);
        }
        if let Some(alt) = alt_repo_url.take() {
            alt_repo_url = Some(format!("git@github.com:{}", alt));
        }
        println!("Connecting to {}", repo_url);
        accepted = true;
    } else if args.https {
        if partial_github {
            repo_url = format!("https://github.com/{}", repo_url);
        }
        if let Some(alt) = alt_repo_url.take() {
            alt_repo_url = Some(format!("https://github.com/{}", alt));
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
            println!(
                "     to  ./{}",
                dest.file_name().unwrap().to_string_lossy()
            );
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
                    if let Some(alt) = alt_repo_url.take() {
                        alt_repo_url = Some(format!("git@github.com:{}", alt));
                    }
                    accepted = true;
                    println!();
                    println!("Using {}", repo_url);
                }
                tui::ConnectChoice::Https => {
                    repo_url = format!("https://github.com/{}", repo_url);
                    if let Some(alt) = alt_repo_url.take() {
                        alt_repo_url = Some(format!("https://github.com/{}", alt));
                    }
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
        let primary = gitwrap::clone(&repo_url, &dest);
        if primary.is_err() && only_user {
            if let Some(alt) = alt_repo_url.clone() {
                eprintln!("Primary clone failed, retrying with {}", alt);
                // If the alt name differs (ralf-conf vs alf-conf), adjust dest
                let alt_name = alt
                    .rsplit('/')
                    .next()
                    .unwrap_or("alf-conf.git")
                    .trim_end_matches(".git");
                let alt_dest = cwd.join(alt_name);
                if dest != alt_dest && !alt_dest.exists() {
                    dest = alt_dest;
                }
                gitwrap::clone(&alt, &dest)?;
            } else {
                primary?;
            }
        } else {
            primary?;
        }
    }

    // If we ended up with alf-conf, migrate folder name to ralf-conf
    if dest.file_name().map(|n| n == "alf-conf").unwrap_or(false) {
        let new_dest = cwd.join("ralf-conf");
        if !new_dest.exists() {
            std::fs::rename(&dest, &new_dest)?;
            dest = new_dest;
        }
    }

    // Write "$PWD/ralf-conf" to rc file (overwrite)
    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let abs = dest.canonicalize().unwrap_or(dest.clone());
    std::fs::write(&rc_file, abs.to_string_lossy().as_bytes())?;
    println!("Storing location in {}", rc_file.display());

    // Regenerate aliases
    crate::cmd_save::run()?;
    Ok(())
}
