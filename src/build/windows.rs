use std::path::{Path, PathBuf};

use crate::command::Build;

pub fn post_build_process<P: AsRef<Path>>(
    _build: &Build,
    filename: P,
    _package_name: &str,
    plugin_name: &str,
) -> PathBuf {
    let dllfilepath = filename.as_ref().to_path_buf();
    let dllfiledir = dllfilepath.parent().unwrap();
    // rename the DLL file to the plugin name
    let new_dll_path = dllfiledir.join(&plugin_name).with_extension("aex");
    std::fs::rename(dllfilepath, &new_dll_path).expect("Failed to rename DLL file");
    eprintln!("Renamed DLL to: {}", new_dll_path.display());

    new_dll_path
}
