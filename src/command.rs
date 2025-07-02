use clap::{Args, Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    /// Command to build JK plugins
    #[command(name = "jk")]
    Input(Input),
}

#[derive(Args, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub cmd: JKCommand,

    #[arg(long, global = true)]
    pub config: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum JKCommand {
    /// Command to build a JK plugin
    Build(Build),
}

#[derive(Args, Debug)]
pub struct Build {
    #[arg(long, default_value = "none")]
    pub format: Format,
}

use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    /// Output in JSON format
    Json,
    /// No output format specified
    None,
}
