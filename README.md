# ssv

`ssv` は `~/.ssh/conf.d/` 配下で SSH 鍵ペアおよびホスト設定ファイルを管理する Rust 製 CLI ツールです。必要な SSH 設定レイアウトのブートストラップ、`ssh-keygen` による鍵生成、`ssh-copy-id` を介したリモートサーバーへの公開鍵設置、管理ホスト設定の更新、リポジトリリモートの書き換え、管理ホスト一覧の表示、公開鍵の表示、アセットの監査、資格情報の削除を扱います。

## 機能

- 初期化: `ssv init` (エイリアス: `i`) により `~/.ssh`、`~/.ssh/conf.d`、`~/.ssh/config` の準備状態が保証されます。
- 鍵生成: `ssv generate` (エイリアス: `g`) により `ssh-keygen` のラッパーとしてホスト固有設定と鍵ペアが生成されます。
- 公開鍵設置: `ssv authorize <HOST>` (エイリアス: `az`) により `ssh-copy-id` を介してリモートの `authorized_keys` に公開鍵が設置されます。
- 設定更新: `ssv set <HOST>` (エイリアス: `s`) により既存の管理ホストの `HostName`、`User`、`Port` が更新されます。
- リポジトリ連携: `ssv link <HOST>` (エイリアス: `ln`) によりリポジトリの `origin` URL が管理ホストへ書き換えられます。
- 一覧表示: `ssv list` (エイリアス: `ls`) により管理下のホスト一覧が表示されます。
- 公開鍵表示: `ssv show <HOST>` (エイリアス: `sw`) により管理ホストの公開鍵が表示されます。
- 監査: `ssv audit` (エイリアス: `au`) によりアセットの欠損やパーミッション不整合が読み取り専用で検査されます。
- 削除: `ssv remove` (エイリアス: `rm`) により指定したホストの鍵ペアおよび設定が削除されます。

## サブコマンド

| サブコマンド | エイリアス | 概要 |
| --- | --- | --- |
| init | i | SSH 設定レイアウトの初期化 |
| generate | g | 鍵ペアおよびホスト設定ファイルの生成 |
| set | s | 管理ホストの設定更新 |
| authorize | az | リモートサーバーへの公開鍵設置 |
| list | ls | 管理ホストの一覧表示 |
| remove | rm | 鍵ペアおよび設定の削除 |
| show | sw | 公開鍵の表示 |
| link | ln | リポジトリの origin リモート書き換え |
| audit | au | 管理アセットの監査 |

## エージェントプラグイン

`plugin/` ディレクトリには、AI エージェント（Claude Code および Codex）向けの Agent Skills プラグインが含まれます。

### ディレクトリ構造

```text
plugin/
├── .claude-plugin/
│   └── plugin.json
├── .codex-plugin/
│   └── plugin.json
└── skills/
    ├── github-ssh-setup/
    │   ├── SKILL.md
    │   └── agents/openai.yaml
    └── ip-ssh-setup/
        ├── SKILL.md
        └── agents/openai.yaml
```

### 提供スキル

| スキル名 | 引数ヒント | 概要 |
| --- | --- | --- |
| github-ssh-setup | `<HOST>` | `ssv` を使用した GitHub 用 SSH ホストの構築および公開鍵登録 |
| ip-ssh-setup | `<IP> <USERNAME>` | IP アドレスとユーザー名を指定した SSH 接続ホストの構築 |

### インストール

マーケットプレイス経由またはローカルディレクトリの指定により読み込みが可能です。

#### Claude Code

マーケットプレイス経由でのインストール:

```bash
claude plugin marketplace add akitorahayashi/ssv
claude plugin install ssv-plugin@ssv
```

ローカルセッションでの一時読み込み:

```bash
claude --plugin-dir ./plugin
```

#### Codex

マーケットプレイス経由でのインストール:

```bash
codex plugin marketplace add akitorahayashi/ssv
codex plugin add ssv-plugin@ssv
```

## 配置規則

設定ファイルは `~/.ssh/conf.d/<HOST>.conf` に保存され、鍵ペアは `~/.ssh/id_<TYPE>_<HOST>` の命名規則に従います。生成される設定は `-t/--type`、`-u/--user`、`-p/--port`、`-n/--hostname` の各オプションに対応します。
