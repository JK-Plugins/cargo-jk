use std::{io, path::Path, process::Command};

use nix::unistd::Uid;

use crate::command::MV;

pub fn is_elevated() -> bool {
    Uid::current().is_root()
}

pub fn elevate_self() {
    // Use `sudo` to re-run the command with elevated privileges
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = Command::new("sudo")
        .arg(std::env::current_exe().unwrap())
        .args(args)
        .status()
        .expect("Failed to execute sudo command");

    if !status.success() {
        eprintln!("Failed to elevate privileges");
        std::process::exit(1);
    }
}

// macOS
// dst は "/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/"
// src はコマンドライン引数で指定されたパスで、windows版と違いプラグインはディレクトリなので、ディレクトリをコピーする
pub fn mv_command(mv: &MV) -> io::Result<()> {
    let target_dir = Path::new("/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/");

    let src = Path::new(&mv.src);
    // get directory name from the source path
    let dirname = src.file_name().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "Source directory does not have a valid name",
    ))?;
    let target = target_dir.join(dirname);

    dircpy::copy_dir(src, &target)?;

    Ok(())
}
