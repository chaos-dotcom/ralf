use anyhow::Result;
use clap::Parser;

mod cli;
mod completions;
mod config_merge;
mod domain;
mod generator;
mod gitwrap;
mod paths;
mod tui;

mod cmd_clean;
mod cmd_connect;
mod cmd_download;
mod cmd_edit;
mod cmd_generate;
mod cmd_help;
mod cmd_info;
mod cmd_machine;
mod cmd_menu;
mod cmd_reset;
mod cmd_save;
mod cmd_upgrade;
mod cmd_upload;
mod cmd_which;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        Some(cli::Commands::Connect(args)) => cmd_connect::run(args)?,
        Some(cli::Commands::Download) => cmd_download::run()?,
        Some(cli::Commands::Upload) => cmd_upload::run()?,
        Some(cli::Commands::Generate) => cmd_generate::run()?,
        Some(cli::Commands::Save) => cmd_save::run()?,
        Some(cli::Commands::Edit { what }) => cmd_edit::run(what)?,
        Some(cli::Commands::Which { code, subcode }) => cmd_which::run(code, subcode)?,
        Some(cli::Commands::Help { topic }) => cmd_help::run(topic)?,
        Some(cli::Commands::Clean { purge }) => cmd_clean::run(purge)?,
        Some(cli::Commands::Reset) => cmd_reset::run()?,
        Some(cli::Commands::Upgrade) => cmd_upgrade::run()?,
        Some(cli::Commands::Machine { name }) => cmd_machine::run(name)?,
        Some(cli::Commands::Info) => cmd_info::run()?,
        None => cmd_menu::run()?,
    }
    Ok(())
}
