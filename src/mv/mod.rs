//! Platform-specific implementation for the `mv` command.
//!
//! This module provides platform-specific implementations for moving files/directories
//! to Adobe plugin directories. Each platform has different behavior due to how
//! Adobe plugins are structured on different operating systems.
//!
//! ## Platform Behavior Differences
//!
//! ### Windows
//! - **Target Directory**: `C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore\`
//! - **Input Type**: Individual **files** (`.aex` files)
//! - **Operation**: Copies files using `fs::copy()`
//! - **Elevation**: Uses UAC (User Account Control) via `ShellExecuteExW`
//!
//! ### macOS
//! - **Target Directory**: `/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/`
//! - **Input Type**: **Directories** (`.plugin` bundles)
//! - **Operation**: Copies directories using `dircpy::copy_dir()`
//! - **Elevation**: Uses `sudo` command
//!
//! This fundamental difference exists because:
//! - Windows Adobe plugins are typically single `.aex` files
//! - macOS Adobe plugins are typically `.plugin` directory bundles
//!
//! ## Error Handling
//!
//! Both platforms use standardized error handling with descriptive messages
//! and proper error context propagation.

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os_impl;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os_impl;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
compile_error!("mv_platform: unsupported operating system");

pub use os_impl::*;
