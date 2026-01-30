# Vercel でフロントエンドをデプロイする手順

フロントエンド（Next.js）を Vercel にデプロイし、**API は Render** に接続する構成。

---

## 1. 前提

- **API**: バックエンドを Render にデプロイ済みであること。[Render のデプロイ手順](./render.md) を参照。
- Render の URL（例: `https://mj-api-xxxx.onrender.com`）を控えておく。デプロイ後に **Environment Variables** でフロントに渡す。

---

## 2. Vercel でプロジェクトを作成

1. [Vercel](https://vercel.com) にログイン。
2. **Add New…** → **Project** を選択。
3. **Import Git Repository** で、このリポジトリ（GitHub）を接続。
4. プロジェクト設定で次を指定。

| 項目 | 値 |
|------|-----|
| **Framework Preset** | Next.js |
| **Root Directory** | `frontend` |
| **Build Command** | 未指定（`next build` が使われる） |
| **Output Directory** | 未指定 |

5. **Environment Variables** で次を追加。

| Key | Value |
|-----|-------|
| **NEXT_PUBLIC_API_BASE_URL** | Render の API URL（例: `https://mj-api-xxxx.onrender.com`）。 |

6. **Deploy** でデプロイを開始する。

---

## 3. デプロイ後

- ビルドが完了すると、`https://xxxx.vercel.app` のような URL が発行される。
- カスタムドメインを割り当てる場合は、Vercel のプロジェクト **Settings** → **Domains** から設定。

---

## まとめ

| 項目 | 内容 |
|------|------|
| Root Directory | `frontend` |
| 環境変数 | `NEXT_PUBLIC_API_BASE_URL` = Render の API URL |
| フレームワーク | Next.js（`frontend/vercel.json` で `framework: "nextjs"` を指定済み） |
