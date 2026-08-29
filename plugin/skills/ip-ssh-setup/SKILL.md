---
name: ip-ssh-setup
description: ユーザーが IPアドレス とユーザー名を渡して、次回から ssh <ユーザー名> で接続できるよう ssv でホストを登録したいと求めた場合に使用する。
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
| `ssv generate` でホストが既に存在する | ユーザーに確認のうえ `ssv remove <USERNAME>` を実行し再生成する |
| `ssv audit` で不整合が報告される | 指摘内容に従い、不足コンポーネントのみ再生成する |
