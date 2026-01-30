# Supabase 利用のベストプラクティス

このプロジェクトで Supabase（DB）を使う際に押さえておきたいポイントをまとめています。公式ドキュメントや一般的な運用をベースにしています。

---

## 1. セキュリティ

### Row Level Security (RLS)

- **public スキーマのテーブルには必ず RLS を有効にする**  
  RLS が無いと、anon キーで全テーブルへの読み書きが可能になります。
- **ポリシーは「誰がどの行にアクセスできるか」を明示する**  
  例: `auth.uid() = user_id` で「自分の行だけ」に限定。
- **フロントから DB に直アクセスする場合は anon キー + RLS**  
  JWT とポリシーでアクセスが制御されるため、anon キーはフロントに含めてよいです。
- **service role キーは絶対にフロントに載せない**  
  RLS をバイパスするため、バックエンド専用・環境変数で渡す・リポジトリに含めない。

### このプロジェクトでの扱い

- バックエンド（Render）は **DB 接続文字列（postgres URI）** で接続しており、RLS をバイパスする運用です。  
  そのため **API 側で `user_id` を検証する**必要があります（ヘッダーや JWT など）。
- Supabase Auth を導入する場合は、フロントから Data API を使う場合に anon キー + RLS が有効になります。

---

## 2. 接続方法の選び方

| 接続先 | 用途 | 推奨 |
|--------|------|------|
| **Transaction mode（ポート 6543）** | サーバーレス・短命プロセス（Render の Web Service など） | ✅ このプロジェクトの API 用 |
| **Session mode（ポート 5432）** | 永続的なサーバーで IPv4 が必要な場合 | 必要に応じて |
| **Direct（db.xxx.supabase.co:5432）** | マイグレーション・pg_dump・バックアップ | 管理作業向け（IPv6 が前提のことが多い） |

- **API サーバー（Render）**: 接続数が増えやすいため **Transaction mode（Pooler）** の URI を使う。
- **Transaction mode では prepared statement が使えない**場合があるため、ドライバ側で無効化する設定が必要なことがあります（sqlx などは接続オプションで対応可能）。
- **SSL は有効のまま**接続する（ダッシュボードの接続文字列はデフォルトで SSL 想定）。

---

## 3. マイグレーション

- **スキーマ変更は必ずマイグレーションで行う**  
  手動で SQL Editor だけ実行すると、本番とローカルで差分が出ます。
- **ファイルは `supabase/migrations/` に番号付きで配置**  
  例: `0001_init.sql`, `0002_add_foo.sql`。適用順が名前で決まります。
- **本番への反映は `supabase db push`（CLI）を推奨**  
  同じマイグレーション履歴を共有できるため。  
  やむを得ず SQL Editor で流す場合は、どのファイルを実行したか記録しておく。
- **DB パスワードや接続文字列はリポジトリに含めない**  
  `.env` や Render の Environment で注入し、`.env.example` にはプレースホルダのみ記載。

---

## 4. RLS のパフォーマンス（フロント直アクセス時）

- **RLS の条件に使うカラムにはインデックスを張る**  
  例: `(user_id, created_at DESC)`。張らないとフルスキャンになりやすいです。
- **`auth.uid()` を多用するポリシーではキャッシュを検討**  
  公式では `(select auth.uid())` のようにサブクエリにするとプランナーがキャッシュしやすくなるとされています。
- **RLS は「守り」用。絞り込みはクエリ側でも行う**  
  `WHERE user_id = ?` をアプリで付け、RLS は二重のチェックとして扱うと安全です。

---

## 5. 環境変数・シークレット

| 変数 | 使う場所 | 注意 |
|------|----------|------|
| **DATABASE_URL**（postgres URI） | バックエンド（Render）・ローカル .env | 本番パスワードを含むため Git に載せない |
| **SUPABASE_URL** | フロント（Supabase クライアント用） | 公開してよい |
| **SUPABASE_ANON_KEY** | フロント | 公開してよい（RLS 有効が前提） |
| **SUPABASE_SERVICE_ROLE_KEY** | バックエンドのみ（必要な場合） | 絶対にフロント・公開しない |

このプロジェクトでは、現状バックエンドは **DATABASE_URL のみ**使用し、Supabase の anon/service キーは使っていません（DB に直接接続する構成）。

---

## 6. 参考リンク

- [Securing your data](https://supabase.com/docs/guides/database/secure-data) — anon / service role、RLS の考え方
- [Connect to Postgres](https://supabase.com/docs/guides/database/connecting-to-postgres) — 接続方法と Pooler の選び方
- [RLS Performance and Best Practices](https://supabase.com/docs/guides/troubleshooting/rls-performance-and-best-practices-Z5Jjwv) — ポリシーとインデックスのチューニング

プロジェクト固有のデプロイ手順は [Supabase にデプロイする手順](../deployment/supabase.md) を参照してください。
