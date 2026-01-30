"use client";

import { ALL_TILE_TYPES, type TileCode } from "@/lib/tiles";
import { Tile } from "./Tile";

type Props = {
  handTiles: TileCode[];
  onAddTile?: (code: TileCode) => void;
  onDragStart?: (code: TileCode) => void;
};

/** 牌プール：全34種。クリックまたはドラッグで手牌に追加（同種は最大4枚まで） */
export function TilePool({ handTiles, onAddTile, onDragStart }: Props) {
  const countInHand = (code: TileCode) => handTiles.filter((t) => t === code).length;
  const canAdd = handTiles.length < 14;

  return (
    <div className="flex flex-wrap gap-1">
      {ALL_TILE_TYPES.map((code) => {
        const used = countInHand(code);
        const available = 4 - used;
        const draggable = canAdd && available > 0;
        const clickable = canAdd && available > 0;

        return (
          <button
            key={code}
            type="button"
            draggable={draggable}
            onDragStart={(e) => {
              if (!draggable) return;
              e.dataTransfer.setData("tile-code", code);
              e.dataTransfer.effectAllowed = "copy";
              onDragStart?.(code);
            }}
            onClick={() => {
              if (clickable && onAddTile) onAddTile(code);
            }}
            disabled={!clickable}
            className={`flex items-center gap-0.5 rounded border border-transparent transition-colors ${clickable ? "cursor-grab active:cursor-grabbing hover:bg-zinc-100 hover:border-zinc-200" : "cursor-not-allowed opacity-40"}`}
            title={clickable ? `クリックまたはドラッグで追加（残り${available}枚）` : "同種4枚まで"}
          >
            <Tile code={code} />
          </button>
        );
      })}
    </div>
  );
}
