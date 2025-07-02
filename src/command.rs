use clap::{Args, Parser, Subcommand};
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    #[command(name = "jk")]
    Input(Input),
}

#[derive(Args, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub cmd: JKCommand,
}

#[derive(Subcommand, Debug)]
pub enum JKCommand {
    Build(Build),
}

#[derive(Args, Debug)]
pub struct Build {}
