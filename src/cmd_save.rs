use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;

    // Detect fish output target to generate proper prelude
    let is_fish = std::env::var("FISH_VERSION").is_ok()
        || std::env::var("SHELL")
            .ok()
            .map(|s| s.ends_with("fish") || s.contains("/fish"))
            .unwrap_or(false)
        || {
            let s = p.aliases_file.to_string_lossy();
            s.ends_with(".fish") || s.contains("/fish/")
        };

    let mut content = crate::generator::generate_config()?;
    // Prepend environment exports (do not override if already set)
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"").replace('$', "\\$");
    let rc_q = esc(&p.rc_file.to_string_lossy());
    let al_q = esc(&p.aliases_file.to_string_lossy());
    let machine = crate::config_merge::resolve_machine_id(&p);
    let mid_q = esc(&machine);

    let mut env_block = if is_fish {
        format!(
            "# ralf environment (auto-set)\n\
            if not set -q ralf_RC_FILE\n  set -gx ralf_RC_FILE \"{rc}\"\nend\n\
            if not set -q ALF_RC_FILE\n  set -gx ALF_RC_FILE \"{rc}\"\nend\n\
            if not set -q ralf_ALIASES_FILE\n  set -gx ralf_ALIASES_FILE \"{al}\"\nend\n\
            if not set -q ALF_ALIASES_FILE\n  set -gx ALF_ALIASES_FILE \"{al}\"\nend\n\
            if not set -q ralf_MACHINE\n  set -gx ralf_MACHINE \"{mid}\"\nend\n\n",
            rc = rc_q,
            al = al_q,
            mid = mid_q
        )
    } else {
        format!(
            "# ralf environment (auto-set)\n\
            export ralf_RC_FILE=\"${{ralf_RC_FILE:-{rc}}}\"\n\
            export ALF_RC_FILE=\"${{ALF_RC_FILE:-{rc}}}\"\n\
            export ralf_ALIASES_FILE=\"${{ralf_ALIASES_FILE:-{al}}}\"\n\
            export ALF_ALIASES_FILE=\"${{ALF_ALIASES_FILE:-{al}}}\"\n\
            export ralf_MACHINE=\"${{ralf_MACHINE:-{mid}}}\"\n\n",
            rc = rc_q,
            al = al_q,
            mid = mid_q
        )
    };

    if is_fish {
        env_block.push_str(&format!(
            "if not set -q RALF_MACHINE; and not set -q ralf_MACHINE; and not set -q ALF_MACHINE; and not set -q alf_MACHINE\n  set -gx RALF_MACHINE \"{mid}\"; set -gx ralf_MACHINE \"{mid}\"; set -gx ALF_MACHINE \"{mid}\"; set -gx alf_MACHINE \"{mid}\"\nend\n",
            mid = mid_q
        ));
    } else {
        env_block.push_str(&format!(
            "if [ -z \"$RALF_MACHINE\" ] && [ -z \"$ralf_MACHINE\" ] && [ -z \"$ALF_MACHINE\" ] && [ -z \"$alf_MACHINE\" ]; then\n  export RALF_MACHINE=\"{mid}\"; export ralf_MACHINE=\"{mid}\"; export ALF_MACHINE=\"{mid}\"; export alf_MACHINE=\"{mid}\";\nfi\n",
            mid = mid_q
        ));
    }

    content = format!("{env}{body}", env = env_block, body = content);

    println!("Saving to {}", p.aliases_file.display());
    if let Some(parent) = p.aliases_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&p.aliases_file, content)?;
    // Auto-source aliases file in future shells, unless overridden by env
    let aliases_env_set = std::env::var("RALF_ALIASES_FILE").is_ok()
        || std::env::var("ralf_ALIASES_FILE").is_ok()
        || std::env::var("ALF_ALIASES_FILE").is_ok();
    if !aliases_env_set && !is_fish {
        if let Some(home) = dirs::home_dir() {
            let shell = std::env::var("SHELL").unwrap_or_default();
            let rcfile = if shell.ends_with("zsh") {
                home.join(".zshrc")
            } else {
                home.join(".bashrc")
            };
            let tag = "# >>> ralf init >>>";
            let snippet = format!(
                "{tag}\n[ -f \"{p}\" ] && . \"{p}\"\n# <<< ralf init <<<\n",
                p = p.aliases_file.display()
            );
            match std::fs::read_to_string(&rcfile) {
                Ok(existing) => {
                    if !existing.contains(tag) {
                        let new = format!("{existing}\n{snippet}");
                        std::fs::write(&rcfile, new)?;
                        println!("Added ralf init to {}", crate::paths::friendly(&rcfile));
                    }
                }
                Err(_) => {
                    std::fs::write(&rcfile, snippet)?;
                    println!("Created {}", crate::paths::friendly(&rcfile));
                }
            }
        }
    }

    println!("To apply the new aliases to the current session, run:");
    println!("$ source {}", crate::paths::friendly(&p.aliases_file));
    Ok(())
}
