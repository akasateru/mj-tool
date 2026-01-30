"use client";

import { useMemo, useState } from "react";
import { apiAnalyzeHand, apiAnalyzeImage, apiCalc, apiCreateHand } from "@/lib/api";
import type {
  AnalyzeHandResponse,
  CalcResponse,
  Fact,
  ValidationError,
} from "@/lib/types";
import { HandArea } from "@/components/HandArea";
import { TilePool } from "@/components/TilePool";
import {
  handTilesToHandString,
  parseHandString,
  sortHandTiles,
  type TileCode,
} from "@/lib/tiles";

const FU_PRESETS = [20, 25, 30, 40, 50, 60, 70, 80, 90, 100, 110];
const HAN_OPTIONS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] as const;
const HONBA_OPTIONS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const RIICHI_STICKS_OPTIONS = [0, 1, 2, 3, 4];

const selectClassName =
  "cursor-pointer appearance-none rounded-xl border border-zinc-200 bg-white px-3 py-2 pr-10 text-sm transition-colors hover:border-zinc-300 focus:border-zinc-400 focus:outline-none focus:ring-2 focus:ring-zinc-200";
const selectStyle = {
  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%23717171'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'/%3E%3C/svg%3E")`,
  backgroundRepeat: "no-repeat" as const,
  backgroundPosition: "right 0.5rem center",
  backgroundSize: "1.25rem",
};

function errorText(e: ValidationError) {
  return e.detail;
}

export default function CalcPage() {
  const [fact, setFact] = useState<Fact>({
    winner_role: "non_dealer",
    win_method: "ron",
    han: 4,
    fu: 30,
    honba: 0,
    riichi_sticks: 0,
    discarder: "opponent1",
  });

  const [calc, setCalc] = useState<CalcResponse | null>(null);
  const [errors, setErrors] = useState<ValidationError[]>([]);
  const [memo, setMemo] = useState("");
  const [tagsText, setTagsText] = useState("");
  const tags = useMemo(
    () =>
      tagsText
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean),
    [tagsText],
  );

  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const [handTiles, setHandTiles] = useState<TileCode[]>([]);
  const handString = useMemo(() => handTilesToHandString(handTiles), [handTiles]);
  const [analyzeRiichi, setAnalyzeRiichi] = useState(false);
  const [analyzeTsumo, setAnalyzeTsumo] = useState(true);
  const [analyzeResult, setAnalyzeResult] = useState<AnalyzeHandResponse | null>(null);
  const [analyzeError, setAnalyzeError] = useState<string | null>(null);
  const [analyzing, setAnalyzing] = useState(false);

  const [imageFile, setImageFile] = useState<File | null>(null);
  const [imageAnalyzing, setImageAnalyzing] = useState(false);
  const [imageError, setImageError] = useState<string | null>(null);

  async function onAnalyze() {
    setAnalyzeError(null);
    setAnalyzeResult(null);
    setAnalyzing(true);
    try {
      const res = await apiAnalyzeHand({
        hand_string: handString.trim(),
        riichi: analyzeRiichi,
        tsumo: analyzeTsumo,
      });
      if (!res.ok) {
        let errMsg = "解析に失敗しました";
        if (res.message) {
          try {
            const body = JSON.parse(res.message) as { detail?: string };
            errMsg = body.detail ?? res.message;
          } catch {
            errMsg = res.message;
          }
        }
        // API が JSON を返さない場合などで "{}" になるのを避ける
        if (!errMsg || errMsg === "{}" || errMsg.trim() === "") {
          errMsg = "解析に失敗しました";
        }
        setAnalyzeError(errMsg);
        return;
      }
      setAnalyzeResult(res.data);
      setFact((f) => ({
        ...f,
        han: res.data.han,
        fu: res.data.fu,
        win_method: analyzeTsumo ? "tsumo" : "ron",
        discarder: analyzeTsumo ? null : (f.discarder ?? "opponent1"),
      }));
    } finally {
      setAnalyzing(false);
    }
  }

  async function onAnalyzeImage() {
    if (!imageFile) return;
    setImageError(null);
    setAnalyzeError(null);
    setImageAnalyzing(true);
    try {
      const res = await apiAnalyzeImage(imageFile, {
        riichi: analyzeRiichi,
        tsumo: analyzeTsumo,
      });
      if (!res.ok) {
        setImageError(res.message ?? "画像解析に失敗しました");
        return;
      }
      // 画像から解析は何か返ってきたら通す。手牌をセットし、解析できたときだけ翻・符を反映
      setHandTiles(parseHandString(res.data.hand_string ?? ""));
      setImageError(null);
      if (res.data.han != null && res.data.fu != null) {
        const han = res.data.han;
        const fu = res.data.fu;
        setAnalyzeResult({
          han,
          fu,
          yaku: res.data.yaku ?? [],
        });
        setFact((f) => ({
          ...f,
          han,
          fu,
          win_method: analyzeTsumo ? "tsumo" : "ron",
          discarder: analyzeTsumo ? null : (f.discarder ?? "opponent1"),
        }));
        setAnalyzeError(null);
      } else {
        setAnalyzeResult(null);
      }
    } finally {
      setImageAnalyzing(false);
    }
  }

  async function onCalc() {
    setMessage(null);
    const res = await apiCalc({
      ...fact,
      discarder: fact.win_method === "ron" ? fact.discarder ?? null : null,
    });
    if (!res.ok) {
      setCalc(null);
      setErrors(res.errors ?? []);
      setMessage(res.message ?? "計算に失敗しました");
      return;
    }
    setErrors([]);
    setCalc(res.data);
  }

  async function onSave() {
    if (!calc) return;
    setSaving(true);
    setMessage(null);
    try {
      const res = await apiCreateHand({
        fact: {
          ...fact,
          discarder: fact.win_method === "ron" ? fact.discarder ?? null : null,
        },
        memo: memo.trim() || undefined,
        tags: tags.length ? tags : undefined,
      });
      if (!res.ok) {
        setErrors(res.errors ?? []);
        setMessage(res.message ?? "保存に失敗しました");
        return;
      }
      window.location.href = `/history/${res.data.id}`;
    } finally {
      setSaving(false);
    }
  }

  const isRon = fact.win_method === "ron";

  return (
    <div className="space-y-6">
      <section className="rounded-2xl border border-zinc-200 bg-white p-4">
        <h1 className="text-lg font-semibold">点数計算（手入力）</h1>
        <p className="mt-1 text-sm text-zinc-600">
          翻・符はユーザー入力、または下の「画像から解析」「牌リストから解析」で自動算出。
        </p>

        <div className="mt-4 rounded-xl border border-zinc-100 bg-zinc-50 p-4">
          <div className="text-sm font-medium">画像から解析</div>
          <p className="mt-1 text-xs text-zinc-500">
            手牌の写真をアップロードすると、AI（OpenAI Vision）で牌を読み取り、翻・符・役を算出します。下のリーチ・ツモにチェックを入れてから実行してください。
          </p>
          <div className="mt-2 flex flex-wrap items-end gap-2">
            <input
              type="file"
              accept="image/jpeg,image/png,image/webp,image/heic"
              className="text-sm file:mr-2 file:rounded-xl file:border file:border-zinc-200 file:bg-white file:px-3 file:py-2 file:text-sm file:font-medium file:hover:bg-zinc-50"
              onChange={(e) => {
                const f = e.target.files?.[0];
                setImageFile(f ?? null);
                setImageError(null);
              }}
            />
            <button
              className="rounded-xl border border-zinc-300 bg-white px-3 py-2 text-sm font-medium hover:bg-zinc-50 disabled:opacity-50"
              onClick={onAnalyzeImage}
              disabled={imageAnalyzing || !imageFile}
            >
              {imageAnalyzing ? "解析中…" : "画像から解析"}
            </button>
          </div>
          {imageError && (
            <div className="mt-2 text-sm text-red-600">{imageError}</div>
          )}
        </div>

        <div className="mt-4 rounded-xl border border-zinc-100 bg-zinc-50 p-4">
          <div className="text-sm font-medium">牌リストから解析</div>
          <p className="mt-1 text-xs text-zinc-500">
            牌プールの牌をクリックまたはドラッグで手牌に追加。手牌内のドラッグで並べ替え、×ボタンで削除。14枚で解析できます。
          </p>
          <div className="mt-3">
            <div className="flex items-center justify-between gap-2 mb-1">
              <span className="text-xs font-medium text-zinc-500">手牌（14枚）</span>
              <button
                type="button"
                className="rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs font-medium text-zinc-600 hover:bg-zinc-50 disabled:opacity-50"
                onClick={() => setHandTiles([])}
                disabled={handTiles.length === 0}
              >
                手牌をリセット
              </button>
            </div>
            <HandArea
            handTiles={handTiles}
            onHandChange={(next) => setHandTiles(sortHandTiles(next))}
          />
          </div>
          <div className="mt-3">
            <div className="text-xs font-medium text-zinc-500 mb-1">牌プール（ドラッグで追加）</div>
            <TilePool
              handTiles={handTiles}
              onAddTile={(code) => {
                if (handTiles.length >= 14) return;
                if (handTiles.filter((t) => t === code).length >= 4) return;
                setHandTiles((prev) => sortHandTiles([...prev, code]));
              }}
            />
          </div>
          <div className="mt-3 flex flex-wrap items-end gap-2">
            <label className="flex items-center gap-1.5 text-sm shrink-0">
              <input
                type="checkbox"
                checked={analyzeRiichi}
                onChange={(e) => setAnalyzeRiichi(e.target.checked)}
              />
              リーチ
            </label>
            <label className="flex items-center gap-1.5 text-sm shrink-0">
              <input
                type="checkbox"
                checked={analyzeTsumo}
                onChange={(e) => setAnalyzeTsumo(e.target.checked)}
              />
              ツモ
            </label>
            <button
              className="rounded-xl border border-zinc-300 bg-white px-3 py-2 text-sm font-medium hover:bg-zinc-50 disabled:opacity-50 shrink-0"
              onClick={onAnalyze}
              disabled={analyzing || handTiles.length !== 14}
            >
              {analyzing ? "解析中…" : "解析"}
            </button>
          </div>
          {handString && (
            <p className="mt-2 text-xs text-zinc-500 font-mono truncate" title={handString}>
              文字列: {handString}
            </p>
          )}
          <div className="mt-3 border-t border-zinc-200 pt-3">
            <div className="text-xs font-medium text-zinc-500">解析結果</div>
            {analyzeError && (
              <div className="mt-1 text-sm text-red-600">{analyzeError}</div>
            )}
            {analyzeResult && !analyzeError && (
              <div className="mt-1 text-sm text-zinc-700">
                <span className="font-medium">{analyzeResult.han}翻</span>
                <span className="mx-1">/</span>
                <span className="font-medium">{analyzeResult.fu}符</span>
                {analyzeResult.yaku.length > 0 && (
                  <span className="ml-2">
                    （{analyzeResult.yaku.join("・")}）
                  </span>
                )}
                <span className="ml-2 text-zinc-500">→ 計算フォームに反映済み</span>
              </div>
            )}
            {!analyzeError && !analyzeResult && !analyzing && (
              <div className="mt-1 text-xs text-zinc-400">
                「解析」を押すと、ここに翻・符・役名が表示されます。
              </div>
            )}
          </div>
        </div>

        <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <div className="text-sm font-medium">親/子</div>
            <div className="mt-2 flex gap-2">
              <button
                className={`rounded-full px-3 py-1 text-sm border ${
                  fact.winner_role === "dealer"
                    ? "bg-zinc-900 text-white border-zinc-900"
                    : "bg-white border-zinc-200"
                }`}
                onClick={() => setFact((f) => ({ ...f, winner_role: "dealer" }))}
              >
                親
              </button>
              <button
                className={`rounded-full px-3 py-1 text-sm border ${
                  fact.winner_role === "non_dealer"
                    ? "bg-zinc-900 text-white border-zinc-900"
                    : "bg-white border-zinc-200"
                }`}
                onClick={() =>
                  setFact((f) => ({ ...f, winner_role: "non_dealer" }))
                }
              >
                子
              </button>
            </div>
          </div>

          <div>
            <div className="text-sm font-medium">ツモ/ロン</div>
            <div className="mt-2 flex gap-2">
              <button
                className={`rounded-full px-3 py-1 text-sm border ${
                  fact.win_method === "tsumo"
                    ? "bg-zinc-900 text-white border-zinc-900"
                    : "bg-white border-zinc-200"
                }`}
                onClick={() =>
                  setFact((f) => ({ ...f, win_method: "tsumo", discarder: null }))
                }
              >
                ツモ
              </button>
              <button
                className={`rounded-full px-3 py-1 text-sm border ${
                  fact.win_method === "ron"
                    ? "bg-zinc-900 text-white border-zinc-900"
                    : "bg-white border-zinc-200"
                }`}
                onClick={() =>
                  setFact((f) => ({
                    ...f,
                    win_method: "ron",
                    discarder: f.discarder ?? "opponent1",
                  }))
                }
              >
                ロン
              </button>
            </div>
          </div>

          <label className="block">
            <div className="text-sm font-medium">翻</div>
            <select
              className={`mt-2 w-full ${selectClassName}`}
              style={selectStyle}
              value={fact.han}
              onChange={(e) =>
                setFact((f) => ({ ...f, han: Number(e.target.value) }))
              }
            >
              {HAN_OPTIONS.map((han) => (
                <option key={han} value={han}>
                  {han}翻
                </option>
              ))}
            </select>
            <div className="mt-1 text-xs text-zinc-500">
              13翻以上は仕様により三倍満止まり（数え役満なし）
            </div>
          </label>

          <label className="block">
            <div className="text-sm font-medium">符</div>
            <select
              className={`mt-2 w-full ${selectClassName}`}
              style={selectStyle}
              value={fact.fu}
              onChange={(e) =>
                setFact((f) => ({ ...f, fu: Number(e.target.value) }))
              }
            >
              {FU_PRESETS.map((fu) => (
                <option key={fu} value={fu}>
                  {fu}符
                </option>
              ))}
            </select>
          </label>

          <label className="block">
            <div className="text-sm font-medium">本場</div>
            <select
              className={`mt-2 w-full ${selectClassName}`}
              style={selectStyle}
              value={fact.honba}
              onChange={(e) =>
                setFact((f) => ({ ...f, honba: Number(e.target.value) }))
              }
            >
              {HONBA_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}本場
                </option>
              ))}
            </select>
            <div className="mt-1 text-xs text-zinc-500">
              1本場=+300（ツモは各家+100、ロンは放銃者+300）
            </div>
          </label>

          <label className="block">
            <div className="text-sm font-medium">供託</div>
            <select
              className={`mt-2 w-full ${selectClassName}`}
              style={selectStyle}
              value={fact.riichi_sticks}
              onChange={(e) =>
                setFact((f) => ({
                  ...f,
                  riichi_sticks: Number(e.target.value),
                }))
              }
            >
              {RIICHI_STICKS_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}本
                </option>
              ))}
            </select>
            <div className="mt-1 text-xs text-zinc-500">
              1本=+1000（アガり者が総取り）
            </div>
          </label>

          {isRon ? (
            <div>
              <div className="text-sm font-medium">放銃者（ロンのみ）</div>
              <select
                className={`mt-2 w-full ${selectClassName}`}
                style={selectStyle}
                value={fact.discarder ?? "opponent1"}
                onChange={(e) =>
                  setFact((f) => ({
                    ...f,
                    discarder: e.target.value as Fact["discarder"],
                  }))
                }
              >
                <option value="opponent1">対面A</option>
                <option value="opponent2">対面B</option>
                <option value="opponent3">対面C</option>
              </select>
            </div>
          ) : (
            <div className="text-sm text-zinc-500">
              子ツモの「親払い」は、内訳では対面Aを親として表示します。
            </div>
          )}
        </div>

        <div className="mt-5 flex flex-wrap items-center gap-2">
          <button
            className="rounded-xl bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-800"
            onClick={onCalc}
          >
            計算
          </button>
          <button
            className="rounded-xl border border-zinc-200 bg-white px-4 py-2 text-sm font-medium hover:bg-zinc-50 disabled:opacity-50"
            onClick={onSave}
            disabled={!calc || saving}
          >
            保存
          </button>
          {message ? <div className="text-sm text-red-600">{message}</div> : null}
        </div>

        {errors.length ? (
          <div className="mt-4 rounded-xl border border-red-200 bg-red-50 p-3 text-sm text-red-700">
            <div className="font-medium">入力エラー</div>
            <ul className="mt-2 list-disc pl-5">
              {errors.map((e, i) => (
                <li key={i}>{errorText(e)}</li>
              ))}
            </ul>
          </div>
        ) : null}
      </section>

      <section className="rounded-2xl border border-zinc-200 bg-white p-4">
        <h2 className="text-base font-semibold">計算結果</h2>
        {!calc ? (
          <p className="mt-2 text-sm text-zinc-600">
            「計算」を押すと結果が表示されます。
          </p>
        ) : (
          <div className="mt-3 space-y-4">
            <div className="rounded-2xl border border-zinc-200 bg-zinc-50 p-4">
              <div className="text-sm text-zinc-600">表示用（基本打点）</div>
              <div className="mt-1 text-3xl font-semibold">
                {calc.result.display_hand_points}
              </div>
              <div className="mt-2 text-sm text-zinc-600">
                合計受取（本場・供託込み）:{" "}
                <span className="font-medium text-zinc-900">
                  {calc.result.totals.total_to_winner}
                </span>
              </div>
              <div className="mt-2 text-xs text-zinc-500">
                calc_version: {calc.calc_version} / rule_set_id:{" "}
                {calc.result.rule_set_id}
              </div>
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div className="rounded-2xl border border-zinc-200 p-4">
                <div className="text-sm font-medium">内訳（本場・供託）</div>
                <div className="mt-2 text-sm text-zinc-700">
                  手牌点: {calc.result.totals.hand_points_total_to_winner}
                  <br />
                  本場: {calc.result.totals.honba_points_total_to_winner}
                  <br />
                  供託: {calc.result.totals.riichi_points_total_to_winner}
                </div>
              </div>
              <div className="rounded-2xl border border-zinc-200 p-4">
                <div className="text-sm font-medium">メモ・タグ（保存用）</div>
                <input
                  className="mt-2 w-full rounded-xl border border-zinc-200 px-3 py-2 text-sm"
                  placeholder="例）帰省/南2局/鳴きあり"
                  value={memo}
                  onChange={(e) => setMemo(e.target.value)}
                />
                <input
                  className="mt-2 w-full rounded-xl border border-zinc-200 px-3 py-2 text-sm"
                  placeholder="タグ（カンマ区切り）例）帰省,南2"
                  value={tagsText}
                  onChange={(e) => setTagsText(e.target.value)}
                />
              </div>
            </div>

            <div className="rounded-2xl border border-zinc-200 p-4">
              <div className="text-sm font-medium">点数移動（payments）</div>
              <div className="mt-2 overflow-x-auto">
                <table className="w-full text-sm">
                  <thead className="text-left text-zinc-500">
                    <tr>
                      <th className="py-1">from</th>
                      <th className="py-1">to</th>
                      <th className="py-1">points</th>
                      <th className="py-1">reason</th>
                    </tr>
                  </thead>
                  <tbody>
                    {calc.result.payments.map((p, i) => (
                      <tr key={i} className="border-t border-zinc-100">
                        <td className="py-1">{p.from}</td>
                        <td className="py-1">{p.to}</td>
                        <td className="py-1">{p.points}</td>
                        <td className="py-1">{p.reason}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

