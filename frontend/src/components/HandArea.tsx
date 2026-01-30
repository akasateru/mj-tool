"use client";

import { type TileCode } from "@/lib/tiles";
import { Tile } from "./Tile";

const SLOT_COUNT = 14;

type Props = {
  handTiles: TileCode[];
  onHandChange: (tiles: TileCode[]) => void;
};

/** 手牌エリア：14スロット。プールからドロップ/クリックで追加、スロット間ドラッグで並べ替え、×で削除 */
export function HandArea({ handTiles, onHandChange }: Props) {
  const slots: (TileCode | null)[] = [...handTiles];
  while (slots.length < SLOT_COUNT) slots.push(null);

  function handleDropAt(dropIndex: number, e: React.DragEvent) {
    e.preventDefault();
    const code = e.dataTransfer.getData("tile-code");
    const fromIndex = e.dataTransfer.getData("hand-index");

    if (fromIndex !== "") {
      const from = parseInt(fromIndex, 10);
      if (from === dropIndex) return;
      const next = [...handTiles];
      const [removed] = next.splice(from, 1);
      if (!removed) return;
      next.splice(dropIndex > from ? dropIndex - 1 : dropIndex, 0, removed);
      onHandChange(next);
      return;
    }

    if (code && handTiles.length < SLOT_COUNT) {
      const count = handTiles.filter((t) => t === code).length;
      if (count >= 4) return;
      const next = [...handTiles];
      next.splice(dropIndex, 0, code as TileCode);
      onHandChange(next.slice(0, SLOT_COUNT));
    }
  }

  function handleRemoveAt(index: number) {
    const next = handTiles.filter((_, i) => i !== index);
    onHandChange(next);
  }

  return (
    <div className="flex flex-wrap gap-1 min-h-[2.5rem] rounded-lg border-2 border-dashed border-zinc-200 bg-zinc-50/50 p-2">
      {slots.slice(0, SLOT_COUNT).map((tile, i) => (
        <div
          key={i}
          className="relative flex items-center justify-center min-w-[1.75rem] h-8 rounded border border-transparent hover:border-zinc-300 hover:bg-zinc-100/80"
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = "copy";
          }}
          onDrop={(e) => handleDropAt(i, e)}
        >
          {tile ? (
            <div className="relative">
              <div
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData("hand-index", String(i));
                  e.dataTransfer.setData("tile-code", tile);
                  e.dataTransfer.effectAllowed = "move";
                }}
                className="cursor-grab active:cursor-grabbing"
                title="ドラッグで並べ替え"
              >
                <Tile code={tile} />
              </div>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  handleRemoveAt(i);
                }}
                className="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-zinc-200 text-zinc-600 hover:bg-red-500 hover:text-white text-[10px] font-bold leading-none"
                title="削除"
                aria-label="削除"
              >
                ×
              </button>
            </div>
          ) : (
            <span className="text-xs text-zinc-400 w-[1.75rem] text-center">＋</span>
          )}
        </div>
      ))}
    </div>
  );
}
