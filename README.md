**1. 環境変数（初回のみ）**

```bash
cp .env.example .env
```

**2. 起動方法**

```bash
set -a && source .env && set +a
cd backend && cargo run -p mj-api    # 別ターミナルで
cd frontend && npm run dev          # 別ターミナルで
```

**本番**: [https://mj-tool.vercel.app/](https://mj-tool.vercel.app/)（Vercel）。手順は [DEPLOY_VERCEL.md](./DEPLOY_VERCEL.md) を参照。API は Shuttle や Render など別ホストを想定しています。

## 麻雀ルールについて

卓によってルールが揺れるので、このプロジェクトでは次のように固定しています。

- **対象** … 4人麻雀、赤あり
- **本場** … 1本場 = +300点（ツモなら各家 +100、ロンなら放銃者のみ +300）
- **供託** … 1本 = +1000点（アガり者が総取り）
- **打点上限** … 切り上げ満貫なし、数え役満なし（13翻以上は三倍満止まり）、ダブル役満なし
