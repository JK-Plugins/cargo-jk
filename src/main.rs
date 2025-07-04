mod build;
mod command;
mod mv;

use crate::command::{Cargo, JKCommand};
use cargo_metadata::Message;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::io;
use std::io::Write;
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
    jk_plugin: JkPluginMetadata,
}

#[derive(Debug, Deserialize)]
struct JkPluginMetadata {
    build_name: String,
    plugin_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginOutput {
    path: String,
}

fn main() {
    let Cargo::Input(input) = Cargo::parse();
    // let ostype = env::consts::OS;
    // println!("Operating System: {}", ostype);
    match input.cmd {
        JKCommand::Build(build) => {
            let _aesdk_root = env::var("AESDK_ROOT")
                .expect("AESDK_ROOT is not defined as an environment variable");

            // load Cargo.toml and read the metadata
            let current_dir = env::current_dir().expect("Failed to get current directory");
            let cargo_toml_path = current_dir.join("Cargo.toml");
            let cargo_toml_content =
                std::fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");
            let cargo_toml: CargoToml =
                toml::from_str(&cargo_toml_content).expect("Failed to parse Cargo.toml");

            // build name and plugin name
            // these are used to set the build name and plugin name
            let build_name = cargo_toml.package.metadata.jk_plugin.build_name;
            let plugin_name = cargo_toml.package.metadata.jk_plugin.plugin_name;
            eprintln!("Build Name: {}", build_name);
            eprintln!("Plugin Name: {}", plugin_name);
            let mut command = Command::new("cargo");
            command.arg("build");
            if build.release {
                command.arg("--release");
            }
            command.arg("--message-format");
            command.arg("json-render-diagnostics");
            command.stdout(Stdio::piped());
            eprintln!("Executing: {:?}", command);
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
                        let plugin_path =
                            build::post_build_process(&build, &filename, &build_name, &plugin_name);
                        eprintln!("Build succeeded.");
                        // check format argument
                        match build.format {
                            command::Format::Json => {
                                let plugin_output = PluginOutput {
                                    path: plugin_path.to_string_lossy().to_string(),
                                };
                                let output = serde_json::to_string(&plugin_output)
                                    .expect("Failed to serialize output to JSON");
                                println!("{}", output);
                            }
                            command::Format::None => {
                                // nothing to do
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
        JKCommand::MV(mv) => {
            if mv::is_elevated() {
                match mv::mv_command(&mv) {
                    Ok(_) => eprintln!("File moved successfully."),
                    Err(e) => eprintln!("Failed to move file: {e}"),
                }

                println!("Press Enter to exit...");
                let _ = io::stdout().flush();
                let _ = io::stdin().read_line(&mut String::new());
            } else {
                eprintln!("Not running with elevated privileges. Attempting to elevate...");
                mv::elevate_self();
            }
        }
        JKCommand::Install(install) => {
            install_command(install.release);
        }
    }
}

fn install_command(release: bool) {
    eprintln!("Starting install process...");

    // Detect if we're running in development mode (cargo run -- jk) or production mode (cargo jk)
    let current_exe = env::current_exe().expect("Failed to get current executable path");
    let is_dev_mode = current_exe.to_string_lossy().contains("target");

    let (cmd_prefix, cmd_args): (&str, Vec<&str>) = if is_dev_mode {
        ("cargo", vec!["run", "--", "jk"])
    } else {
        ("cargo", vec!["jk"])
    };

    // Step 1: Execute build command
    let release_flag = if release { " --release" } else { "" };
    let build_cmd = format!(
        "{} {} build{} --format json",
        cmd_prefix,
        cmd_args.join(" "),
        release_flag
    );
    eprintln!("Running: {}", build_cmd);

    let mut build_command = Command::new(cmd_prefix);
    for arg in &cmd_args {
        build_command.arg(arg);
    }
    build_command.arg("build");
    if release {
        build_command.arg("--release");
    }
    let build_output = build_command.arg("--format").arg("json").output();

    match build_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            // Step 2: Parse the JSON output to get the plugin path
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("Build output: {}", stdout);

            match serde_json::from_str::<PluginOutput>(&stdout) {
                Ok(plugin_output) => {
                    let plugin_path = &plugin_output.path;
                    eprintln!("Built plugin: {}", plugin_path);

                    // Step 3: Execute mv command
                    let mv_cmd =
                        format!("{} {} mv {}", cmd_prefix, cmd_args.join(" "), plugin_path);
                    eprintln!("Running: {}", mv_cmd);

                    let mut mv_command = Command::new(cmd_prefix);
                    for arg in &cmd_args {
                        mv_command.arg(arg);
                    }
                    let mv_output = mv_command.arg("mv").arg(plugin_path).output();

                    match mv_output {
                        Ok(mv_result) => {
                            if mv_result.status.success() {
                                eprintln!("Install completed successfully!");
                                eprintln!("{}", String::from_utf8_lossy(&mv_result.stdout));
                            } else {
                                eprintln!(
                                    "Move failed: {}",
                                    String::from_utf8_lossy(&mv_result.stderr)
                                );
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to execute move command: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse build output JSON: {}", e);
                    eprintln!("Raw output: {}", stdout);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to execute build command: {}", e);
            std::process::exit(1);
        }
    }
}
