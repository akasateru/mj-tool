**1. Backend（Rust API）**

```bash
cd backend
cargo run -p mj-api
```

ポートは `8080`（`PORT` で変更可）です。DB は `DATABASE_URL` を渡さなければ `sqlite://./mj.sqlite?mode=rwc` が使われます。

**2. Frontend（Next.js）**

```bash
cd frontend
npm ci
npm run dev
```

`NEXT_PUBLIC_API_BASE_URL` を指定しなければ `http://localhost:8080` を見に行きます。ブラウザで `http://localhost:3000/calc` を開くと入力画面、`/history` で履歴一覧になります。環境変数の一覧は `ENVIRONMENT.md` にまとめてあります。

**API の例**

- `POST /calc` … Fact を送ると Validation → Pure 計算の結果が返ります
- `POST /hands` … 計算してそのまま保存します
- `GET /hands` … 履歴一覧（`q` でメモ/タグ検索、`limit` 指定可）
- `GET /hands/:id` … 履歴詳細
- `POST /hands/:id/recalc` … 現行ロジックで再計算し、旧結果との差分を返します

ローカルでは `x-user-id` ヘッダでユーザーを識別します（省略時は `"local"`）。`GET /health` でヘルスチェックできます。

---

## 麻雀ルールについて

卓によってルールが揺れるので、このプロジェクトでは次のように固定しています。

- **対象** … 4人麻雀、赤あり
- **本場** … 1本場 = +300点（ツモなら各家 +100、ロンなら放銃者のみ +300）
- **供託** … 1本 = +1000点（アガり者が総取り）
- **打点上限** … 切り上げ満貫なし、数え役満なし（13翻以上は三倍満止まり）、ダブル役満なし

---

## 構成とこれから

関数型ドメインモデリングの「Pure と Effects の分離」に沿って、バックエンドを次のように分けています。

- `frontend/` … Next.js（`/calc` が入力、`/history` が一覧・詳細）です。ルートは `/calc` にリダイレクトします。
- `backend/mj-domain/` … **Fact の型定義、Validation、Pure 計算**です。DB や HTTP に依存せず、`ValidatedFact` と `RuleSet` から `CalcResult` を返すだけのクレートにしています。
- `backend/mj-api/` … **Effects の境界**です。HTTP を受け、Validation → Pure を呼び出し、その結果を DB に保存する責務を持ちます。
- `supabase/migrations/` … Supabase（Postgres）向けのスキーマ案です。`hands` の RLS など、`calc_version`・`rule_set_id`・`fact_json` を軸にし、**Fact と結果を永続化する**形にしています。

今後は、符計算補助（面子・待ち形から符を算出）、写真入力の補助、履歴への AI コメントなどもスコープに入れたいと思っています。いずれも「入力が Fact に正規化される」前提であれば、既存の Pure 計算とは切り分けたまま拡張できる想定です。

