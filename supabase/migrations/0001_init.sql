-- Supabase(Postgres)用スキーマ案（MVP）
-- - hands: Fact/結果の保存（calc_version, rule_set_id）
-- - games / hand_attachments / ai_comments は拡張で追加予定

create table if not exists public.hands (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id),
  created_at timestamptz not null default now(),
  rule_set_id text not null,
  calc_version text not null,
  fact_json jsonb not null,
  result_json jsonb not null,
  memo text,
  tags text[] not null default '{}'
);

create index if not exists hands_user_created_at on public.hands(user_id, created_at desc);
create index if not exists hands_user_memo on public.hands using gin (to_tsvector('simple', coalesce(memo, '')));
create index if not exists hands_user_tags on public.hands using gin (tags);

alter table public.hands enable row level security;

-- 自分の履歴のみ参照可能
create policy "hands_select_own"
on public.hands
for select
using (auth.uid() = user_id);

create policy "hands_insert_own"
on public.hands
for insert
with check (auth.uid() = user_id);

create policy "hands_update_own"
on public.hands
for update
using (auth.uid() = user_id)
with check (auth.uid() = user_id);

create policy "hands_delete_own"
on public.hands
for delete
using (auth.uid() = user_id);

