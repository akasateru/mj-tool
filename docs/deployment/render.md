# Render でバックエンドをデプロイする手順

バックエンド（Rust API）を Render の Web Service としてデプロイし、**DB は Supabase** を使う構成です。

---

## 1. 前提

- **DB**: Supabase の Postgres。[Supabase のデプロイ手順](./supabase.md) を参照し、**Settings → Database** で「Connection string」の **URI** をコピーしておく。
- **フロント**: Vercel など別ホスト。デプロイ後に `NEXT_PUBLIC_API_BASE_URL` に Render の URL を設定する。

---

## 2. Render で Web Service を作成

1. [Render](https://render.com) にログインし、**Dashboard** → **New +** → **Web Service** を選択。
2. このリポジトリ（GitHub / GitLab）を接続する。
3. 次のように設定する。

| 項目 | 値 |
|------|-----|
| **Name** | 任意（例: `mj-api`） |
| **Region** | 希望のリージョン |
| **Root Directory** | `backend` |
| **Runtime** | **Docker** |
| **Dockerfile Path** | `Dockerfile`（Root が `backend` なのでそのまま） |
| **Instance Type** | **Free**（無料枠。15分でスリープする） |

4. **Environment Variables** で次を追加する。

| Key | Value |
|-----|-------|
| **DATABASE_URL** | Supabase の Postgres 接続文字列（URI） |

5. **Create Web Service** で作成する。

---

## 3. デプロイ後

- ビルド・デプロイが終わると、**https://mj-api-xxxx.onrender.com** のような URL が表示される。
- この URL を Vercel の **Environment Variables** の `NEXT_PUBLIC_API_BASE_URL` に設定し、必要ならフロントを再デプロイする。

---

## 4. 注意（無料枠）

- **15 分間アクセスがないとスリープ**する。次のリクエストで復帰するまで数十秒〜約1分かかることがある。
- 常時稼働させたい場合は有料プラン（Starter など）を検討する。

---

## まとめ

| 項目 | 内容 |
|------|------|
| Root Directory | `backend` |
| Runtime | Docker |
| 環境変数 | `DATABASE_URL` = Supabase の接続文字列 |
| フロント | Vercel の `NEXT_PUBLIC_API_BASE_URL` に Render の URL を設定 |

フロントのデプロイ手順は README などを参照してください。
