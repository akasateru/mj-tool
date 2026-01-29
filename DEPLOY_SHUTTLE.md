# Shuttle でバックエンドをデプロイする手順

バックエンド（Rust API）を Shuttle にデプロイし、**DB は Supabase** を使う構成です。Shuttle の組み込み Postgres は使わず、Supabase の接続文字列をシークレットで渡します。

---

## 1. 前提

- **DB**: Supabase の Postgres を使います。Supabase ダッシュボードの **Settings → Database** で「Connection string」の **URI** をコピーします（`postgresql://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres` のような形式）。
- **フロント**: Vercel など別ホスト。デプロイ後に `NEXT_PUBLIC_API_BASE_URL` に Shuttle の URL を設定します。

---

## 2. Shuttle にシークレットを設定する

Shuttle では **Secrets** で `DATABASE_URL` を渡します。

### 方法 A: Secrets.toml（デプロイ時にアップロード）

プロジェクトの **backend/mj-api** 直下に `Secrets.toml` を作成します。

```toml
DATABASE_URL = "postgresql://postgres.[ref]:[YOUR-PASSWORD]@aws-0-[region].pooler.supabase.com:6543/postgres"
```

- Supabase の接続文字列に置き換えてください。パスワードに `#` や `@` が含まれる場合はクォートで囲みます。
- **このファイルは .gitignore に含めてください**（`Secrets*.toml` を追加済みです）。

デプロイ時は次のように指定できます。

```bash
cd backend
cargo shuttle deploy --features shuttle --secrets Secrets.toml
```

### 方法 B: Shuttle ダッシュボード

[Shuttle](https://console.shuttle.dev) でプロジェクトを開き、**Secrets** から `DATABASE_URL` を追加・編集できます。デプロイ後もここで変更可能です。

---

## 3. デプロイ

```bash
cd backend
cargo shuttle deploy --features shuttle
```

初回は `cargo install cargo-shuttle` で CLI を入れ、`shuttle login` でログインしてください。

- **Secrets.toml** を使う場合: 上記のとおり `--secrets Secrets.toml` を付けてデプロイ。
- **ダッシュボードで DATABASE_URL を設定済み**の場合: `--secrets` は不要です。

デプロイが終わると `https://xxx.shuttleapp.rs` のような URL が表示されます。この URL を Vercel の `NEXT_PUBLIC_API_BASE_URL` に設定します。

---

## 4. ローカルで Shuttle 向けに試す場合

Supabase の接続文字列を用意し、**backend/mj-api** に `Secrets.dev.toml` を置きます。

```toml
DATABASE_URL = "postgresql://..."
```

その後:

```bash
cd backend
cargo shuttle run --features shuttle
```

`Secrets.dev.toml` も .gitignore 対象（`Secrets*.toml`）です。

---

## まとめ

| 項目 | 内容 |
|------|------|
| DB | Supabase（Postgres）。Shuttle の組み込み DB は使わない |
| 接続文字列 | Shuttle の Secrets で `DATABASE_URL` を設定 |
| デプロイ | `cargo shuttle deploy --features shuttle`（必要なら `--secrets Secrets.toml`） |
| フロント | Vercel の `NEXT_PUBLIC_API_BASE_URL` に Shuttle の URL を設定 |

フロントのデプロイ手順は README または Vercel 向けのドキュメントを参照してください。
