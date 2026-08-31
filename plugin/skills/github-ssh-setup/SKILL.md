---
name: github-ssh-setup
description: ssv を使って GitHub SSH ホストを作成し、公開鍵登録と接続確認を案内する。
compatibility: ssv、ssh、および GitHub へのネットワーク接続が必要
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

出力は作成、修復、または準備済みの状態を示す。再実行は同じレイアウトを維持する。

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

1. GitHub の Settings → SSH and GPG keys → New SSH key を開く。
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

注意: GitHub の仕様上、シェルアクセスが無効なため `ssh -T` は成功時もプロセス終了ステータス `1` を返します。成功判定は終了コードではなく、SSH の出力に `Hi <ユーザー名>! You've successfully authenticated` が含まれているかで判断します。認証成功メッセージがない場合のみ `ssv audit` を実行します。

## 障害対応

| 症状 | 対応 |
|---|---|
| `ssv generate` でホストが既に存在すると報告される | `ssv audit` で既存状態を確認する。正常であれば `ssv show <HOST>` と `ssh -T` で検証し、設定変更は `ssv set` を使用する。鍵ローテーション要求時のみユーザー確認後に `ssv remove` して再生成する |
| `ssh -T` で `Permission denied` が返る | 公開鍵が GitHub に登録されているか確認する。`ssv show <HOST>` で鍵を再表示して照合する |
| `ssv audit` で不整合が報告される | 有効な管理設定の接続値は `ssv set` で修正する。必須フィールドが壊れた設定は信頼できるバックアップから正規形を復元する。鍵ペア欠損・破損時は設定復元後、ユーザー確認を得て `ssv remove` と再生成を行う |
| `ssv init` が失敗する | `~/.ssh` のパーミッションを確認する。`700` でなければならない |
