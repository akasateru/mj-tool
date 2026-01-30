# 麻雀点数計算ツール

手入力で親/子・ツモ/ロン・翻・符・本場・供託などを入力すると、打点と点数移動を計算します。計算結果は保存でき、履歴一覧・詳細の参照、メモ・タグでの検索ができます。

- **本番**: [https://mj-tool.vercel.app/](https://mj-tool.vercel.app/)（Vercel）
- **ドキュメント**: [docs/](docs/README.md) にデプロイ手順・設計・麻雀ルールをまとめています。

---

## クイックスタート

```bash
cp .env.example .env
set -a && source .env && set +a
cd backend && cargo run -p mj-api    # 別ターミナルで
cd frontend && npm run dev           # 別ターミナルで
```

ブラウザで [http://localhost:3000](http://localhost:3000) を開き、`/calc` で計算、`/history` で履歴を確認できます。

---

## デプロイ環境

| 役割 | サービス | 手順 |
|------|----------|------|
| Frontend | Vercel | [docs/deployment/vercel.md](docs/deployment/vercel.md) |
| Backend | Render | [docs/deployment/render.md](docs/deployment/render.md) |
| DB | Supabase | [docs/deployment/supabase.md](docs/deployment/supabase.md) |

---

## 麻雀ルール（このアプリでの固定仕様）

- **対象** … 4人麻雀、赤あり
- **本場** … 1本場 = +300点（ツモなら各家 +100、ロンなら放銃者のみ +300）
- **供託** … 1本 = +1000点（アガり者が総取り）
- **打点上限** … 切り上げ満貫なし、数え役満なし（13翻以上は三倍満止まり）、ダブル役満なし

詳細は [docs/reference/mahjong-rules.md](docs/reference/mahjong-rules.md) を参照。
