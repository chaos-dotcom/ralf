use anyhow::Result;
use clap::CommandFactory;

const MAN: &str = r#"NAME
  ralf - Your Little Rusty Bash & zsh Alias Friend

SYNOPSIS
  ralf [COMMAND] [ARGS]
  ralf                 # opens the TUI menu

DESCRIPTION
  ralf manages a repository of shell aliases and functions and generates
  a shell-friendly aliases file. It supports machine-specific overlays,
  and works natively in bash and zsh.

COMMANDS
  connect   Connect this machine to a config repo (clone and set rc)
  download  Pull latest changes from the repo and regenerate aliases
  upload    Commit and push changes in the repo
  generate  Print the generated aliases to stdout
  save      Write the generated aliases to your aliases file
  edit      Edit base config; 'ralf edit machine' edits machine overlay
  which     Show the command behind an alias (and optional subcommand)
  machine   Show or set current machine id used for overlays
  info      Show environment, paths, files, and git remote
  upgrade   Placeholder for self-update (not implemented yet)
  help      Show this page; 'ralf help [command]' shows command help

FILES
  ~/.ralfrc or ~/.alfrc
      Path to the repository (written by 'ralf connect')
  <repo>/ralf.conf (or alf.conf)
      Base configuration file with aliases and subcommands
  <repo>/machines/<machine>.conf
      Machine overlay, applied on top of base config
  <repo>/ralf.local.conf
      Local overlay (not shared), applied last

ENVIRONMENT
  ralf_RC_FILE / ALF_RC_FILE      Path to rc file (repo location)
  ralf_ALIASES_FILE / ALF_ALIASES_FILE
                                  Path to write the generated aliases
  ralf_MACHINE                    Machine id override
  RALF_TUI_FORCE                  '1/true/yes/on' to force TUI in connect

SHELLS
  Works in bash and zsh. For zsh, we initialize compinit and bashcompinit
  so bash-style 'complete -W' lines are honored.

EXAMPLES
  ralf help
  ralf help connect
  ralf connect you --https
  ralf save
  ralf which g l
  ralf edit machine
"#;

pub fn run(topic: Option<String>) -> Result<()> {
    if let Some(t) = topic {
        // Prefer exact subcommand name
        let mut root = crate::cli::Cli::command();
        if let Some(sub) = root.find_subcommand_mut(&t) {
            sub.print_long_help()?;
            println!();
            return Ok(());
        }
        // Fallback: resolve known aliases (shortcuts)
        let canonical = match t.as_str() {
            "c" => "connect",
            "d" | "pull" => "download",
            "u" | "push" => "upload",
            "g" => "generate",
            "s" => "save",
            "e" => "edit",
            "w" => "which",
            "m" => "machine",
            "h" => "help",
            _ => "",
        };
        if !canonical.is_empty() {
            let mut root2 = crate::cli::Cli::command();
            if let Some(sub2) = root2.find_subcommand_mut(canonical) {
                sub2.print_long_help()?;
                println!();
                return Ok(());
            }
        }
        eprintln!("Unknown help topic: {}", t);
        std::process::exit(1);
    }
    print!("{}", MAN);
    Ok(())
}
