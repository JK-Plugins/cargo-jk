use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::command::{self, Build};

pub fn post_build_process(
    build: &Build,
    filename: &Option<std::path::PathBuf>,
    build_name: &str,
    plugin_name: &str,
) -> PathBuf {
    let dllfilepath = filename.as_ref().expect("No artifact filename found");
    let dllfiledir = dllfilepath.parent().unwrap();
    // rename the DLL file to the plugin name
    let new_dll_path = dllfiledir.join(&plugin_name).with_extension("aex");
    std::fs::rename(dllfilepath, &new_dll_path).expect("Failed to rename DLL file");
    eprintln!("Renamed DLL to: {}", new_dll_path.display());

    new_dll_path
}
