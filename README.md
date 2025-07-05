# cargo-jk

cargo-jkは、JKプラグイン（Adobe After Effectsプラグイン）を構築するためのCargoプラグインです。

## 前提条件

- Rust開発環境
- `AESDK_ROOT`環境変数が設定されている必要があります
- Cargo.tomlに`[package.metadata.jk_plugin]`セクションが設定されている必要があります

## ヘルプ

```
Command to build JK plugins

Usage: cargo jk [OPTIONS] <COMMAND>

Commands:
  build    Command to build a JK plugin
  mv       Command to move a file
  install  Command to build and install a JK plugin
  help     Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>
  -h, --help             Print help
```

## Cargo.tomlの設定

プロジェクトのCargo.tomlファイルに以下の設定が必要です：

```toml
[package.metadata.jk_plugin]
plugin_name = "YourPluginName"
identifier = "com.yourcompany.yourplugin"
```

**注意：** この設定がない場合、ビルドは失敗します。

## コマンド

### cargo jk build

JKプラグインをビルドします。

```bash
cargo jk build
```

#### オプション

- `--release`: リリースモードでビルドします（最適化有効）
- `--format <FORMAT>`: 出力形式を指定します（json | none）

#### 例

```bash
# デバッグモードでビルド
cargo jk build

# リリースモードでビルド
cargo jk build --release

# JSON形式で出力
cargo jk build --format json
```

### cargo jk mv

ビルドしたプラグインファイルを指定した場所に移動します。

```bash
cargo jk mv
```

このコマンドは、ビルドされたプラグインファイル（.aex）を適切な場所に移動するために使用されます。

### cargo jk install

JKプラグインをビルドして、システムにインストールします。

```bash
cargo jk install
```

#### オプション

- `--release`: リリースモードでビルドとインストールを行います

#### 例

```bash
# デバッグモードでビルド・インストール
cargo jk install

# リリースモードでビルド・インストール
cargo jk install --release
```

`install`コマンドは以下の処理を自動的に行います：
1. `cargo jk build`でプラグインをビルド
2. `cargo jk mv`でプラグインファイルをシステムの適切な場所に移動

## リリース版への切り替え

リリース版（最適化されたバージョン）を使用するには、各コマンドに`--release`フラグを追加してください：

```bash
# リリースモードでビルド
cargo jk build --release

# リリースモードでビルド・インストール
cargo jk install --release
```

リリースモードでは、Rustコンパイラの最適化が有効になり、実行速度が向上したプラグインが生成されます。

## 環境変数

- `AESDK_ROOT`: Adobe After Effects SDKのルートディレクトリを指定する必要があります

## サポートするプラットフォーム

- Windows
- macOS

## プラグインの出力

ビルドが成功すると、プラグインファイル（.aex）が生成されます。ファイル名は`[package.metadata.jk_plugin]`セクションの`plugin_name`設定に基づいて決定されます。

## トラブルシューティング

### "no [package.metadata.jk_plugin] section in Cargo.toml"エラー

このエラーが発生した場合、Cargo.tomlファイルに必要なメタデータセクションを追加してください：

```toml
[package.metadata.jk_plugin]
plugin_name = "YourPluginName"
identifier = "com.yourcompany.yourplugin"
```

### "AESDK_ROOT is not defined"エラー

`AESDK_ROOT`環境変数が設定されていません。Adobe After Effects SDKのルートディレクトリを指定してください。

## 例

完全な例：

```bash
# 環境変数の設定
export AESDK_ROOT=/path/to/aesdk

# プラグインのビルド
cargo jk build --release

# プラグインのインストール
cargo jk install --release
```