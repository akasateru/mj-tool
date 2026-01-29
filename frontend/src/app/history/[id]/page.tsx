"use client";

import { use, useEffect, useState } from "react";
import Link from "next/link";
import { apiGetHand, apiRecalcHand } from "@/lib/api";
import type { HandRecord, RecalcResponse } from "@/lib/types";

export default function HistoryDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const [item, setItem] = useState<HandRecord | null>(null);
  const [recalc, setRecalc] = useState<RecalcResponse | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function load() {
    setMessage(null);
    const res = await apiGetHand(id);
    if (!res.ok) {
      setMessage(res.message ?? "取得に失敗しました");
      return;
    }
    setItem(res.data);
  }

  async function runRecalc() {
    setMessage(null);
    const res = await apiRecalcHand(id);
    if (!res.ok) {
      setMessage(res.message ?? "再計算に失敗しました");
      return;
    }
    setRecalc(res.data);
  }

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <h1 className="text-lg font-semibold">履歴詳細</h1>
        <Link className="text-sm hover:underline" href="/history">
          一覧へ
        </Link>
      </div>

      {message ? <div className="text-sm text-red-600">{message}</div> : null}

      {!item ? (
        <div className="text-sm text-zinc-600">読み込み中...</div>
      ) : (
        <div className="space-y-4">
          <section className="rounded-2xl border border-zinc-200 bg-white p-4">
            <div className="text-sm text-zinc-600">
              {item.result.display_hand_points.includes("/")
                ? "結果（親から/子から）"
                : "結果"}
            </div>
            <div className="mt-1 text-3xl font-semibold">
              {item.result.display_hand_points}
            </div>
            <div className="mt-2 text-sm text-zinc-700">
              合計受取（本場・供託込み）:{" "}
              <span className="font-medium">{item.result.totals.total_to_winner}</span>
            </div>
            <div className="mt-2 text-xs text-zinc-500">
              created_at: {item.created_at}
              <br />
              calc_version: {item.calc_version}
              <br />
              rule_set_id: {item.rule_set_id}
            </div>
          </section>

          <section className="rounded-2xl border border-zinc-200 bg-white p-4">
            <div className="text-sm font-medium">Fact</div>
            <pre className="mt-2 overflow-x-auto rounded-xl bg-zinc-50 p-3 text-xs">
              {JSON.stringify(item.fact, null, 2)}
            </pre>
          </section>

          <section className="rounded-2xl border border-zinc-200 bg-white p-4">
            <div className="flex items-center justify-between gap-2">
              <div className="text-sm font-medium">再計算</div>
              <button
                className="rounded-xl bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-800"
                onClick={runRecalc}
              >
                再計算して差分確認
              </button>
            </div>
            {recalc ? (
              <div className="mt-3 space-y-3 text-sm">
                <div>
                  same:{" "}
                  <span className={`font-medium ${recalc.same ? "text-green-700" : "text-red-700"}`}>
                    {String(recalc.same)}
                  </span>
                </div>
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                  <div>
                    <div className="text-xs text-zinc-500">stored</div>
                    <pre className="mt-1 overflow-x-auto rounded-xl bg-zinc-50 p-3 text-xs">
                      {JSON.stringify(recalc.stored_result.totals, null, 2)}
                    </pre>
                  </div>
                  <div>
                    <div className="text-xs text-zinc-500">current</div>
                    <pre className="mt-1 overflow-x-auto rounded-xl bg-zinc-50 p-3 text-xs">
                      {JSON.stringify(recalc.current_result.totals, null, 2)}
                    </pre>
                  </div>
                </div>
              </div>
            ) : (
              <div className="mt-2 text-sm text-zinc-600">
                ロジック更新後にここで差分表示します（同一バージョンならsame=true）。
              </div>
            )}
          </section>
        </div>
      )}
    </div>
  );
}

