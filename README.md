# 麻雀×関数型ドメインモデリング（MVP）

- **frontend**: Next.js（入力UI / 履歴UI）
- **backend**: Rust（Validation + Pure計算 + 永続化API）

## 起動（ローカル: SQLite）

### 1) Rust API

```bash
cd backend
cargo run -p mj-api
```

- デフォルトで `./mj.sqlite`（SQLite）に保存します
- ポートは `8080`（変更する場合は `PORT`）

### 2) Next.js

```bash
cd frontend
npm run dev
```

環境変数（任意）:

```bash
export NEXT_PUBLIC_API_BASE_URL="http://localhost:8080"
```

## API（MVP）

- `POST /calc`: 入力（Fact）→ Validation → Pure計算
- `POST /hands`: 計算して保存
- `GET /hands`: 履歴一覧（`q` でメモ/タグ検索）
- `GET /hands/:id`: 履歴詳細
- `POST /hands/:id/recalc`: 現行ロジックで再計算して差分確認

## 仕様上の注意（MVP固定）

- **役判定なし**: 翻は手入力
- **符計算なし**: 符は手入力
- **本場**: 1本場=+300（ツモ: 各家+100 / ロン: 放銃者+300）
- **供託**: 1本=+1000（アガり者が総取り）
- **切り上げ満貫なし**
- **数え役満なし**（13翻以上は三倍満止まり）
- **ダブル役満なし**

