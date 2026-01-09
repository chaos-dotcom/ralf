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
mod cmd_menu;
mod cmd_help;
mod cmd_clean;
mod cmd_reset;

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
