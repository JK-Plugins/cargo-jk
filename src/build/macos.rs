use std::{
    path::{Path, PathBuf},
    process::Command,
};

use plist::{Dictionary, Value};

use crate::{JkPluginMetadata, command::Build};

pub fn post_build_process<P: AsRef<Path>>(
    build: &Build,
    filename: P,
    package_name: &str,
    jk_plugin_metadata: &JkPluginMetadata,
) -> PathBuf {
    let binary_name = &package_name.to_lowercase().replace("-", "_");
    let plugin_name = &jk_plugin_metadata.plugin_name;
    let identifier = &jk_plugin_metadata.identifier;

    // set -eはrustでエラー処理を行うので不要
    // echo "Creating plugin bundle"
    eprintln!("Creating plugin bundle");

    let lib_dylib_path = filename.as_ref().to_path_buf();

    // ../target/(debug or release)/
    let target_build_dir = lib_dylib_path.parent().unwrap();

    // rm -Rf "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin"
    let plugin_dir = target_build_dir.join(&plugin_name).with_extension("plugin");

    if plugin_dir.exists() {
        std::fs::remove_dir_all(&plugin_dir).expect("Failed to remove old plugin bundle");
    }

    // mkdir -p "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources"
    // mkdir -p "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS"
    let plugin_resource_path = plugin_dir.join("Contents/Resources");
    let plugin_macos_path = plugin_dir.join("Contents/MacOS");

    std::fs::create_dir_all(&plugin_resource_path)
        .expect("Failed to create plugin Resources directory");
    std::fs::create_dir_all(&plugin_macos_path).expect("Failed to create plugin MacOS directory");

    // echo "eFKTFXTC" >> "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/PkgInfo"
    let pkg_info_path = plugin_dir.join("Contents/PkgInfo");
    std::fs::write(&pkg_info_path, "eFKTFXTC").expect("Failed to write PkgInfo file");

    // Info.plistファイルの作成
    let info_plist_path = plugin_dir.join("Contents/Info.plist");

    let mut plist_dict = Dictionary::new();
    plist_dict.insert(
        "CFBundlePackageType".to_string(),
        Value::String("eFKT".to_string()),
    );
    plist_dict.insert(
        "CFBundleSignature".to_string(),
        Value::String("FXTC".to_string()),
    );

    // Bundle Identifierの設定（適切な値に変更してください）
    let bundle_identifier = identifier.to_string();
    plist_dict.insert(
        "CFBundleIdentifier".to_string(),
        Value::String(bundle_identifier),
    );

    let plist_value = Value::Dictionary(plist_dict);
    plist_value
        .to_file_xml(&info_plist_path)
        .expect("Failed to write Info.plist file");

    if build.release {
        // # Build universal binary
        let target_dir = target_build_dir.parent().unwrap();
        let x86_64 = "x86_64-apple-darwin";
        let aarch64 = "aarch64-apple-darwin";

        // rustup target add aarch64-apple-darwin
        Command::new("rustup")
            .arg("target")
            .arg("add")
            .arg(aarch64)
            .status()
            .expect("Failed to add aarch64 target");

        // rustup target add x86_64-apple-darwin
        Command::new("rustup")
            .arg("target")
            .arg("add")
            .arg(x86_64)
            .status()
            .expect("Failed to add x86_64 target");

        // cargo build --release --target x86_64-apple-darwin
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--target")
            .arg(x86_64)
            .current_dir(target_dir)
            .status()
            .expect("Failed to build for x86_64 target");

        // cargo build --release --target aarch64-apple-darwin
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--target")
            .arg(aarch64)
            .current_dir(target_dir)
            .status()
            .expect("Failed to build for aarch64 target");

        // cp "{{TargetDir}}/x86_64-apple-darwin/release/{{BinaryName}}.rsrc" "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources/{{PluginName}}.rsrc"
        std::fs::copy(
            target_dir
                .join(x86_64)
                .join("release")
                .join(binary_name)
                .with_extension("rsrc"),
            &plugin_resource_path
                .join(plugin_name)
                .with_extension("rsrc"),
        )
        .expect("Failed to copy resource file");

        // lipo "{{TargetDir}}/{x86_64,aarch64}-apple-darwin/release/lib{{BinaryName}}.dylib" -create -output "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}.dylib"
        Command::new("lipo")
            .arg(
                target_dir
                    .join(x86_64)
                    .join("release")
                    .join(lib_dylib_path.file_name().unwrap()),
            )
            .arg(
                target_dir
                    .join(aarch64)
                    .join("release")
                    .join(lib_dylib_path.file_name().unwrap()),
            )
            .arg("-create")
            .arg("-output")
            .arg(plugin_macos_path.join(plugin_name).with_extension("dylib"))
            .status()
            .expect("Failed to create universal binary");

        // mv "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}.dylib" "{{TargetDir}}/{{profile}}/{{PluginName}}"
        std::fs::rename(
            plugin_macos_path.join(plugin_name).with_extension("dylib"),
            &plugin_dir.join(&plugin_name),
        )
        .expect("Failed to rename binary file to plugin name");
    } else {
        // cp "{{TargetDir}}/{{profile}}/{{BuildName}}.rsrc" "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources/{{PluginName}}.rsrc"
        std::fs::copy(
            &target_build_dir.join(package_name).with_extension("rsrc"),
            &plugin_resource_path
                .join(&plugin_name)
                .with_extension("rsrc"),
        )
        .expect("Failed to copy resource file");

        // cp "{{TargetDir}}/{{profile}}/lib{{BinaryName}}.dylib" "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}"
        std::fs::copy(&lib_dylib_path, &plugin_macos_path.join(&plugin_name))
            .expect("Failed to copy binary file");
    }

    Command::new("codesign")
        .arg("--options")
        .arg("runtime")
        .arg("--timestamp")
        .arg("-strict")
        .arg("--sign")
        .arg("-") // Use ad-hoc signing
        .arg(&plugin_dir)
        .status()
        .expect("Failed to codesign the plugin");

    plugin_dir
}
