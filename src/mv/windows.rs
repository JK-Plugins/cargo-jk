use std::{fs, io, path::Path, process::Command};

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

use crate::command::MV;

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

// Windows
// dst は "C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore\"
// src はコマンドライン引数で指定されたパスで、ファイルをコピーする
pub fn mv_command(mv: &MV) -> io::Result<()> {
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
