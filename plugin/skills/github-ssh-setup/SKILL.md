---
name: github-ssh-setup
description: ユーザーが ssv を使って GitHub への SSH 接続用ホストを作成したい、鍵ペアを生成したい、または公開鍵を登録したいと求めた場合に使用する。
disable-model-invocation: true
argument-hint: <HOST>
---

# ssv による GitHub SSH セットアップ

管理対象の SSH ホストを作成し、鍵ペアを生成して公開鍵の登録まで案内する。

## 入力

| 入力 | 取得方法 |
|---|---|
| ホスト識別子 | 未指定の場合はユーザーに確認する。例: `play`、`github.com-work` |
| GitHub アカウント種別 | コンテキストから推定する（個人 / 仕事）。判断できない場合は確認する |

## 手順

### 1. SSH レイアウトのブートストラップ

マシンごとに一度だけ実行する。完了済みであればスキップする。

```bash
ssv init
```

期待される出力: `SSH bootstrap is already up-to-date`（冪等）

### 2. 鍵ペアとホスト設定の生成

```bash
ssv generate <HOST> -n github.com -u git
```

- `<HOST>` はユーザーが指定したホスト識別子。
- `-n github.com` は SSH 接続先を GitHub に向ける `HostName` 設定。
- `-u git` は GitHub が要求する SSH ユーザー。
- 鍵種別はデフォルトで `ed25519`。明示的に要求された場合のみ `-t rsa` を追加する。

期待される出力:

```
Generated SSH assets for '<HOST>'
ssh-ed25519 <公開鍵> <コメント>
```

生成されるファイル:
- `~/.ssh/conf.d/<HOST>.conf` — 鍵を参照するホスト設定ファイル
- `~/.ssh/id_ed25519_<HOST>` — 秘密鍵（絶対に共有しない）
- `~/.ssh/id_ed25519_<HOST>.pub` — 公開鍵

### 3. 公開鍵をユーザーに提示する

生成時に表示された公開鍵をそのまま提示する。後から取得する場合は以下を実行する。

```bash
ssv show <HOST>
```

コピーしやすいようにコードブロックで表示する。

### 4. GitHub に公開鍵を登録する

ユーザーに以下を案内する。

1. GitHub の **Settings → SSH and GPG keys → New SSH key** を開く。
2. 手順 3 で提示した公開鍵を貼り付ける。
3. 保存する。

### 5. 接続を確認する

```bash
ssh -T git@<HOST>
```

成功時の出力（GitHub からのメッセージ）:

```
Hi <ユーザー名>! You've successfully authenticated, but GitHub does not provide shell access.
```

注意: GitHub の仕様上、シェルアクセスが無効なため `ssh -T` は成功時もプロセス終了ステータス `1` を返します。成功判定は終了コードではなく、標準出力に `Hi <ユーザー名>! You've successfully authenticated` が含まれているかで判断します。出力内に認証成功メッセージがない場合のみ `ssv audit` を実行します。


## 障害対応

| 症状 | 対応 |
|---|---|
| `ssv generate` でホストが既に存在すると報告される | `ssv show <HOST>` で既存鍵を表示し `ssh -T` で接続検証する。設定変更は `ssv set` を使用し、鍵ローテーション要求時のみユーザー確認後に `ssv remove` して再生成する |
| `ssh -T` で `Permission denied` が返る | 公開鍵が GitHub に登録されているか確認する。`ssv show <HOST>` で鍵を再表示して照合する |
| `ssv audit` で不整合が報告される | 設定不整合は `ssv set` で修正し、鍵ペア欠損・破損時のみユーザー確認後に `ssv remove` 経由で再生成する |
| `ssv init` が失敗する | `~/.ssh` のパーミッションを確認する。`700` でなければならない |
