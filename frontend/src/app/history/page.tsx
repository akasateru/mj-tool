"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { apiListHands } from "@/lib/api";
import type { HandSummary } from "@/lib/types";

export default function HistoryPage() {
  const [q, setQ] = useState("");
  const [items, setItems] = useState<HandSummary[]>([]);
  const [message, setMessage] = useState<string | null>(null);

  async function load() {
    setMessage(null);
    const res = await apiListHands({ q: q.trim() || undefined, limit: 100 });
    if (!res.ok) {
      setMessage(res.message ?? "取得に失敗しました");
      return;
    }
    setItems(res.data);
  }

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">履歴</h1>

      <div className="flex flex-col gap-2 sm:flex-row">
        <input
          className="w-full rounded-xl border border-zinc-200 bg-white px-3 py-2 text-sm"
          placeholder="検索（メモ/タグ/日時）"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <button
          className="rounded-xl bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-800"
          onClick={load}
        >
          検索
        </button>
      </div>

      {message ? <div className="text-sm text-red-600">{message}</div> : null}

      <div className="rounded-2xl border border-zinc-200 bg-white">
        {items.length === 0 ? (
          <div className="p-4 text-sm text-zinc-600">
            まだ保存された履歴がありません。
          </div>
        ) : (
          <ul className="divide-y divide-zinc-100">
            {items.map((h) => (
              <li key={h.id} className="p-4">
                <Link className="block hover:underline" href={`/history/${h.id}`}>
                  <div className="flex items-baseline justify-between gap-3">
                    <div className="font-medium">
                      {h.display_hand_points}（合計 {h.total_to_winner}）
                    </div>
                    <div className="text-xs text-zinc-500">{h.created_at}</div>
                  </div>
                  {h.memo ? (
                    <div className="mt-1 text-sm text-zinc-700">{h.memo}</div>
                  ) : null}
                  {h.tags.length ? (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {h.tags.map((t) => (
                        <span
                          key={t}
                          className="rounded-full border border-zinc-200 bg-zinc-50 px-2 py-0.5 text-xs text-zinc-700"
                        >
                          {t}
                        </span>
                      ))}
                    </div>
                  ) : null}
                </Link>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

