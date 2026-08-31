---
name: ip-ssh-setup
description: ssv を使って IP アドレスとユーザー名から直接 SSH 接続できる管理ホストを作成する。
compatibility: ssv、ssh、および接続先へのネットワーク接続が必要
disable-model-invocation: true
argument-hint: <IP> <USERNAME>
---

# IP指定での SSH セットアップ via ssv

IPアドレスとユーザー名を受け取り、`ssh <USERNAME>` で接続できる管理ホストを作成する。

## 入力

| 入力 | 取得方法 |
|---|---|
| IPアドレス | コマンド引数。未指定の場合はユーザーに確認する |
| ユーザー名 | コマンド引数。未指定の場合はユーザーに確認する |

## 手順

### 1. Bootstrap（初回のみ）

```bash
ssv init
```

### 2. ホスト登録

ホスト識別子にユーザー名を使うことで `ssh <USERNAME>` が成立する。

```bash
ssv generate <USERNAME> -n <IP> -u <USERNAME>
```

生成されるファイル:
- `~/.ssh/conf.d/<USERNAME>.conf` — `HostName <IP>`, `User <USERNAME>` を含むホスト設定
- `~/.ssh/id_ed25519_<USERNAME>` — 秘密鍵
- `~/.ssh/id_ed25519_<USERNAME>.pub` — 公開鍵

### 3. 公開鍵をリモートに転送

リモートが指定のIP経由で到達可能な状態で実行する。パスワードを一度求められる。

```bash
ssv authorize <USERNAME>
```

### 4. 接続確認

```bash
ssh <USERNAME>
```

接続できれば完了。

## 障害対応

| 症状 | 対応 |
|---|---|
| `ssv authorize` で接続拒否される | IP が正しく、到達可能か確認する |
| `ssh <USERNAME>` で `Permission denied` | `ssv authorize <USERNAME>` を再実行する |
| `ssv generate` でホストが既に存在する | `ssv audit` で既存状態を確認する。正常であれば `ssv show <USERNAME>` で鍵を確認する。IP/ユーザー変更時は `ssv set` を使用し、明示的な鍵更新時のみユーザー確認後に `ssv remove` して再生成する |
| `ssv audit` で不整合が報告される | 有効な管理設定の接続値は `ssv set` で修正する。必須フィールドが壊れた設定は信頼できるバックアップから正規形を復元する。鍵ペア不整合・欠損時は設定復元後、ユーザー確認を得て `ssv remove` と再生成を行う |
