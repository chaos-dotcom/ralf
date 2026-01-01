use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ralf", version, about = "Your Little Rusty Bash & zsh Alias Friend")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(alias = "c")]
    Connect(ConnectArgs),

    #[command(aliases = ["d", "pull"])]
    Download,

    #[command(aliases = ["u", "push"])]
    Upload,

    #[command(alias = "g")]
    Generate,

    #[command(alias = "s")]
    Save,

    #[command(alias = "e")]
    Edit { what: Option<String> },

    #[command(alias = "w")]
    Which {
        code: String,
        subcode: Option<String>,
    },

    #[command(alias = "m")]
    Machine { name: Option<String> },

    #[command(alias = "h")]
    Help { topic: Option<String> },

    Upgrade,
    Info,
}

#[derive(Args, Debug)]
pub struct ConnectArgs {
    pub repo: String,
    #[arg(long)]
    pub ssh: bool,
    #[arg(long)]
    pub https: bool,
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    #[arg(long)]
    pub tui: bool,
}
