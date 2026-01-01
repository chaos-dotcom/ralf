use anyhow::Result;
use clap::Parser;

mod cli;
mod paths;
mod generator;
mod completions;
mod tui;
mod gitwrap;
mod config_merge;

mod cmd_connect;
mod cmd_download;
mod cmd_upload;
mod cmd_generate;
mod cmd_save;
mod cmd_edit;
mod cmd_which;
mod cmd_upgrade;
mod cmd_info;
mod cmd_machine;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Connect(args) => cmd_connect::run(args)?,
        cli::Commands::Download => cmd_download::run()?,
        cli::Commands::Upload => cmd_upload::run()?,
        cli::Commands::Generate => cmd_generate::run()?,
        cli::Commands::Save => cmd_save::run()?,
        cli::Commands::Edit { what } => cmd_edit::run(what)?,
        cli::Commands::Which { code, subcode } => cmd_which::run(code, subcode)?,
        cli::Commands::Upgrade => cmd_upgrade::run()?,
        cli::Commands::Machine { name } => cmd_machine::run(name)?,
        cli::Commands::Info => cmd_info::run()?,
    }
    Ok(())
}
