"use client";

import { parseHandString } from "@/lib/tiles";
import { Tile } from "./Tile";

type Props = {
  handString: string;
  className?: string;
};

/** 手牌文字列をパースして牌を横並びで表示 */
export function HandTiles({ handString, className = "" }: Props) {
  const tiles = parseHandString(handString);
  if (tiles.length === 0) return null;

  return (
    <div className={`flex flex-wrap items-center gap-1 ${className}`}>
      {tiles.map((code, i) => (
        <Tile key={`${code}-${i}`} code={code} />
      ))}
    </div>
  );
}
