use serde::{Deserialize, Serialize};

use crate::command::{self, Build};

#[derive(Debug, Serialize, Deserialize)]
struct AexFileOutput {
    aex_file: String,
}

pub fn post_build_process(
    build: &Build,
    filename: &Option<std::path::PathBuf>,
    build_name: &str,
    plugin_name: &str,
) {
    let dllfilepath = filename.as_ref().expect("No artifact filename found");
    let dllfiledir = dllfilepath.parent().unwrap();
    // rename the DLL file to the plugin name
    let new_dll_path = dllfiledir.join(&plugin_name).with_extension("aex");
    std::fs::rename(dllfilepath, &new_dll_path).expect("Failed to rename DLL file");
    eprintln!("Renamed DLL to: {}", new_dll_path.display());
    eprintln!("Build succeeded.");
    // check format argument
    match build.format {
        command::Format::Json => {
            let aex_file = AexFileOutput {
                aex_file: new_dll_path.to_string_lossy().to_string(),
            };
            let output =
                serde_json::to_string(&aex_file).expect("Failed to serialize output to JSON");
            println!("{}", output);
        }
        command::Format::None => {
            // nothing to do
        }
    }
}
