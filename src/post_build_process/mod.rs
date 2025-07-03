// #[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os_post_build_process;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os_post_build_process;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
compile_error!("mv_platform: unsupported operating system");

pub use os_post_build_process::*;
