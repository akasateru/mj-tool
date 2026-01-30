# Supabase にデプロイする手順

このプロジェクトでは **DB を Supabase（Postgres）** で運用します。フロントは Vercel、API は Render から Supabase に接続する構成です。

---

## 1. Supabase プロジェクトの作成

1. [Supabase](https://supabase.com) にログインし、**New project** をクリック。
2. **Organization** を選択（なければ作成）。
3. 次を設定する。

   | 項目 | 説明 |
   |------|------|
   | **Name** | 任意（例: `mj-tool`） |
   | **Database Password** | 強めのパスワードを設定し、控えておく |
   | **Region** | フロント・API に近いリージョン（例: Northeast Asia (Seoul)） |

4. **Create new project** で作成。数分かかることがあります。

---

## 2. マイグレーションの適用

### 方法 A: ダッシュボードの SQL Editor

1. Supabase ダッシュボードでプロジェクトを開く。
2. 左メニュー **SQL Editor** を開く。
3. **New query** を選択し、`supabase/migrations/0001_init.sql` の内容を貼り付ける。
4. **Run** で実行する。

### 方法 B: Supabase CLI（ローカルでリンクして push）

```bash
# CLI のインストール（未導入の場合）
# npm: npm i -g supabase
# または https://supabase.com/docs/guides/cli を参照

# ログイン
supabase login

# プロジェクトとリンク（ダッシュボードの Settings → General の Reference ID を使用）
cd /path/to/2026_tech_challenge
supabase link --project-ref <PROJECT_REF>

# マイグレーションをリモートに反映
supabase db push
```

初回は `supabase/migrations/0001_init.sql` が適用されます。

---

## 3. 接続情報の取得

1. ダッシュボードで **Settings** → **Database** を開く。
2. **Connection string** の **URI** をコピーする。
   - プレースホルダ `<YOUR-PASSWORD>` を、プロジェクト作成時に設定した **Database Password** に置き換える。
3. この URI を次のように使う。
   - **Render**: Web Service の **Environment** に `DATABASE_URL` として設定（[Render のデプロイ手順](./render.md) 参照）。
   - **ローカル**: `.env` の `DATABASE_URL` に設定（本番 DB に繋ぐ場合のみ。通常は SQLite で十分）。

**例（マスクした URI）**

```
postgresql://postgres.[ref]:[YOUR-PASSWORD]@aws-0-ap-northeast-1.pooler.supabase.com:6543/postgres
```

- **Transaction mode**（ポート 6543）を API 用に使うのがおすすめです。
- **Session mode**（ポート 5432）は接続数が多くなりやすいため、必要に応じて使い分けてください。

---

## 4. 動作確認

- **API（Render）**: `DATABASE_URL` に上記 URI を設定して再デプロイし、`/health` や `/hands` にアクセスして確認。
- **フロント（Vercel）**: `NEXT_PUBLIC_API_BASE_URL` に Render の URL を設定していれば、本番 DB 経由で履歴などが表示されます。

---

## 補足

| 項目 | 内容 |
|------|------|
| **マイグレーション** | `supabase/migrations/0001_init.sql` で `hands` テーブルと RLS を作成 |
| **RLS** | `auth.uid() = user_id` で自分の行のみアクセス可能（Supabase Auth 利用時） |
| **API からの接続** | Render の API は **service_role** 相当の接続で DB にアクセスする想定。RLS をバイパスする場合はサービスロールキー／接続文字列を利用し、アプリ側で user_id を検証する必要があります。 |

現在のバックエンド（mj-api）は、起動時に `init_db` で自前の `hands` スキーマも作成します。Supabase 側のマイグレーション（`user_id uuid references auth.users(id)` など）と揃えたい場合は、バックエンドのスキーマや `user_id` の扱いを合わせて調整してください。
