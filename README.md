# tmrch

`tmrch` は、拡張子ごとのテンプレートから新規ファイルを作成する CLI です。テンプレートがない拡張子では空ファイルを作成します。

## 導入

Rust の stable toolchain を用意してから、リポジトリを取得してインストールします。

```console
git clone https://github.com/UtakataKyosui/tmtch.git
cd tmtch
cargo install --path .
```

インストール後は `tmrch --help` で確認できます。`cargo install` の出力先（通常は `~/.cargo/bin`）を `PATH` に追加してください。

## 使い方

```console
# Rust テンプレートで新規ファイルを作成
tmrch app.rs

# テンプレートがなければ空ファイルを作成
tmrch README

# rs 用テンプレートを既定エディタで編集
tmrch --edit rs

# 登録済みテンプレートを一覧表示
tmrch --list
```

既存ファイルは上書きしません。親ディレクトリはあらかじめ作成してください。

## テンプレートとエディタ

テンプレートは OS の設定ディレクトリ配下に保存されます。

- macOS: `~/Library/Application Support/tmrch/templates/`
- Linux: `$XDG_CONFIG_HOME/tmrch/templates/`、未設定時は `~/.config/tmrch/templates/`
- Windows: `%APPDATA%/tmrch/templates/`

`tmrch --edit rs` は `templates/rs` を作成または開きます。エディタは次の順で選ばれます。

1. `config.toml` の `editor`
2. `VISUAL`
3. `EDITOR`

設定ファイルはテンプレートディレクトリの親にある `config.toml` です。たとえば VS Code を使う場合は、次の内容にします。

```toml
editor = "code --wait"
```

`--wait` を指定すると、エディタを閉じるまで `tmrch` は終了しません。
