mod command;
use crate::command::MV;
use crate::command::{Cargo, JKCommand};
use cargo_metadata::Message;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::{env, fs};
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

#[derive(Debug, Serialize)]
struct AexFileOutput {
    aex_file: String,
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

            let os_type = env::consts::OS;

            // build name and plugin name
            // these are used to set the build name and plugin name
            let build_name = cargo_toml.package.metadata.jk_plugin.build_name;
            let plugin_name = cargo_toml.package.metadata.jk_plugin.plugin_name;
            eprintln!("Build Name: {}", build_name);
            eprintln!("Plugin Name: {}", plugin_name);
            let mut command = Command::new("cargo");
            command.arg("build");
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
                    match os_type {
                        "windows" => {
                            if status.success() {
                                let dllfilepath =
                                    filename.as_ref().expect("No artifact filename found");
                                let dllfiledir = dllfilepath.parent().unwrap();
                                // rename the DLL file to the plugin name
                                let new_dll_path =
                                    dllfiledir.join(&plugin_name).with_extension("aex");
                                std::fs::rename(dllfilepath, &new_dll_path)
                                    .expect("Failed to rename DLL file");
                                eprintln!("Renamed DLL to: {}", new_dll_path.display());
                                eprintln!("Build succeeded.");
                                // check format argument
                                match build.format {
                                    command::Format::Json => {
                                        let aex_file = AexFileOutput {
                                            aex_file: new_dll_path.to_string_lossy().to_string(),
                                        };
                                        let output = serde_json::to_string(&aex_file)
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
                        "macos" => {
                            todo!("MacOS build is not implemented yet");
                        }
                        _ => {
                            eprintln!("Unsupported operating system: {}", os_type);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                    std::process::exit(1);
                }
            }
        }
        JKCommand::MV(mv) =>
        {
            #[cfg(target_os = "windows")]
            if platform::is_elevated() {
                match mv_command(&mv) {
                    Ok(_) => eprintln!("File moved successfully."),
                    Err(e) => eprintln!("Failed to move file: {e}"),
                }

                println!("Press Enter to exit...");
                let _ = io::stdout().flush();
                let _ = io::stdin().read_line(&mut String::new());
            } else {
                eprintln!("Not running with elevated privileges. Attempting to elevate...");
                platform::elevate_self();
            }
        }
        JKCommand::Install(install) => todo!(),
    }
}

fn mv_command(mv: &MV) -> io::Result<()> {
    let target_dir = Path::new("C:\\Program Files\\Adobe\\Common\\Plug-ins\\7.0\\MediaCore\\");

    let src = Path::new(&mv.src);
    // get filename from the source path
    let filename = src.file_name().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "Source file does not have a valid filename",
    ))?;
    let target = target_dir.join(filename);

    fs::copy(src, target).map_err(|e| io::Error::other(format!("Failed to move file: {e}")))?;

    Ok(())
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::{
            Foundation::*,
            Security::*,
            System::Threading::*,
            UI::{Shell::*, WindowsAndMessaging::SW_SHOW},
        },
        core::*,
    };

    pub fn is_elevated() -> bool {
        let mut token = HANDLE::default();
        unsafe {
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
                let mut elevation = TOKEN_ELEVATION::default();
                let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
                if GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut core::ffi::c_void),
                    size,
                    &mut size,
                )
                .is_ok()
                {
                    CloseHandle(token).unwrap();
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
        false
    }

    pub fn elevate_self() {
        let exe_path = std::env::current_exe().unwrap();

        let args: Vec<String> = std::env::args().skip(1).collect(); // 最初は自分自身なので skip
        let arg_str = args
            .into_iter()
            .map(|arg| {
                if arg.contains(' ') {
                    format!("\"{arg}\"")
                } else {
                    arg
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let cmd = OsStr::new(exe_path.to_str().unwrap())
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<u16>>();

        let params = OsStr::new(&arg_str)
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<u16>>();

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: PCWSTR(w!("runas").as_ptr()),
            lpFile: PCWSTR(cmd.as_ptr()),
            lpParameters: PCWSTR(params.as_ptr()),
            nShow: SW_SHOW.0,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            ..Default::default()
        };

        unsafe {
            if ShellExecuteExW(&mut sei).is_ok() {
                WaitForSingleObject(sei.hProcess, INFINITE);
                CloseHandle(sei.hProcess).unwrap();
            } else {
                eprintln!("Failed to elevate via UAC");
            }
        }

        std::process::exit(0);
    }
}
