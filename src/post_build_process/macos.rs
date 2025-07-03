use plist::{Dictionary, Value};

use crate::command::Build;

pub fn post_build_process(
    build: &Build,
    filename: &Option<std::path::PathBuf>,
    build_name: &str,
    plugin_name: &str,
) {
    // set -eはrustでエラー処理を行うので不要
    // echo "Creating plugin bundle"
    eprintln!("Creating plugin bundle");

    // filenameが"../target/debug/lib(build_name).dylib"なので調整とか
    let lib_dylib_path = filename.as_ref().expect("No artifact filename found");
    let lib_dylib_dir = lib_dylib_path.parent().unwrap();

    // rm -Rf "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin"
    let plugin_path = lib_dylib_dir.join(&plugin_name).with_extension("plugin");

    if !plugin_path.exists() {
        std::fs::remove_file(&plugin_path).expect("Failed to remove old plugin bundle");
    }

    // mkdir -p "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources"
    // mkdir -p "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS"
    let plugin_resource_path = plugin_path.join("Contents/Resources");
    let plugin_macos_path = plugin_path.join("Contents/MaxOS");

    std::fs::create_dir_all(&plugin_resource_path)
        .expect("Failed to create plugin Resources directory");
    std::fs::create_dir_all(&plugin_macos_path).expect("Failed to create plugin MacOS directory");

    // echo "eFKTFXTC" >> "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/PkgInfo"
    let pkg_info_path = plugin_path.join("Contents/PkgInfo");
    std::fs::write(&pkg_info_path, "eFKTFXTC").expect("Failed to write PkgInfo file");

    // Info.plistファイルの作成
    let info_plist_path = plugin_path.join("Contents/Info.plist");

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
    let bundle_identifier = format!("com.example.{}", plugin_name);
    plist_dict.insert(
        "CFBundleIdentifier".to_string(),
        Value::String(bundle_identifier),
    );

    let plist_value = Value::Dictionary(plist_dict);
    plist_value
        .to_file_xml(&info_plist_path)
        .expect("Failed to write Info.plist file");
}
