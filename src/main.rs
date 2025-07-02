mod command;
use crate::command::{Cargo, JKCommand};
use clap::Parser;
use std::env;
use std::process::Command;

fn main() {
    let Cargo::Input(input) = Cargo::parse();
    // let ostype = env::consts::OS;
    // println!("Operating System: {}", ostype);
    match env::var("AESDK_ROOT") {
        Ok(val) => println!("AESDK_ROOT: {}", val),
        Err(e) => println!("AESDK_ROOT is not set: {}", e),
    }
    match input.cmd {
        JKCommand::Build(_) => {
            let mut command = Command::new("cargo");
            command.arg("build");
            println!("Executing: {:?}", command);
            match command.status() {
                Ok(status) => {
                    if status.success() {
                        println!("Build succeeded.");
                    } else {
                        eprintln!("Build failed with status: {}", status);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}