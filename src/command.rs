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
    /// Command to move a file
    MV(MV),
    /// Command to build and install a JK plugin
    Install(Install),
}

#[derive(Args, Debug)]
pub struct Build {
    #[arg(long, default_value = "none")]
    pub format: Format,
    /// Build artifacts in release mode, with optimizations
    #[arg(long, default_value_t = false)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct MV {
    /// The source file to move
    pub src: String,
}

#[derive(Args, Debug)]
pub struct Install {
    /// Build artifacts in release mode, with optimizations
    #[arg(long, default_value_t = false)]
    pub release: bool,
}

use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    /// Output in JSON format
    Json,
    /// No output format specified
    None,
}
