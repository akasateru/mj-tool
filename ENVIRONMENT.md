# 環境変数

このリポジトリでは `.env*` が環境側のルールで扱いづらい場合があるため、必要な環境変数をここにまとめます。

## Frontend（Next.js）

- `NEXT_PUBLIC_API_BASE_URL`
  - 例: `http://localhost:8080`
  - 未設定の場合は `http://localhost:8080` を使用します

## Backend（Rust / mj-api）

- `PORT`
  - 例: `8080`
  - 未設定の場合は `8080`

# 
- `DATABASE_URL`
  - 例（SQLite）: `sqlite://./mj.sqlite?mode=rwc`
  - 未設定の場合は `sqlite://./mj.sqlite?mode=rwc`
