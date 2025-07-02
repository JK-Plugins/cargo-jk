mod command;
use crate::command::{Cargo, JKCommand};
use cargo_metadata::Message;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::io;
use std::process::Command;
use std::process::Stdio;
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: PackageMetadata,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    aftereffects: AfterEffectsMetadata,
}

#[derive(Debug, Deserialize)]
struct AfterEffectsMetadata {
    buildname: String,
    pluginname: String,
}

fn main() {
    let Cargo::Input(input) = Cargo::parse();
    // let ostype = env::consts::OS;
    // println!("Operating System: {}", ostype);
    let aesdk_root =
        env::var("AESDK_ROOT").expect("AESDK_ROOT is not defined as an environment variable");
    match input.cmd {
        JKCommand::Build(build) => {
            // load Cargo.toml and read the metadata
            let current_dir = env::current_dir().expect("Failed to get current directory");
            let cargo_toml_path = current_dir.join("Cargo.toml");
            let cargo_toml_content =
                std::fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");
            let cargo_toml: CargoToml =
                toml::from_str(&cargo_toml_content).expect("Failed to parse Cargo.toml");

            // build name and plugin name
            // these are used to set the build name and plugin name
            let buildname = cargo_toml.package.metadata.aftereffects.buildname;
            let pluginname = cargo_toml.package.metadata.aftereffects.pluginname;
            println!("Build Name: {}", buildname);
            println!("Plugin Name: {}", pluginname);
            let mut command = Command::new("cargo");
            command.arg("build");
            command.arg("--message-format");
            command.arg("json-render-diagnostics");
            command.stdout(Stdio::piped());
            println!("Executing: {:?}", command);

            match command.spawn() {
                Ok(mut child) => {
                    let reader = io::BufReader::new(child.stdout.take().unwrap());
                    let mut filename: Option<std::path::PathBuf> = None;
                    for message in cargo_metadata::Message::parse_stream(reader) {
                        match message.unwrap() {
                            Message::CompilerArtifact(artifact) => {
                                if let Some(first) = artifact.filenames.get(0) {
                                    filename = Some(first.clone().into());
                                }
                            }
                            _ => (), // Unknown message
                        }
                    }
                    let status = child.wait().expect("Failed to wait on child process");
                    if status.success() {
                        let dllfilepath = filename.as_ref().expect("No artifact filename found");
                        let dllfiledir = dllfilepath.parent().unwrap();
                        // rename the DLL file to the plugin name
                        let new_dll_path = dllfiledir.join(&pluginname).with_extension("aex");
                        std::fs::rename(dllfilepath, &new_dll_path)
                            .expect("Failed to rename DLL file");
                        println!("Renamed DLL to: {}", new_dll_path.display());
                        println!("Build succeeded.");
                        // check format argument
                        if let Some(format) = build.format {
                            if format == "json" {
                                // If the format is json, we will output the aex file path in JSON format
                                let output = serde_json::json!({
                                    "aex_file": new_dll_path.to_string_lossy()
                                });
                                println!("{}", serde_json::to_string(&output).unwrap());
                            } else {
                                eprintln!("Unsupported format: {}", format);
                                std::process::exit(1);
                            }
                        }
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
