# Render でバックエンドをデプロイする手順

Rust APIを Render の Web Service としてデプロイ。
DBはSupabase(PostgreSQL)を使用。

---

## 1. 前提

- **DB**: Supabase の Postgres。[Supabase のデプロイ手順](./supabase.md) を参照し、**Settings → Database** で「Connection string」の **URI** をコピーしておく。
- **フロント**: デプロイ後、Vercelの環境変数に `NEXT_PUBLIC_API_BASE_URL` を設定。値は、Render の URL を設定。

---

## 2. Render で Web Service を作成

1. [Render](https://render.com) にログインし、**Dashboard** → **New +** → **Web Service** を選択。
2. このリポジトリ（GitHub / GitLab）を接続。
3. 次のように設定。

| 項目 | 値 |
|------|-----|
| **Name** | `mj-api` |
| **Region** | 任意 |
| **Root Directory** | `backend` |
| **Runtime** | **Docker** |
| **Dockerfile Path** | `Dockerfile` |
| **Instance Type** | **Free**（無料枠。15分でスリープする） |

4. **Environment Variables** で次を追加し、**Render から Supabase にアクセスできるようにする**。

| Key | Value |
|-----|-------|
| **DATABASE_URL** | Supabase の Postgres 接続文字列（URI） |

   - Supabase ダッシュボード: **Settings** → **Database** → **Connection string** の **URI** をコピー。
   - `<YOUR-PASSWORD>` をプロジェクト作成時の DB パスワードに置き換え。
   - **Transaction mode**（ポート 6543）の URI を使う。

5. **Create Web Service** で作成する。

**既存の Web Service に DB を繋ぐ場合**: Dashboard で該当サービスを開く → **Environment** → **Add Environment Variable** で `DATABASE_URL` を追加 → **Save Changes** で再デプロイが走る。

---

## 3. デプロイ後

- ビルド・デプロイが終わると、**https://mj-api-xxxx.onrender.com** のような URL が表示される。
- この URL を Vercel の **Environment Variables** の `NEXT_PUBLIC_API_BASE_URL` に設定し、必要ならフロントを再デプロイする。

---

## 4. 注意（無料枠）

- **15 分間アクセスがないとスリープ**する。次のリクエストで復帰するまで数十秒〜約1分かかることがある。

---

## まとめ

| 項目 | 内容 |
|------|------|
| Root Directory | `backend` |
| Runtime | Docker |
| 環境変数 | `DATABASE_URL` = Supabase の接続文字列 |
| フロント | Vercel の `NEXT_PUBLIC_API_BASE_URL` に Render の URL を設定 |
