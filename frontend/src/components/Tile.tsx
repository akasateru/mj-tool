"use client";

import { tileLabel, tileSuit, type TileCode } from "@/lib/tiles";

const SUIT_COLORS: Record<string, string> = {
  m: "bg-rose-50 border-rose-200 text-rose-900",   // 萬子
  p: "bg-sky-50 border-sky-200 text-sky-900",     // 筒子
  s: "bg-emerald-50 border-emerald-200 text-emerald-900", // 索子
  z: "bg-zinc-100 border-zinc-300 text-zinc-800", // 字牌
};

export function Tile({ code }: { code: TileCode | string }) {
  const suit = tileSuit(code);
  const label = tileLabel(code);
  const style = suit ? SUIT_COLORS[suit] : "bg-zinc-50 border-zinc-200 text-zinc-700";

  return (
    <span
      className={`inline-flex h-8 min-w-[1.75rem] items-center justify-center rounded border px-1 text-sm font-medium shadow-sm ${style}`}
      title={code}
    >
      {label}
    </span>
  );
}
