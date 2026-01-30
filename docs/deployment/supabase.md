# Supabase にデプロイする手順

 **DB は Supabase（Postgres）** 。フロントは Vercel、API は Render から Supabase に接続する構成。

---

## 1. Supabase プロジェクトの作成

1. [Supabase](https://supabase.com) にログインし、**New project** をクリック。
2. **Organization** を選択（なければ作成）。
3. 次を設定。

   | 項目 | 説明 |
   |------|------|
   | **Name** | `mj-tool` |
   | **Database Password** | 自動生成 |
   | **Region** | ortheast Asia (Seoul) |

4. **Create new project** 

---

## 2. Supabase CLI のインストールとログイン

```bash
cd /path/to/2026_tech_challenge
npm install
npx supabase login
```

---

## 3. プロジェクトをリンクしてマイグレーションを push

1. ダッシュボードの **Settings** → **General** で **Reference ID**（プロジェクト ID）をコピー。
2. リポジトリのルートで次を実行。

```bash
cd /path/to/2026_tech_challenge
npx supabase link --project-ref <PROJECT_REF>   # <PROJECT_REF> を Reference ID に置き換え
npx supabase db push
```

---

## 4. 接続情報の取得

1. ダッシュボードで **Settings** → **Database** を開く。
2. **Connection string** の **URI** をコピー。
   - プレースホルダ `<YOUR-PASSWORD>` を、プロジェクト作成時に設定した **Database Password** に置き換える。
3. この URI を次のように使う。
   - **Render**: Web Service の **Environment** に `DATABASE_URL` として設定。
   - **ローカル**: `.env` の `DATABASE_URL` に設定。

**例（マスクした URI）**

```
postgresql://postgres.[ref]:[YOUR-PASSWORD]@aws-0-ap-northeast-1.pooler.supabase.com:6543/postgres
```

- **Transaction mode**（ポート 6543）を API 用に使用。
- **Session mode**（ポート 5432）は接続数が多くなりやすいため、必要に応じて使い分け。

---

## 5. 動作確認

- **API（Render）**: `DATABASE_URL` に上記 URI を設定して再デプロイし、`/health` にアクセスして確認。
- **フロント（Vercel）**: `NEXT_PUBLIC_API_BASE_URL` に Render の URL を設定。

---

## 補足

| 項目 | 内容 |
|------|------|
| **マイグレーション** | `supabase/migrations/0001_init.sql` で `hands` テーブルと RLS を作成 |
| **RLS** | `auth.uid() = user_id` で自分の行のみアクセス可能（Supabase Auth 利用時） |
