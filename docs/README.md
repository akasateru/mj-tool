# ドキュメント

このプロジェクトのドキュメント一覧です。

## デプロイ

本番環境の構築手順です。

| ドキュメント | 内容 |
|-------------|------|
| [Render（バックエンド）](./deployment/render.md) | Rust API を Render の Web Service としてデプロイする手順 |
| [Supabase（DB）](./deployment/supabase.md) | Supabase プロジェクト作成・マイグレーション・接続情報の取得 |

フロントは Vercel にデプロイする想定です（Vercel とリポジトリを連携し、`frontend/` をルートに設定）。

---

## 設計・背景

| ドキュメント | 内容 |
|-------------|------|
| [関数型ドメインモデリング](./design/functional-domain-modeling.md) | 設計の背景（Fact / Validation / Pure / Effects の分離） |

---

## 参照

| ドキュメント | 内容 |
|-------------|------|
| [麻雀ルール](./reference/mahjong-rules.md) | このアプリで採用している麻雀ルールの固定仕様 |
| [Supabase ベストプラクティス](./reference/supabase-best-practices.md) | Supabase 利用時のセキュリティ・接続・マイグレーションのポイント |
