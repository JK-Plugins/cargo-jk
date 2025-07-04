mod build;
mod command;
mod mv;

use crate::command::{Cargo, JKCommand};
use cargo_metadata::Message;
use cargo_metadata::MetadataCommand;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Deserialize)]
struct JkPluginMetadata {
    plugin_name: String,
    identifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginOutput {
    path: String,
}

fn package_for_cwd() -> cargo_metadata::Package {
    let cwd = std::env::current_dir().unwrap();
    let meta = MetadataCommand::new().no_deps().exec().unwrap();

    let mut pkgs: Vec<_> = meta
        .packages
        .into_iter()
        .filter(|p| {
            let manifest_dir = p.manifest_path.parent().unwrap();
            cwd.starts_with(manifest_dir)
        })
        .collect();

    pkgs.sort_by_key(|p| p.manifest_path.components().count());
    pkgs.pop().unwrap()
}

fn main() {
    let Cargo::Input(input) = Cargo::parse();
    // let ostype = env::consts::OS;
    // println!("Operating System: {}", ostype);
    match input.cmd {
        JKCommand::Build(build) => {
            let _aesdk_root = env::var("AESDK_ROOT")
                .expect("AESDK_ROOT is not defined as an environment variable");
            let package = package_for_cwd();

            let jk_pluging_metadata_value = package
                .metadata
                .get("jk_plugin")
                .expect("no [package.metadata.jk_plugin] section in Cargo.toml")
                .clone();
            let jk_plugin_metadata: JkPluginMetadata =
                serde_json::from_value(jk_pluging_metadata_value)
                    .map_err(|e| {
                        io::Error::other(format!("Failed to parse jk_plugin metadata: {}", e))
                    })
                    .unwrap();

            eprintln!("Plugin Name: {}", jk_plugin_metadata.plugin_name);
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
                    let mut filename: Option<PathBuf> = None;
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
                    let filename = filename.expect("No artifact filename found after build");

                    let status = child.wait().expect("Failed to wait on child process");
                    if status.success() {
                        let plugin_path: PathBuf = build::post_build_process(
                            &build,
                            &filename,
                            &package.name,
                            &jk_plugin_metadata,
                        );
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
