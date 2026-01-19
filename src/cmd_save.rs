use anyhow::Result;

pub fn run() -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;
    let text = crate::config_merge::load_and_merge(&p)?;
    let cfg_dir = crate::paths::config_dir();
    std::fs::create_dir_all(&cfg_dir)?;

    let sh_target = cfg_dir.join("aliases.sh");
    let fish_target = cfg_dir.join("aliases.fish");
    let in_tui = std::env::var("RALF_TUI").is_ok();

    // Helpers
    let esc = |s: &str| {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
    };
    let rc_q = esc(&p.rc_file.to_string_lossy());
    let machine = crate::config_merge::resolve_machine_id(&p);
    let mid_q = esc(&machine);
    let env_block_sh = |aliases_abs: &str| -> String {
        format!(
            r#"# ralf environment (auto-set)
    export ralf_RC_FILE="${{ralf_RC_FILE:-{rc}}}"
    export ALF_RC_FILE="${{ALF_RC_FILE:-{rc}}}"
    export ralf_ALIASES_FILE="${{ralf_ALIASES_FILE:-{al}}}"
    export ALF_ALIASES_FILE="${{ALF_ALIASES_FILE:-{al}}}"
    export ralf_MACHINE="${{ralf_MACHINE:-{mid}}}"
    if [ -z "$RALF_MACHINE" ] && [ -z "$ralf_MACHINE" ] && [ -z "$ALF_MACHINE" ] && [ -z "$alf_MACHINE" ]; then
      export RALF_MACHINE="{mid}"; export ralf_MACHINE="{mid}"; export ALF_MACHINE="{mid}"; export alf_MACHINE="{mid}";
    fi

    "#,
            rc = rc_q,
            al = esc(aliases_abs),
            mid = mid_q
        )
    };
    let env_block_fish = |aliases_abs: &str| -> String {
        format!(
            r#"# ralf environment (auto-set)
    if not set -q ralf_RC_FILE
      set -gx ralf_RC_FILE "{rc}"
    end
    if not set -q ALF_RC_FILE
      set -gx ALF_RC_FILE "{rc}"
    end
    if not set -q ralf_ALIASES_FILE
      set -gx ralf_ALIASES_FILE "{al}"
    end
    if not set -q ALF_ALIASES_FILE
      set -gx ALF_ALIASES_FILE "{al}"
    end
    if not set -q ralf_MACHINE
      set -gx ralf_MACHINE "{mid}"
    end
    if not set -q RALF_MACHINE; and not set -q ralf_MACHINE; and not set -q ALF_MACHINE; and not set -q alf_MACHINE
      set -gx RALF_MACHINE "{mid}"; set -gx ralf_MACHINE "{mid}"; set -gx ALF_MACHINE "{mid}"; set -gx alf_MACHINE "{mid}"
    end

    "#,
            rc = rc_q,
            al = esc(aliases_abs),
            mid = mid_q
        )
    };

    // Generate both unified variants
    let mut sh_content = crate::generator::generate_config_sh_from_text(&text)?;
    sh_content = format!(
        "{}{}",
        env_block_sh(&sh_target.to_string_lossy()),
        sh_content
    );
    let mut fish_content = crate::generator::generate_config_fish_from_text(&text)?;
    fish_content = format!(
        "{}{}",
        env_block_fish(&fish_target.to_string_lossy()),
        fish_content
    );

    // Save unified files
    std::fs::write(&sh_target, sh_content)?;
    std::fs::write(&fish_target, fish_content)?;
    if !in_tui {
        println!("Saved unified aliases to:");
        println!("- {}", crate::paths::friendly(&sh_target));
        println!("- {}", crate::paths::friendly(&fish_target));
    }

    // Backward-compat: also write to requested aliases file path (env/default)
    let is_target_fish = {
        std::env::var("FISH_VERSION").is_ok()
            || std::env::var("SHELL")
                .ok()
                .map(|s| s.ends_with("fish") || s.contains("/fish"))
                .unwrap_or(false)
            || {
                let s = p.aliases_file.to_string_lossy();
                s.ends_with(".fish") || s.contains("/fish/")
            }
    };
    let compat_content = if is_target_fish {
        let mut c = crate::generator::generate_config_fish_from_text(&text)?;
        let al_q = esc(&p.aliases_file.to_string_lossy());
        c = format!("{}{}", env_block_fish(&al_q), c);
        c
    } else {
        let mut c = crate::generator::generate_config_sh_from_text(&text)?;
        let al_q = esc(&p.aliases_file.to_string_lossy());
        c = format!("{}{}", env_block_sh(&al_q), c);
        c
    };

    if !in_tui {
        println!("Saving to {}", p.aliases_file.display());
    }
    if let Some(parent) = p.aliases_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p.aliases_file, compat_content)?;

    // Install init stubs in rc files to source the unified files
    if let Some(home) = dirs::home_dir() {
        // bash/zsh: append tagged snippet if missing
        let tag = "# >>> ralf init >>>";
        let close = "# <<< ralf init <<<";
        let sh_path = sh_target.to_string_lossy();

        for rc in [&home.join(".bashrc"), &home.join(".zshrc")] {
            let snippet = format!("{tag}\n[ -f \"{p}\" ] && . \"{p}\"\n{close}\n", p = sh_path);
            match std::fs::read_to_string(rc) {
                Ok(existing) => {
                    if !existing.contains(tag) {
                        let sep = if existing.ends_with('\n') { "" } else { "\n" };
                        let new = format!("{existing}{sep}{snippet}");
                        std::fs::write(rc, new)?;
                        if !in_tui {
                            println!("Added ralf init to {}", crate::paths::friendly(rc));
                        }
                    }
                }
                Err(_) => {
                    std::fs::write(rc, snippet)?;
                    if !in_tui {
                        println!("Created {}", crate::paths::friendly(rc));
                    }
                }
            }
        }

        // fish: conf.d stub that sources unified fish file
        let fish_stub = home
            .join(".config")
            .join("fish")
            .join("conf.d")
            .join("ralf.fish");
        if let Some(parent) = fish_stub.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let stub = format!(
            "{tag}\nset -l f \"{p}\"\nif test -f $f\n  source $f\nend\n{close}\n",
            tag = "# >>> ralf init >>>",
            close = "# <<< ralf init <<<",
            p = fish_target.to_string_lossy()
        );
        match std::fs::read_to_string(&fish_stub) {
            Ok(existing) => {
                if !existing.contains("# >>> ralf init >>>") {
                    let sep = if existing.ends_with('\n') { "" } else { "\n" };
                    let new = format!("{existing}{sep}{stub}");
                    std::fs::write(&fish_stub, new)?;
                    if !in_tui {
                        println!("Added ralf init to {}", crate::paths::friendly(&fish_stub));
                    }
                }
            }
            Err(_) => {
                std::fs::write(&fish_stub, stub)?;
                if !in_tui {
                    println!("Created {}", crate::paths::friendly(&fish_stub));
                }
            }
        }
    }

    if in_tui {
        println!("Saved aliases.");
    } else {
        println!("To apply the new aliases to the current session, run:");
        println!("$ source {}", crate::paths::friendly(&p.aliases_file));
    }
    Ok(())
}
