/**
 * 手牌文字列（riichi-tools 形式）をパースして牌コードの配列にし、表示用ラベルを返す
 */

const MAN = "萬";
const PIN = "筒";
const SOU = "索";
const NUM_ZH = ["一", "二", "三", "四", "五", "六", "七", "八", "九"] as const;
const HONORS = ["東", "南", "西", "北", "白", "發", "中"] as const;

export type TileCode = `${number}${"m" | "p" | "s" | "z"}`;

/** 牌コードから表示用ラベル（一萬, 東 など） */
export function tileLabel(code: string): string {
  if (code.length < 2) return code;
  const suit = code.slice(-1);
  const n = code.slice(0, -1);
  const num = parseInt(n, 10);
  if (suit === "m") return (NUM_ZH[num - 1] ?? n) + MAN;
  if (suit === "p") return (NUM_ZH[num - 1] ?? n) + PIN;
  if (suit === "s") return (NUM_ZH[num - 1] ?? n) + SOU;
  if (suit === "z" && num >= 1 && num <= 7) return HONORS[num - 1];
  return code;
}

/** 牌コードからスート種別（色分け用） */
export function tileSuit(code: string): "m" | "p" | "s" | "z" | null {
  if (code.length < 2) return null;
  const s = code.slice(-1);
  if (s === "m" || s === "p" || s === "s" || s === "z") return s;
  return null;
}

/**
 * 手牌文字列をパースして牌コードの配列を返す。
 * 鳴き (k1m), (p4m1), (123m0) なども展開する。パースできない部分は無視。
 */
export function parseHandString(handString: string): TileCode[] {
  const out: TileCode[] = [];
  const s = handString.trim();
  let i = 0;

  while (i < s.length) {
    // 鳴き: (k1m), (k4z1), (p4m1), (123m0), (s2s1) など
    if (s[i] === "(") {
      const close = s.indexOf(")", i);
      if (close === -1) {
        i++;
        continue;
      }
      const inner = s.slice(i + 1, close);
      i = close + 1;

      // 暗槓・明槓 (k1m) (k4z1) 小明槓 (s2s1)
      if (inner.startsWith("k") || inner.startsWith("s")) {
        const num = inner[1];
        const suit = inner[2];
        if (num && suit && "mpsz".includes(suit)) {
          const tile = (num === "0" ? "5" : num) + suit as TileCode;
          for (let j = 0; j < 4; j++) out.push(tile);
        }
        continue;
      }
      // ポン (p4m1)
      if (inner.startsWith("p")) {
        const num = inner[1];
        const suit = inner[2];
        if (num && suit && "mpsz".includes(suit)) {
          const tile = (num === "0" ? "5" : num) + suit as TileCode;
          for (let j = 0; j < 3; j++) out.push(tile);
        }
        continue;
      }
      // チー (123m0)
      if (/^\d{3}[msp][0-2]$/.test(inner)) {
        const suit = inner[3];
        const a = parseInt(inner[0], 10);
        const b = parseInt(inner[1], 10);
        const c = parseInt(inner[2], 10);
        out.push(`${a}${suit}` as TileCode, `${b}${suit}` as TileCode, `${c}${suit}` as TileCode);
        continue;
      }
      continue;
    }

    // 数字 + m/p/s/z
    let digits = "";
    while (i < s.length && /[0-9]/.test(s[i])) {
      digits += s[i];
      i++;
    }
    if (i < s.length && "mpsz".includes(s[i])) {
      const suit = s[i];
      i++;
      for (const d of digits) {
        const num = d === "0" ? 5 : d;
        out.push(`${num}${suit}` as TileCode);
      }
    }
  }

  return out;
}

/** 手牌を萬→筒→索→字、各スート内で 1〜9（字は1〜7）の順にソート */
export function sortHandTiles(tiles: TileCode[]): TileCode[] {
  const suitOrder: Record<string, number> = { m: 0, p: 1, s: 2, z: 3 };
  return [...tiles].sort((a, b) => {
    const suitA = a.slice(-1);
    const suitB = b.slice(-1);
    const numA = parseInt(a.slice(0, -1), 10);
    const numB = parseInt(b.slice(0, -1), 10);
    if (suitOrder[suitA] !== suitOrder[suitB]) return suitOrder[suitA] - suitOrder[suitB];
    return numA - numB;
  });
}

/** 全34種の牌コード。スートごとに 1,2,3… の順（萬→筒→索→字） */
export const ALL_TILE_TYPES: TileCode[] = [
  ...(["m", "p", "s"] as const).flatMap((s) =>
    (["1", "2", "3", "4", "5", "6", "7", "8", "9"] as const).map((n) => `${n}${s}` as TileCode),
  ),
  ...(["1", "2", "3", "4", "5", "6", "7"] as const).map((n) => `${n}z` as TileCode),
];

/**
 * 手牌配列を riichi-tools 形式の文字列に変換（スートごとにグループ化）
 */
export function handTilesToHandString(tiles: TileCode[]): string {
  const bySuit: Record<string, string[]> = { m: [], p: [], s: [], z: [] };
  for (const t of tiles) {
    const s = t.slice(-1) as "m" | "p" | "s" | "z";
    const n = t.slice(0, -1);
    if (bySuit[s]) bySuit[s].push(n);
  }
  return ["m", "p", "s", "z"].map((s) => bySuit[s].sort().join("") + (bySuit[s].length ? s : "")).join("");
}
