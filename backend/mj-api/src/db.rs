use crate::state::DbPool;

/// Supabase本番スキーマ（RLS/uuid/jsonb等）は supabase/migrations を参照。
pub async fn init_db(db: &DbPool) -> anyhow::Result<()> {
    const SQL: &str = r#"
CREATE TABLE IF NOT EXISTS hands (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  rule_set_id TEXT NOT NULL,
  calc_version TEXT NOT NULL,
  fact_json TEXT NOT NULL,
  result_json TEXT NOT NULL,
  memo TEXT,
  tags_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_hands_user_created_at ON hands(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_hands_user_memo ON hands(user_id, memo);
"#;
    sqlx::query(SQL).execute(&db.0).await?;
    Ok(())
}
